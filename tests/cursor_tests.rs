//! Tests for cursor shape rendering in the prompt.

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use std::process::Command;

fn get_slick_binary() -> String {
    env!("CARGO_BIN_EXE_slick").to_string()
}

#[test]
fn test_default_cursor_shape_is_included() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Default SLICK_PROMPT_CURSOR_SHAPE is 4 (steady underscore)
    assert!(stdout.contains("%{\x1b[4 q%}"));
}

#[test]
fn test_custom_cursor_shape_is_included() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("SLICK_PROMPT_CURSOR_SHAPE", "6")
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("%{\x1b[6 q%}"));
}

#[test]
fn test_cursor_shape_is_consistent_in_vicmd() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "vicmd", "-d", ""])
        .env("SLICK_PROMPT_CURSOR_SHAPE", "4")
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should still be 4 even in vicmd
    assert!(stdout.contains("%{\x1b[4 q%}"));
}

#[test]
fn test_invalid_cursor_shape_is_ignored() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("SLICK_PROMPT_CURSOR_SHAPE", "9")
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("\x1b["));
}

#[test]
fn test_transient_prompt_includes_cursor_shape() {
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
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("%{\x1b[4 q%}"));
}
