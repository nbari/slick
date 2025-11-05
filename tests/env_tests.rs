//! Environment variable configuration tests
//!
//! These tests verify the behavior of get_env() without modifying
//! environment variables to avoid unsafe operations.

#[test]
fn test_get_env_returns_non_empty_defaults() {
    // Test that known env vars return non-empty strings
    assert!(!slick::get_env("SLICK_PROMPT_SYMBOL").is_empty());
    assert!(!slick::get_env("SLICK_PROMPT_ROOT_SYMBOL").is_empty());
    assert!(!slick::get_env("SLICK_PROMPT_VICMD_SYMBOL").is_empty());
}

#[test]
fn test_get_env_git_remote_symbols_exist() {
    // These should return some symbol (user may have customized)
    assert!(!slick::get_env("SLICK_PROMPT_GIT_REMOTE_AHEAD").is_empty());
    assert!(!slick::get_env("SLICK_PROMPT_GIT_REMOTE_BEHIND").is_empty());
}

#[test]
fn test_get_env_color_values_are_numeric_or_named() {
    let colors = [
        "SLICK_PROMPT_ERROR_COLOR",
        "SLICK_PROMPT_PATH_COLOR",
        "SLICK_PROMPT_GIT_BRANCH_COLOR",
        "SLICK_PROMPT_GIT_STATUS_COLOR",
        "SLICK_PROMPT_ROOT_COLOR",
        "SLICK_PROMPT_SSH_COLOR",
        "SLICK_PROMPT_SYMBOL_COLOR",
        "SLICK_PROMPT_GIT_MASTER_BRANCH_COLOR",
    ];

    for color_var in &colors {
        let val = slick::get_env(color_var);
        // Should return either a number or a color name
        assert!(!val.is_empty(), "{} should not be empty", color_var);
        assert_ne!(val, "??", "{} should be a valid config", color_var);
    }
}

#[test]
fn test_get_env_git_fetch_is_boolean() {
    let val = slick::get_env("SLICK_PROMPT_GIT_FETCH");
    assert!(val == "0" || val == "1", "GIT_FETCH should be 0 or 1");
}

#[test]
fn test_get_env_max_exec_time_is_numeric() {
    let val = slick::get_env("SLICK_PROMPT_CMD_MAX_EXEC_TIME");
    assert!(
        val.parse::<u64>().is_ok(),
        "CMD_MAX_EXEC_TIME should be numeric"
    );
}

#[test]
fn test_get_env_special_chars() {
    // Non-breaking space should be a single Unicode character
    let nbsp = slick::get_env("SLICK_PROMPT_NON_BREAKING_SPACE");
    assert_eq!(nbsp.chars().count(), 1);
}

#[test]
fn test_get_env_unknown() {
    assert_eq!(slick::get_env("UNKNOWN_VAR"), "??");
    assert_eq!(slick::get_env("DOES_NOT_EXIST"), "??");
    assert_eq!(slick::get_env(""), "??");
}
