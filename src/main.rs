use anyhow::Context as _;

fn run_command(cmd: &[String]) -> anyhow::Result<()> {
    let (program, args) = cmd.split_first().context("no command")?;
    let status = std::process::Command::new(program).args(args).status()?;
    anyhow::ensure!(status.success(), "{program}: {status}");
    Ok(())
}

fn run_post_op_hook(h: &jj_cli::cli_util::CommandHelper) -> anyhow::Result<()> {
    let key = match h.matches().subcommand() {
        Some(("commit", _)) => "x.hooks.post-commit",
        Some(("squash", _)) => "x.hooks.post-squash",
        _ => return Ok(()),
    };
    use jj_lib::config::ConfigGetResultExt as _;
    h.settings()
        .get::<Vec<String>>(key)
        .optional()?
        .map_or(Ok(()), |hook| run_command(&hook).context(key))
}

fn main() -> std::process::ExitCode {
    jj_cli::cli_util::CliRunner::init()
        .add_dispatch_hook(|ui, helper, command| {
            command(ui, helper)?;
            run_post_op_hook(helper).map_err(jj_cli::command_error::user_error)
        })
        .run()
        .into()
}
