//! Tests for prompt shortening options (Issue #22).

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use std::process::Command;

fn get_slick_binary() -> String {
    env!("CARGO_BIN_EXE_slick").to_string()
}

#[test]
fn test_short_path_option_full_prompt() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("SLICK_PROMPT_SHORT_PATH", "1")
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // %c is the Zsh escape for trailing component of current working directory
    assert!(stdout.contains("%c"));
    assert!(!stdout.contains("%~"));
}

#[test]
fn test_short_path_option_transient_prompt() {
    let output = Command::new(get_slick_binary())
        .args([
            "prompt",
            "--transient",
            "-e",
            "0",
            "-r",
            "0",
            "-k",
            "main",
            "-d",
            "",
        ])
        .env("SLICK_PROMPT_SHORT_PATH", "1")
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("%c"));
    assert!(!stdout.contains("%~"));
}

#[test]
fn test_default_path_is_not_shortened() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("%~"));
    assert!(!stdout.contains("%c"));
}

#[test]
fn test_auto_short_context_is_off_by_default() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("AWS_PROFILE", "production-high-availability")
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("(aws production-high-availability)"));
}

#[test]
fn test_enable_auto_short_context() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("SLICK_PROMPT_AUTO_SHORT_CONTEXT", "1")
        .env("AWS_PROFILE", "production-high-availability")
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("(aws)"));
    assert!(!stdout.contains("production-high-availability"));
}
