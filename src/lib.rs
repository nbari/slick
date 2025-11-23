pub mod git;
pub mod precmd;
pub mod prompt;

use std::env;
use std::sync::OnceLock;

// Cache for environment variable defaults to avoid repeated env::var() calls
static ENV_CACHE: OnceLock<EnvDefaults> = OnceLock::new();

struct EnvDefaults {
    cmd_max_exec_time: String,
    error_color: String,
    git_action_color: String,
    git_auth_color: String,
    git_auth_symbol: String,
    git_branch_color: String,
    git_fetch: String,
    git_master_branch_color: String,
    git_remote_color: String,
    git_remote_ahead: String,
    git_remote_behind: String,
    git_staged_color: String,
    git_status_color: String,
    git_uname_color: String,
    non_breaking_space: String,
    path_color: String,
    root_color: String,
    root_symbol: String,
    ssh_color: String,
    symbol: String,
    symbol_color: String,
    time_elapsed_color: String,
    vicmd_color: String,
    vicmd_symbol: String,
}

impl EnvDefaults {
    fn new() -> Self {
        Self {
            cmd_max_exec_time: env::var("SLICK_PROMPT_CMD_MAX_EXEC_TIME")
                .unwrap_or_else(|_| "5".into()),
            error_color: env::var("SLICK_PROMPT_ERROR_COLOR").unwrap_or_else(|_| "196".into()),
            git_action_color: env::var("SLICK_PROMPT_GIT_ACTION_COLOR")
                .unwrap_or_else(|_| "3".into()),
            git_auth_color: env::var("SLICK_PROMPT_GIT_AUTH_COLOR")
                .unwrap_or_else(|_| "red".into()),
            git_auth_symbol: env::var("SLICK_PROMPT_GIT_AUTH_SYMBOL")
                .unwrap_or_else(|_| "🔒".into()),
            git_branch_color: env::var("SLICK_PROMPT_GIT_BRANCH_COLOR")
                .unwrap_or_else(|_| "3".into()),
            git_fetch: env::var("SLICK_PROMPT_GIT_FETCH").unwrap_or_else(|_| "1".into()),
            git_master_branch_color: env::var("SLICK_PROMPT_GIT_MASTER_BRANCH_COLOR")
                .unwrap_or_else(|_| "160".into()),
            git_remote_color: env::var("SLICK_PROMPT_GIT_REMOTE_COLOR")
                .unwrap_or_else(|_| "6".into()),
            git_remote_ahead: env::var("SLICK_PROMPT_GIT_REMOTE_AHEAD")
                .unwrap_or_else(|_| "\u{21e1}".into()),
            git_remote_behind: env::var("SLICK_PROMPT_GIT_REMOTE_BEHIND")
                .unwrap_or_else(|_| "\u{21e3}".into()),
            git_staged_color: env::var("SLICK_PROMPT_GIT_STAGED_COLOR")
                .unwrap_or_else(|_| "7".into()),
            git_status_color: env::var("SLICK_PROMPT_GIT_STATUS_COLOR")
                .unwrap_or_else(|_| "5".into()),
            git_uname_color: env::var("SLICK_PROMPT_GIT_UNAME_COLOR")
                .unwrap_or_else(|_| "8".into()),
            non_breaking_space: env::var("SLICK_PROMPT_NON_BREAKING_SPACE")
                .unwrap_or_else(|_| "\u{a0}".into()),
            path_color: env::var("SLICK_PROMPT_PATH_COLOR").unwrap_or_else(|_| "74".into()),
            root_color: env::var("SLICK_PROMPT_ROOT_COLOR").unwrap_or_else(|_| "1".into()),
            root_symbol: env::var("SLICK_PROMPT_ROOT_SYMBOL").unwrap_or_else(|_| "#".into()),
            ssh_color: env::var("SLICK_PROMPT_SSH_COLOR").unwrap_or_else(|_| "8".into()),
            symbol: env::var("SLICK_PROMPT_SYMBOL").unwrap_or_else(|_| "$".into()),
            symbol_color: env::var("SLICK_PROMPT_SYMBOL_COLOR").unwrap_or_else(|_| "5".into()),
            time_elapsed_color: env::var("SLICK_PROMPT_TIME_ELAPSED_COLOR")
                .unwrap_or_else(|_| "3".into()),
            vicmd_color: env::var("SLICK_PROMPT_VICMD_COLOR").unwrap_or_else(|_| "3".into()),
            vicmd_symbol: env::var("SLICK_PROMPT_VICMD_SYMBOL").unwrap_or_else(|_| ">".into()),
        }
    }
}

#[must_use]
pub fn get_env(e: &str) -> &str {
    let cache = ENV_CACHE.get_or_init(EnvDefaults::new);

    match e {
        "SLICK_PROMPT_CMD_MAX_EXEC_TIME" => &cache.cmd_max_exec_time,
        "SLICK_PROMPT_ERROR_COLOR" => &cache.error_color,
        "SLICK_PROMPT_GIT_ACTION_COLOR" => &cache.git_action_color,
        "SLICK_PROMPT_GIT_AUTH_COLOR" => &cache.git_auth_color,
        "SLICK_PROMPT_GIT_AUTH_SYMBOL" => &cache.git_auth_symbol,
        "SLICK_PROMPT_GIT_BRANCH_COLOR" => &cache.git_branch_color,
        "SLICK_PROMPT_GIT_FETCH" => &cache.git_fetch,
        "SLICK_PROMPT_GIT_MASTER_BRANCH_COLOR" => &cache.git_master_branch_color,
        "SLICK_PROMPT_GIT_REMOTE_COLOR" => &cache.git_remote_color,
        "SLICK_PROMPT_GIT_REMOTE_AHEAD" => &cache.git_remote_ahead,
        "SLICK_PROMPT_GIT_REMOTE_BEHIND" => &cache.git_remote_behind,
        "SLICK_PROMPT_GIT_STAGED_COLOR" => &cache.git_staged_color,
        "SLICK_PROMPT_GIT_STATUS_COLOR" => &cache.git_status_color,
        "SLICK_PROMPT_GIT_UNAME_COLOR" => &cache.git_uname_color,
        "SLICK_PROMPT_NON_BREAKING_SPACE" => &cache.non_breaking_space,
        "SLICK_PROMPT_PATH_COLOR" => &cache.path_color,
        "SLICK_PROMPT_ROOT_COLOR" => &cache.root_color,
        "SLICK_PROMPT_ROOT_SYMBOL" => &cache.root_symbol,
        "SLICK_PROMPT_SSH_COLOR" => &cache.ssh_color,
        "SLICK_PROMPT_SYMBOL" => &cache.symbol,
        "SLICK_PROMPT_SYMBOL_COLOR" => &cache.symbol_color,
        "SLICK_PROMPT_TIME_ELAPSED_COLOR" => &cache.time_elapsed_color,
        "SLICK_PROMPT_VICMD_COLOR" => &cache.vicmd_color,
        "SLICK_PROMPT_VICMD_SYMBOL" => &cache.vicmd_symbol,
        _ => "??",
    }
}

// For environment variables that aren't cached in EnvDefaults, use this function
#[must_use]
pub fn get_env_var(e: &str) -> String {
    env::var(e).unwrap_or_default()
}

#[must_use]
pub fn get_env_var_or(e: &str, default: &str) -> String {
    env::var(e).unwrap_or_else(|_| default.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_env_returns_default_symbol() {
        // Test with an env var that's unlikely to be set in test environment
        assert_eq!(get_env("SLICK_PROMPT_SYMBOL"), "$");
    }

    #[test]
    fn test_get_env_returns_default_colors() {
        assert_eq!(get_env("SLICK_PROMPT_ERROR_COLOR"), "196");
        assert_eq!(get_env("SLICK_PROMPT_PATH_COLOR"), "74");
        assert_eq!(get_env("SLICK_PROMPT_GIT_BRANCH_COLOR"), "3");
    }

    #[test]
    fn test_get_env_unknown_returns_question_marks() {
        assert_eq!(get_env("UNKNOWN_VAR"), "??");
    }

    #[test]
    fn test_get_env_var_returns_empty_for_missing() {
        // Test with an env var that's unlikely to be set
        assert_eq!(get_env_var("UNLIKELY_TO_EXIST_VAR_12345"), "");
    }

    #[test]
    fn test_get_env_var_or_returns_default() {
        assert_eq!(
            get_env_var_or("UNLIKELY_TO_EXIST_VAR_12345", "default"),
            "default"
        );
    }
}
