use jj_lib::config::ConfigGetResultExt as _;

struct HooksSettings {
    post_commit: Vec<String>,
    post_squash: Vec<String>,
}

impl TryFrom<&jj_lib::settings::UserSettings> for HooksSettings {
    type Error = jj_lib::config::ConfigGetError;

    fn try_from(settings: &jj_lib::settings::UserSettings) -> Result<Self, Self::Error> {
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

fn run_hook(hook: &[String]) -> Result<(), jj_cli::command_error::CommandError> {
    if hook.is_empty() {
        return Ok(());
    }
    let status = std::process::Command::new(&hook[0])
        .args(&hook[1..])
        .status()
        .map_err(|err| {
            jj_cli::command_error::user_error_with_message(
                format!("Hook '{}' failed to run", hook[0]),
                err,
            )
        })?;
    match status.code() {
        Some(0) => Ok(()),
        Some(exit_code) => Err(jj_cli::command_error::user_error(format!(
            "Hook '{}' exited with code {}",
            hook[0], exit_code
        ))),
        None => Err(jj_cli::command_error::user_error(format!(
            "Hook '{}' was terminated by {}",
            hook[0], status
        ))),
    }
}

fn main() -> std::process::ExitCode {
    jj_cli::cli_util::CliRunner::init()
        .add_dispatch_hook(|ui, helper, command| {
            command(ui, helper)?;
            let hooks = HooksSettings::try_from(helper.settings())?;
            match helper.matches().subcommand() {
                Some(("commit", _)) => run_hook(&hooks.post_commit),
                Some(("squash", _)) => run_hook(&hooks.post_squash),
                _ => Ok(()),
            }
        })
        .run()
        .into()
}
