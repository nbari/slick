use clap::{Arg, Command};
use slick::{precmd, prompt};

fn main() {
    let matches = Command::new("slick")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(true)
        .subcommand(
            Command::new("precmd")
                .about("precmd")
                .override_help("Executed before each prompt."),
        )
        .subcommand(
            Command::new("prompt")
                .about("prompt")
                .override_help(
                    r#"Builds the prompt, render is affected by this environment vars:

The default values are:

    SLICK_PROMPT_CMD_MAX_EXEC_TIME=5
    SLICK_PROMPT_ERROR_COLOR=196
    SLICK_PROMPT_GIT_ACTION_COLOR=3
    SLICK_PROMPT_GIT_BRANCH_COLOR=3
    SLICK_PROMPT_GIT_FETCH=1 (if set to 0 disables git fetch)
    SLICK_PROMPT_GIT_MASTER_BRANCH_COLOR=160
    SLICK_PROMPT_GIT_REMOTE_COLOR=6
    SLICK_PROMPT_GIT_REMOTE_AHEAD=⇡
    SLICK_PROMPT_GIT_REMOTE_BEHIND=⇣
    SLICK_PROMPT_GIT_STAGED_COLOR=7
    SLICK_PROMPT_GIT_STATUS_COLOR=5
    SLICK_PROMPT_GIT_UNAME_COLOR=8
    SLICK_PROMPT_NON_BREAKING_SPACE="\u{a0}"
    SLICK_PROMPT_PATH_COLOR=74
    SLICK_PROMPT_ROOT_COLOR=1
    SLICK_PROMPT_ROOT_SYMBOL=#
    SLICK_PROMPT_SSH_COLOR=8
    SLICK_PROMPT_SYMBOL="$"
    SLICK_PROMPT_SYMBOL_COLOR=5
    SLICK_PROMPT_TIME_ELAPSED_COLOR=3
    SLICK_PROMPT_VICMD_COLOR=3
    SLICK_PROMPT_VICMD_SYMBOL=">"
    PIPENV_ACTIVE_COLOR=7
"#,
                )
                .arg(Arg::new("last_return_code").short('r').takes_value(true))
                .arg(Arg::new("keymap").short('k').takes_value(true))
                .arg(Arg::new("data").short('d').takes_value(true))
                .arg(Arg::new("time").short('t').takes_value(true)),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("precmd", _)) => precmd::render(),
        Some(("prompt", sub_m)) => prompt::display(sub_m),
        _ => (),
    }
}
