struct HooksSettings {
    post_commit: Vec<String>,
    post_squash: Vec<String>,
}

impl TryFrom<&jj_lib::settings::UserSettings> for HooksSettings {
    type Error = jj_lib::config::ConfigGetError;

    fn try_from(settings: &jj_lib::settings::UserSettings) -> Result<Self, Self::Error> {
        use jj_lib::config::ConfigGetResultExt as _;
        Ok(Self {
            post_commit: settings
                .get("x.hooks.post-commit")
                .optional()?
                .unwrap_or_default(),
            post_squash: settings
                .get("x.hooks.post-squash")
                .optional()?
                .unwrap_or_default(),
        })
    }
}

fn run_command(cmd: &[String]) -> anyhow::Result<()> {
    let Some((program, args)) = cmd.split_first() else {
        return Ok(());
    };
    let status = std::process::Command::new(program).args(args).status()?;
    anyhow::ensure!(status.success(), "{program}: {status}");
    Ok(())
}

fn run_post_op_hooks(h: &jj_cli::cli_util::CommandHelper) -> anyhow::Result<()> {
    let hooks = HooksSettings::try_from(h.settings())?;
    match h.matches().subcommand() {
        Some(("commit", _)) => run_command(&hooks.post_commit),
        Some(("squash", _)) => run_command(&hooks.post_squash),
        _ => Ok(()),
    }
}

fn main() -> std::process::ExitCode {
    jj_cli::cli_util::CliRunner::init()
        .add_dispatch_hook(|ui, helper, command| {
            command(ui, helper)?;
            run_post_op_hooks(helper).map_err(jj_cli::command_error::user_error)
        })
        .run()
        .into()
}
