use slick::get_env;

#[test]
fn test_default_symbol() {
    // Test that get_env returns values (defaults are set at startup)
    let symbol = get_env("SLICK_PROMPT_SYMBOL");
    assert!(!symbol.is_empty());
}

#[test]
fn test_custom_colors() {
    // Test that get_env returns values for color variables
    let error_color = get_env("SLICK_PROMPT_ERROR_COLOR");
    assert!(!error_color.is_empty());

    let path_color = get_env("SLICK_PROMPT_PATH_COLOR");
    assert!(!path_color.is_empty());

    let git_branch_color = get_env("SLICK_PROMPT_GIT_BRANCH_COLOR");
    assert!(!git_branch_color.is_empty());
}

#[test]
fn test_git_symbols() {
    let ahead = get_env("SLICK_PROMPT_GIT_REMOTE_AHEAD");
    assert!(!ahead.is_empty());
    // Just verify it returns something, don't check exact value
    // as it could be customized via envrc

    let behind = get_env("SLICK_PROMPT_GIT_REMOTE_BEHIND");
    assert!(!behind.is_empty());

    let auth = get_env("SLICK_PROMPT_GIT_AUTH_SYMBOL");
    assert!(!auth.is_empty());
}

#[test]
fn test_fetch_setting() {
    let fetch = get_env("SLICK_PROMPT_GIT_FETCH");
    // Default should be "1"
    assert!(fetch == "1" || fetch == "0");
}

#[test]
fn test_all_env_vars_have_defaults() {
    // Verify all env vars return non-empty values
    let vars = vec![
        "SLICK_PROMPT_CMD_MAX_EXEC_TIME",
        "SLICK_PROMPT_ERROR_COLOR",
        "SLICK_PROMPT_GIT_ACTION_COLOR",
        "SLICK_PROMPT_GIT_AUTH_COLOR",
        "SLICK_PROMPT_GIT_AUTH_SYMBOL",
        "SLICK_PROMPT_GIT_BRANCH_COLOR",
        "SLICK_PROMPT_GIT_FETCH",
        "SLICK_PROMPT_GIT_MASTER_BRANCH_COLOR",
        "SLICK_PROMPT_GIT_REMOTE_COLOR",
        "SLICK_PROMPT_GIT_REMOTE_AHEAD",
        "SLICK_PROMPT_GIT_REMOTE_BEHIND",
        "SLICK_PROMPT_GIT_STAGED_COLOR",
        "SLICK_PROMPT_GIT_STATUS_COLOR",
        "SLICK_PROMPT_GIT_UNAME_COLOR",
        "SLICK_PROMPT_NON_BREAKING_SPACE",
        "SLICK_PROMPT_PATH_COLOR",
        "SLICK_PROMPT_ROOT_COLOR",
        "SLICK_PROMPT_ROOT_SYMBOL",
        "SLICK_PROMPT_SSH_COLOR",
        "SLICK_PROMPT_SYMBOL",
        "SLICK_PROMPT_SYMBOL_COLOR",
        "SLICK_PROMPT_TIME_ELAPSED_COLOR",
        "SLICK_PROMPT_VICMD_COLOR",
        "SLICK_PROMPT_VICMD_SYMBOL",
    ];

    for var in vars {
        let value = get_env(var);
        assert!(!value.is_empty(), "{} should have a default value", var);
    }
}
