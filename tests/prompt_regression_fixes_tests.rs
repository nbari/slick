//! Regression tests for prompt rendering and prompt-related environment options.

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn prompt_command(data: &str, keymap: &str, last_return_code: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_slick"));
    command
        .args([
            "prompt",
            "-e",
            "0",
            "-r",
            last_return_code,
            "-k",
            keymap,
            "-d",
            data,
        ])
        .env_clear()
        .env("SLICK_PROMPT_CURSOR_SHAPE", "");
    command
}

fn successful_stdout(command: &mut Command) -> String {
    let output = command.output().expect("command should run");
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout should be UTF-8")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn render_with_zsh(prompt: &str, mode: &str) -> String {
    let output = Command::new("zsh")
        .args([
            "-f",
            "-c",
            r#"psvar=('$' '`' '\'); case $SLICK_TEST_PROMPT_MODE in off) unsetopt PROMPT_SUBST; PROMPT=$SLICK_TEST_PROMPT ;; on) setopt PROMPT_SUBST; PROMPT=$SLICK_TEST_PROMPT ;; transition) unsetopt PROMPT_SUBST; PROMPT=$SLICK_TEST_PROMPT; setopt PROMPT_SUBST ;; esac; print -r -P -- "$PROMPT""#,
        ])
        .env("SLICK_TEST_PROMPT", prompt)
        .env("SLICK_TEST_PROMPT_MODE", mode)
        .output()
        .expect("zsh should run");
    assert_success(&output);
    String::from_utf8(output.stdout).expect("zsh output should be UTF-8")
}

#[test]
fn test_no_git_uname_case_insensitive_boolean_values() {
    let data = serde_json::json!({ "u_name": "visible-git-user" }).to_string();
    let cases = [
        (None, true),
        (Some("0"), true),
        (Some("false"), true),
        (Some("FaLsE"), true),
        (Some("no"), true),
        (Some("OFF"), true),
        (Some("invalid"), true),
        (Some("1"), false),
        (Some("true"), false),
        (Some("TrUe"), false),
        (Some("YES"), false),
        (Some("oN"), false),
    ];

    for (value, should_show) in cases {
        let mut command = prompt_command(&data, "main", "0");
        if let Some(value) = value {
            command.env("SLICK_PROMPT_NO_GIT_UNAME", value);
        }

        let stdout = successful_stdout(&mut command);
        assert_eq!(
            stdout.contains("visible-git-user"),
            should_show,
            "SLICK_PROMPT_NO_GIT_UNAME={value:?}"
        );
    }
}

#[test]
fn test_vicmd_color_uses_keymap_when_symbols_are_identical() {
    let cases = [
        ("main", "0", "101"),
        ("main", "1", "103"),
        ("vicmd", "0", "102"),
        ("vicmd", "1", "102"),
    ];

    for (keymap, last_return_code, expected_color) in cases {
        let mut command = prompt_command("", keymap, last_return_code);
        command.envs([
            ("SLICK_PROMPT_SYMBOL", "@"),
            ("SLICK_PROMPT_ROOT_SYMBOL", "@"),
            ("SLICK_PROMPT_VICMD_SYMBOL", "@"),
            ("SLICK_PROMPT_SYMBOL_COLOR", "101"),
            ("SLICK_PROMPT_VICMD_COLOR", "102"),
            ("SLICK_PROMPT_ERROR_COLOR", "103"),
        ]);

        let stdout = successful_stdout(&mut command);
        assert!(
            stdout.contains(&format!("\n%F{{{expected_color}}}@%f")),
            "keymap={keymap:?}, return code={last_return_code:?}, prompt={stdout:?}"
        );
    }
}

#[test]
fn test_untrusted_prompt_text_renders_literally_in_both_prompt_modes() {
    let target_tmp = Path::new(env!("CARGO_TARGET_TMPDIR"));
    fs::create_dir_all(target_tmp).expect("Cargo target test directory should exist");
    let tempdir = tempfile::Builder::new()
        .prefix("slick-prompt-regression-")
        .tempdir_in(target_tmp)
        .expect("test directory should be created");

    let branch = r"branch-$(id)-%n-`id`-\literal";
    let git_user = r"user-$(id)-%n-`id`-\literal";
    let context = r"context-$(id)-%n-`id`-\literal";
    let path_component = r"path-$(id)-%n-`id`-\literal";
    let home = tempdir.path().join("home");
    let workdir = tempdir.path().join(path_component);
    fs::create_dir_all(&home).expect("home should be created");
    fs::create_dir_all(&workdir).expect("work directory should be created");

    let data = serde_json::json!({
        "branch": branch,
        "u_name": git_user,
    })
    .to_string();
    let mut command = prompt_command(&data, "main", "0");
    command
        .current_dir(&workdir)
        .env("HOME", &home)
        .env("AWS_PROFILE", context)
        .env("SLICK_PROMPT_SHORT_PATH", "1")
        .env("SLICK_PROMPT_GIT_BRANCH_SYMBOL", "")
        .env("SLICK_PROMPT_NO_GIT_UNAME", "0")
        .env("_SLICK_PROMPT_PSVAR_DOLLAR", "1")
        .env("_SLICK_PROMPT_PSVAR_BACKTICK", "2")
        .env("_SLICK_PROMPT_PSVAR_BACKSLASH", "3");
    let prompt = successful_stdout(&mut command);

    assert!(
        prompt.contains("%F{"),
        "slick formatting should remain active"
    );
    assert!(
        prompt.contains(r"branch-%1v(id)-%%n-%2vid%2v-%3vliteral"),
        "canonical prompt should contain transition-safe literals: {prompt:?}"
    );

    for (mode, description) in [
        ("off", "PROMPT_SUBST off"),
        ("on", "PROMPT_SUBST on"),
        ("transition", "PROMPT_SUBST off-to-on transition"),
    ] {
        let rendered = render_with_zsh(&prompt, mode);
        for literal in [
            branch,
            git_user,
            path_component,
            &format!("(aws {context})"),
        ] {
            assert!(
                rendered.contains(literal),
                "{description}: literal {literal:?} missing from rendered prompt: {rendered:?}"
            );
        }
        assert!(
            !rendered.contains("uid="),
            "{description}: command substitution executed: {rendered:?}"
        );
    }

    let mut direct_command = prompt_command(&data, "main", "0");
    direct_command
        .current_dir(&workdir)
        .env("HOME", &home)
        .env("AWS_PROFILE", context)
        .env("SLICK_PROMPT_SHORT_PATH", "1")
        .env("SLICK_PROMPT_GIT_BRANCH_SYMBOL", "")
        .env("SLICK_PROMPT_NO_GIT_UNAME", "0");
    let direct_prompt = successful_stdout(&mut direct_command);
    assert!(
        direct_prompt.contains(r"branch-\$(id)-%%n-\`id\`-\\literal"),
        "unknown direct callers should receive safe backslash encoding: {direct_prompt:?}"
    );
    let direct_rendered = render_with_zsh(&direct_prompt, "on");
    assert!(direct_rendered.contains(branch));
    assert!(
        !direct_rendered.contains("uid="),
        "direct caller command substitution executed: {direct_rendered:?}"
    );
}
