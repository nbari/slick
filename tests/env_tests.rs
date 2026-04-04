//! Environment variable configuration tests
//!
//! These tests verify the behavior of `get_env()` without modifying
//! environment variables to avoid unsafe operations.

use slick::get_env;

const ALL_ENV_VARS: &[&str] = &[
    "SLICK_PROMPT_AWS_COLOR",
    "SLICK_PROMPT_CMD_MAX_EXEC_TIME",
    "SLICK_PROMPT_DEVPOD_COLOR",
    "SLICK_PROMPT_DEVPOD_SYMBOL",
    "SLICK_PROMPT_ERROR_COLOR",
    "SLICK_PROMPT_GIT_ACTION_COLOR",
    "SLICK_PROMPT_GIT_AUTH_COLOR",
    "SLICK_PROMPT_GIT_AUTH_SYMBOL",
    "SLICK_PROMPT_GIT_BRANCH_COLOR",
    "SLICK_PROMPT_GIT_BRANCH_SYMBOL",
    "SLICK_PROMPT_GIT_FETCH",
    "SLICK_PROMPT_GIT_MASTER_BRANCH_COLOR",
    "SLICK_PROMPT_GIT_REMOTE_COLOR",
    "SLICK_PROMPT_GIT_REMOTE_AHEAD",
    "SLICK_PROMPT_GIT_REMOTE_BEHIND",
    "SLICK_PROMPT_GIT_STAGED_COLOR",
    "SLICK_PROMPT_GIT_STATUS_COLOR",
    "SLICK_PROMPT_GIT_UNAME_COLOR",
    "SLICK_PROMPT_K8S_COLOR",
    "SLICK_PROMPT_NON_BREAKING_SPACE",
    "SLICK_PROMPT_PATH_COLOR",
    "SLICK_PROMPT_PYTHON_ENV_COLOR",
    "SLICK_PROMPT_ROOT_COLOR",
    "SLICK_PROMPT_ROOT_SYMBOL",
    "SLICK_PROMPT_SSH_COLOR",
    "SLICK_PROMPT_SYMBOL",
    "SLICK_PROMPT_SYMBOL_COLOR",
    "SLICK_PROMPT_TIME_ELAPSED_COLOR",
    "SLICK_PROMPT_TOOLBOX_COLOR",
    "SLICK_PROMPT_TRANSIENT",
    "SLICK_PROMPT_TOOLBOX_SYMBOL",
    "SLICK_PROMPT_VICMD_COLOR",
    "SLICK_PROMPT_VICMD_SYMBOL",
];

const OPTIONAL_EMPTY_ENV_VARS: &[&str] = &[];

#[test]
fn test_get_env_returns_non_empty_defaults() {
    assert!(!get_env("SLICK_PROMPT_SYMBOL").is_empty());
    assert!(!get_env("SLICK_PROMPT_ROOT_SYMBOL").is_empty());
    assert!(!get_env("SLICK_PROMPT_VICMD_SYMBOL").is_empty());
}

#[test]
fn test_get_env_git_remote_symbols_exist() {
    assert!(!get_env("SLICK_PROMPT_GIT_REMOTE_AHEAD").is_empty());
    assert!(!get_env("SLICK_PROMPT_GIT_REMOTE_BEHIND").is_empty());
    assert!(!get_env("SLICK_PROMPT_GIT_AUTH_SYMBOL").is_empty());
    assert!(!get_env("SLICK_PROMPT_TOOLBOX_SYMBOL").is_empty());
}

#[test]
fn test_get_env_color_values_are_numeric_or_named() {
    let colors = [
        "SLICK_PROMPT_AWS_COLOR",
        "SLICK_PROMPT_DEVPOD_COLOR",
        "SLICK_PROMPT_ERROR_COLOR",
        "SLICK_PROMPT_GIT_BRANCH_COLOR",
        "SLICK_PROMPT_GIT_MASTER_BRANCH_COLOR",
        "SLICK_PROMPT_GIT_STATUS_COLOR",
        "SLICK_PROMPT_K8S_COLOR",
        "SLICK_PROMPT_PATH_COLOR",
        "SLICK_PROMPT_PYTHON_ENV_COLOR",
        "SLICK_PROMPT_ROOT_COLOR",
        "SLICK_PROMPT_SSH_COLOR",
        "SLICK_PROMPT_SYMBOL_COLOR",
        "SLICK_PROMPT_TOOLBOX_COLOR",
    ];

    for color_var in &colors {
        let val = get_env(color_var);
        assert!(!val.is_empty(), "{color_var} should not be empty");
        assert_ne!(val, "??", "{color_var} should be a valid config");
    }
}

#[test]
fn test_get_env_git_fetch_is_boolean() {
    let val = get_env("SLICK_PROMPT_GIT_FETCH");
    assert!(val == "0" || val == "1", "GIT_FETCH should be 0 or 1");
}

#[test]
fn test_get_env_transient_is_boolean() {
    let val = get_env("SLICK_PROMPT_TRANSIENT");
    assert!(val == "0" || val == "1", "TRANSIENT should be 0 or 1");
}

#[test]
fn test_get_env_max_exec_time_is_numeric() {
    let val = get_env("SLICK_PROMPT_CMD_MAX_EXEC_TIME");
    assert!(
        val.parse::<u64>().is_ok(),
        "CMD_MAX_EXEC_TIME should be numeric"
    );
}

#[test]
fn test_get_env_special_chars() {
    let nbsp = get_env("SLICK_PROMPT_NON_BREAKING_SPACE");
    assert_eq!(nbsp.chars().count(), 1);

    let toolbox_symbol = get_env("SLICK_PROMPT_TOOLBOX_SYMBOL");
    assert_eq!(toolbox_symbol, "▣");

    let devpod_symbol = get_env("SLICK_PROMPT_DEVPOD_SYMBOL");
    assert_eq!(devpod_symbol, "");

    let git_branch_symbol = get_env("SLICK_PROMPT_GIT_BRANCH_SYMBOL");
    assert_eq!(git_branch_symbol, "");
}

#[test]
fn test_all_env_vars_have_defaults() {
    for var in ALL_ENV_VARS {
        let value = get_env(var);
        assert_ne!(value, "??", "{var} should resolve to a known default");
        if !OPTIONAL_EMPTY_ENV_VARS.contains(var) {
            assert!(!value.is_empty(), "{var} should have a default value");
        }
    }
}

#[test]
fn test_get_env_unknown_vars_return_question_marks() {
    assert_eq!(get_env("UNKNOWN_VAR"), "??");
    assert_eq!(get_env("DOES_NOT_EXIST"), "??");
    assert_eq!(get_env(""), "??");
}
