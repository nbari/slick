//! Tests for Toolbx prompt marker rendering.

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use std::{fs, process::Command};
use tempfile::tempdir;

fn get_slick_binary() -> String {
    env!("CARGO_BIN_EXE_slick").to_string()
}

fn write_toolbox_metadata() -> (tempfile::TempDir, String, String) {
    let tempdir = tempdir().expect("tempdir should be created");
    let toolboxenv_path = tempdir.path().join(".toolboxenv");
    let containerenv_path = tempdir.path().join(".containerenv");

    fs::write(&toolboxenv_path, "").expect("toolboxenv should be written");
    fs::write(
        &containerenv_path,
        "engine=\"podman\"\nname=\"codex\"\nid=\"abc\"\n",
    )
    .expect("containerenv should be written");

    (
        tempdir,
        toolboxenv_path.display().to_string(),
        containerenv_path.display().to_string(),
    )
}

#[test]
fn test_toolbox_marker_renders_with_default_symbol_before_path() {
    let (_tempdir, toolboxenv_path, containerenv_path) = write_toolbox_metadata();

    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("SLICK_TEST_TOOLBOXENV_PATH", &toolboxenv_path)
        .env("SLICK_TEST_CONTAINERENV_PATH", &containerenv_path)
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let marker_index = stdout
        .find("(🧰 codex)")
        .expect("toolbox marker should be present");
    let path_index = stdout.find("%~").expect("path marker should be present");

    assert!(marker_index < path_index);
}

#[test]
fn test_toolbox_marker_uses_custom_symbol_and_color() {
    let (_tempdir, toolboxenv_path, containerenv_path) = write_toolbox_metadata();

    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("SLICK_PROMPT_TOOLBOX_SYMBOL", "📦")
        .env("SLICK_PROMPT_TOOLBOX_COLOR", "42")
        .env("SLICK_TEST_TOOLBOXENV_PATH", &toolboxenv_path)
        .env("SLICK_TEST_CONTAINERENV_PATH", &containerenv_path)
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("%F{42}(📦 codex) "));
}

#[test]
fn test_toolbox_marker_is_absent_without_toolbox_metadata() {
    let tempdir = tempdir().expect("tempdir should be created");
    let missing_toolboxenv = tempdir.path().join(".missing-toolboxenv");
    let missing_containerenv = tempdir.path().join(".missing-containerenv");

    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("SLICK_PROMPT_TOOLBOX_SYMBOL", "📦")
        .env("SLICK_TEST_TOOLBOXENV_PATH", missing_toolboxenv)
        .env("SLICK_TEST_CONTAINERENV_PATH", missing_containerenv)
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("📦"));
    assert!(!stdout.contains("codex"));
}

#[test]
fn test_toolbox_marker_precedes_virtual_env_marker() {
    let (_tempdir, toolboxenv_path, containerenv_path) = write_toolbox_metadata();

    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("VIRTUAL_ENV", "/tmp/venvs/project")
        .env("SLICK_TEST_TOOLBOXENV_PATH", &toolboxenv_path)
        .env("SLICK_TEST_CONTAINERENV_PATH", &containerenv_path)
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let toolbox_index = stdout
        .find("(🧰 codex)")
        .expect("toolbox marker should be present");
    let venv_index = stdout
        .find("(project)")
        .expect("virtualenv marker should be present");

    assert!(toolbox_index < venv_index);
}

#[test]
fn test_devpod_marker_renders_before_path() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("DEVPOD", "true")
        .env("DEVPOD_WORKSPACE_ID", "hfile")
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let marker_index = stdout
        .find("(hfile)")
        .expect("devpod marker should be present");
    let path_index = stdout.find("%~").expect("path marker should be present");

    assert!(marker_index < path_index);
}

#[test]
fn test_devpod_marker_uses_custom_symbol_and_color() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("DEVPOD", "true")
        .env("DEVPOD_WORKSPACE_ID", "hfile")
        .env("SLICK_PROMPT_DEVPOD_SYMBOL", "🧪")
        .env("SLICK_PROMPT_DEVPOD_COLOR", "42")
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("%F{42}(🧪 hfile) "));
}

#[test]
fn test_devpod_marker_precedes_virtual_env_marker() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("DEVPOD", "true")
        .env("DEVPOD_WORKSPACE_ID", "hfile")
        .env("VIRTUAL_ENV", "/tmp/venvs/project")
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let devpod_index = stdout
        .find("(hfile)")
        .expect("devpod marker should be present");
    let venv_index = stdout
        .find("(project)")
        .expect("virtualenv marker should be present");

    assert!(devpod_index < venv_index);
}

#[test]
fn test_virtualenv_marker_uses_configured_color() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("VIRTUAL_ENV", "/tmp/venvs/project")
        .env("SLICK_PROMPT_PYTHON_ENV_COLOR", "42")
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("%F{42}(project) "));
}

#[test]
fn test_pyenv_marker_renders_with_python_env_color() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("PYENV_VERSION", "3.12.1/envs/project")
        .env("SLICK_PROMPT_PYTHON_ENV_COLOR", "99")
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("%F{99}(project) "));
}

#[test]
fn test_pipenv_marker_keeps_internal_hyphens() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("PIPENV_ACTIVE", "1")
        .env("VIRTUAL_ENV", "/tmp/venvs/my-app-a1b2c3d4")
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("(my-app) "));
    assert!(!stdout.contains("(my) "));
}

#[test]
fn test_python_env_color_falls_back_to_legacy_pipenv_color() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("PIPENV_ACTIVE", "1")
        .env("PIPENV_ACTIVE_COLOR", "88")
        .env("VIRTUAL_ENV", "/tmp/venvs/project-a1b2c3d4")
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("%F{88}(project) "));
}

#[test]
fn test_pyenv_system_only_marker_is_suppressed() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("PYENV_VERSION", "system")
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("(system) "));
}

#[test]
fn test_pyenv_marker_uses_first_real_entry() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("PYENV_VERSION", "system:3.12.1/envs/project")
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("(project) "));
    assert!(!stdout.contains("(system) "));
}

#[test]
fn test_pyenv_color_does_not_fall_back_to_legacy_pipenv_color() {
    let output = Command::new(get_slick_binary())
        .args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", ""])
        .env("PYENV_VERSION", "3.12.1/envs/project")
        .env("PIPENV_ACTIVE_COLOR", "88")
        .output()
        .expect("Failed to execute slick");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("%F{7}(project) "));
    assert!(!stdout.contains("%F{88}(project) "));
}
