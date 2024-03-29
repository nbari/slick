pub mod precmd;
pub mod prompt;

use std::env;

#[must_use]
pub fn get_env(e: &str) -> String {
    match e {
        "SLICK_PROMPT_CMD_MAX_EXEC_TIME" => env::var(e).unwrap_or_else(|_| "5".into()),
        "SLICK_PROMPT_ERROR_COLOR" => env::var(e).unwrap_or_else(|_| "196".into()),
        "SLICK_PROMPT_GIT_ACTION_COLOR" => env::var(e).unwrap_or_else(|_| "3".into()),
        "SLICK_PROMPT_GIT_BRANCH_COLOR" => env::var(e).unwrap_or_else(|_| "3".into()),
        "SLICK_PROMPT_GIT_FETCH" => env::var(e).unwrap_or_else(|_| "1".into()),
        "SLICK_PROMPT_GIT_MASTER_BRANCH_COLOR" => env::var(e).unwrap_or_else(|_| "160".into()),
        "SLICK_PROMPT_GIT_REMOTE_COLOR" => env::var(e).unwrap_or_else(|_| "6".into()),
        "SLICK_PROMPT_GIT_REMOTE_AHEAD" => env::var(e).unwrap_or_else(|_| "\u{21e1}".into()),
        "SLICK_PROMPT_GIT_REMOTE_BEHIND" => env::var(e).unwrap_or_else(|_| "\u{21e3}".into()),
        "SLICK_PROMPT_GIT_STAGED_COLOR" => env::var(e).unwrap_or_else(|_| "7".into()),
        "SLICK_PROMPT_GIT_STATUS_COLOR" => env::var(e).unwrap_or_else(|_| "5".into()),
        "SLICK_PROMPT_GIT_UNAME_COLOR" => env::var(e).unwrap_or_else(|_| "8".into()),
        "SLICK_PROMPT_NON_BREAKING_SPACE" => env::var(e).unwrap_or_else(|_| "\u{a0}".into()),
        "SLICK_PROMPT_PATH_COLOR" => env::var(e).unwrap_or_else(|_| "74".into()),
        "SLICK_PROMPT_ROOT_COLOR" => env::var(e).unwrap_or_else(|_| "1".into()),
        "SLICK_PROMPT_ROOT_SYMBOL" => env::var(e).unwrap_or_else(|_| "#".into()),
        "SLICK_PROMPT_SSH_COLOR" => env::var(e).unwrap_or_else(|_| "8".into()),
        "SLICK_PROMPT_SYMBOL" => env::var(e).unwrap_or_else(|_| "$".into()),
        "SLICK_PROMPT_SYMBOL_COLOR" => env::var(e).unwrap_or_else(|_| "5".into()),
        "SLICK_PROMPT_TIME_ELAPSED_COLOR" => env::var(e).unwrap_or_else(|_| "3".into()),
        "SLICK_PROMPT_VICMD_COLOR" => env::var(e).unwrap_or_else(|_| "3".into()),
        "SLICK_PROMPT_VICMD_SYMBOL" => env::var(e).unwrap_or_else(|_| ">".into()),
        "SLICK_PROMPT_NO_GIT_UNAME" => env::var(e).unwrap_or_else(|_| String::new()),
        "PIPENV_ACTIVE" => env::var(e).unwrap_or_else(|_| String::new()),
        "PIPENV_ACTIVE_COLOR" => env::var(e).unwrap_or_else(|_| "7".into()),
        "VIRTUAL_ENV" => env::var(e).unwrap_or_else(|_| String::new()),
        _ => "??".into(),
    }
}
