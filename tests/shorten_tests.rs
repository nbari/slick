//! Tests for prompt shortening options.

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn compact_path_for(path: &Path, home: &Path) -> String {
    if let Ok(relative) = path.strip_prefix(home) {
        let rendered = compact_segments(
            &relative
                .iter()
                .filter_map(|segment| segment.to_str())
                .filter(|segment| !segment.is_empty())
                .collect::<Vec<_>>(),
        );

        return if rendered.is_empty() {
            "~".to_string()
        } else {
            format!("~/{rendered}")
        };
    }

    let rendered = compact_segments(
        &path
            .iter()
            .filter_map(|segment| segment.to_str())
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>(),
    );
    format!("/{rendered}")
}

fn compact_segments(parts: &[&str]) -> String {
    let mut compacted = Vec::with_capacity(parts.len());
    for (index, part) in parts.iter().enumerate() {
        if index + 1 == parts.len() {
            compacted.push((*part).to_string());
        } else {
            compacted.push(
                part.chars()
                    .next()
                    .expect("segment should not be empty")
                    .to_string(),
            );
        }
    }
    compacted.join("/")
}

fn get_slick_binary() -> String {
    env!("CARGO_BIN_EXE_slick").to_string()
}

fn temp_workdir(home_relative: bool) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tempdir = tempfile::tempdir().expect("tempdir should be created");
    let home = tempdir.path().join("home");
    let workdir = if home_relative {
        home.join("projects").join("rust").join("slick")
    } else {
        tempdir.path().join("alpha").join("bravo").join("charlie")
    };

    fs::create_dir_all(&home).expect("home should be created");
    fs::create_dir_all(&workdir).expect("workdir should be created");

    (tempdir, home, workdir)
}

#[test]
fn test_short_path_option_full_prompt() {
    let (_tempdir, home, workdir) = temp_workdir(true);
    let expected_path = compact_path_for(&workdir, &home);

    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("SLICK_PROMPT_SHORT_PATH", "1")
        .env("HOME", &home)
        .current_dir(&workdir)
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&expected_path));
    assert!(!stdout.contains("%~"));
}

#[test]
fn test_short_path_option_transient_prompt() {
    let (_tempdir, home, workdir) = temp_workdir(true);
    let expected_path = compact_path_for(&workdir, &home);

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
        .env("HOME", &home)
        .current_dir(&workdir)
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&expected_path));
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
}

#[test]
fn test_short_path_preserves_full_context_markers() {
    let (_tempdir, home, workdir) = temp_workdir(true);
    let expected_path = compact_path_for(&workdir, &home);

    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("SLICK_PROMPT_SHORT_PATH", "1")
        .env("AWS_PROFILE", "production-high-availability")
        .env("HOME", &home)
        .current_dir(&workdir)
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("(aws production-high-availability)"));
    assert!(stdout.contains(&expected_path));
}

#[test]
fn test_short_context_shortens_markers_without_shortening_path() {
    let (_tempdir, home, workdir) = temp_workdir(true);

    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("SLICK_PROMPT_SHORT_CONTEXT", "1")
        .env("AWS_PROFILE", "production-high-availability")
        .env("HOME", &home)
        .current_dir(&workdir)
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("(aws)"));
    assert!(!stdout.contains("production-high-availability"));
    assert!(stdout.contains("%~"));
}

#[test]
fn test_short_path_outside_home_uses_absolute_compact_form() {
    let (_tempdir, home, workdir) = temp_workdir(false);

    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("SLICK_PROMPT_SHORT_PATH", "1")
        .env("HOME", &home)
        .current_dir(&workdir)
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("/a/b/charlie"));
    assert!(!stdout.contains("%~"));
}
