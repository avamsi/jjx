#[derive(clap::Subcommand)]
enum XCommand {
    Run {
        #[arg(short, long, default_value = "@")]
        revision: jj_cli::cli_util::RevisionArg,
        #[arg(required = true)]
        command: Vec<String>,
    },
}

#[derive(clap::Subcommand)]
enum Command {
    #[command(subcommand)]
    X(XCommand),
}

fn run_command_in<S: AsRef<std::ffi::OsStr>>(
    cmd: &[S],
    dir: Option<&std::path::Path>,
) -> std::io::Result<()> {
    let (program, args) = cmd
        .split_first()
        .ok_or(std::io::Error::other("no command"))?;
    let mut process = std::process::Command::new(program);
    if let Some(dir) = dir {
        process.current_dir(dir);
    }
    let status = process.args(args).status()?;
    status.success().then_some(()).ok_or_else(|| {
        std::io::Error::other(format!("{}: {status}", program.as_ref().to_string_lossy()))
    })
}

fn cmd_x_run(
    h: &jj_cli::cli_util::CommandHelper,
    r: &jj_cli::cli_util::RevisionArg,
    cmd: &[String],
) -> Result<(), jj_cli::command_error::CommandError> {
    let base = jj_cli::cli_util::find_workspace_dir(h.cwd()).join(".jj/x/workspaces");
    std::fs::create_dir_all(&base)?;
    let ws = (1..=42)
        .map(|i| base.join(i.to_string()))
        .find(|d| !d.exists())
        .ok_or(jj_cli::command_error::user_error(
            "no more than 42 workspaces",
        ))?;
    let jj = std::env::current_exe()?.to_string_lossy().into_owned();
    run_command_in(
        &[
            jj.as_str(),
            "workspace",
            "add",
            "--revision",
            r.as_ref(),
            &ws.to_string_lossy(),
        ],
        None,
    )?;
    run_command_in(cmd, Some(&ws))
        .and(run_command_in(
            &[jj.as_str(), "workspace", "forget"],
            Some(&ws),
        ))
        .and(std::fs::remove_dir_all(&ws))
        .map_err(|e| e.into())
}

fn run_post_op_hook(
    h: &jj_cli::cli_util::CommandHelper,
) -> Result<(), jj_cli::command_error::CommandError> {
    let key = match h.matches().subcommand() {
        Some(("commit", _)) => "x.hooks.post-commit",
        Some(("squash", _)) => "x.hooks.post-squash",
        _ => return Ok(()),
    };
    use jj_lib::config::ConfigGetResultExt as _;
    h.settings()
        .get::<Vec<String>>(key)
        .optional()?
        .map_or(Ok(()), |hook| {
            run_command_in(&hook, Some(jj_cli::cli_util::find_workspace_dir(h.cwd())))
                .map_err(|e| jj_cli::command_error::user_error_with_message(key, e))
        })
}

fn main() -> std::process::ExitCode {
    jj_cli::cli_util::CliRunner::init()
        .add_subcommand(|_ui, helper, Command::X(cmd)| match cmd {
            XCommand::Run { revision, command } => cmd_x_run(helper, &revision, &command),
        })
        .add_dispatch_hook(|ui, helper, command| {
            command(ui, helper)?;
            run_post_op_hook(helper)
        })
        .run()
        .into()
}
