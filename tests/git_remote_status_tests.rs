//! Tests for remote status markers and the post-fetch ahead/behind refresh.
//!
//! Regression: the ahead/behind counters used to be computed in phase 1, before the
//! background `git fetch` ran, and were never recomputed. `cd`-ing into a repository
//! therefore never showed that you were behind the remote until the *next* prompt.

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

mod common;

use slick::git::{self, FetchStatus};
use std::{fs, process::Command};
use tempfile::TempDir;

fn slick_binary() -> String {
    env!("CARGO_BIN_EXE_slick").to_string()
}

fn render_prompt(data: &str, envs: &[(&str, &str)]) -> String {
    let mut command = Command::new(slick_binary());
    command.args(["prompt", "-e", "0", "-r", "0", "-k", "main", "-d", data]);
    for (key, value) in envs {
        command.env(key, value);
    }

    let output = command.output().expect("slick prompt should run");
    assert!(output.status.success(), "slick prompt should succeed");
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn prompt_data(auth_failed: bool, fetch_failed: bool) -> String {
    format!(
        r#"{{"action":"","branch":"main","remote":[],"staged":false,"status":"","u_name":"","auth_failed":{auth_failed},"fetch_failed":{fetch_failed}}}"#
    )
}

/// Pin both markers to sentinels so the assertions do not depend on the
/// developer's own `SLICK_PROMPT_GIT_*_SYMBOL` settings leaking in from the env.
const MARKER_ENV: [(&str, &str); 2] = [
    ("SLICK_PROMPT_GIT_AUTH_SYMBOL", "LOCKED"),
    ("SLICK_PROMPT_GIT_OFFLINE_SYMBOL", "OFFLINE"),
];

#[test]
fn test_unreachable_remote_renders_offline_marker() {
    let stdout = render_prompt(&prompt_data(false, true), &MARKER_ENV);
    assert!(
        stdout.contains("OFFLINE"),
        "offline marker should be rendered, got: {stdout}"
    );
    assert!(
        !stdout.contains("LOCKED"),
        "lock marker should not be rendered for an unreachable remote"
    );
}

#[test]
fn test_auth_failure_takes_precedence_over_offline_marker() {
    let stdout = render_prompt(&prompt_data(true, true), &MARKER_ENV);
    assert!(
        stdout.contains("LOCKED"),
        "lock marker should be rendered, got: {stdout}"
    );
    assert!(
        !stdout.contains("OFFLINE"),
        "only one remote marker should be rendered at a time"
    );
}

#[test]
fn test_healthy_remote_renders_no_marker() {
    let stdout = render_prompt(&prompt_data(false, false), &MARKER_ENV);
    assert!(!stdout.contains("OFFLINE"), "got: {stdout}");
    assert!(!stdout.contains("LOCKED"), "got: {stdout}");
}

#[test]
fn test_offline_marker_uses_default_warning_symbol() {
    let stdout = render_prompt(
        &prompt_data(false, true),
        &[("SLICK_PROMPT_GIT_OFFLINE_SYMBOL", "\u{26a0}")],
    );
    assert!(stdout.contains('\u{26a0}'), "got: {stdout}");
}

#[test]
fn test_offline_marker_honors_custom_symbol_and_color() {
    let stdout = render_prompt(
        &prompt_data(false, true),
        &[
            ("SLICK_PROMPT_GIT_OFFLINE_SYMBOL", "OFFLINE"),
            ("SLICK_PROMPT_GIT_OFFLINE_COLOR", "202"),
        ],
    );
    assert!(stdout.contains("%F{202}OFFLINE"), "got: {stdout}");
}

#[test]
fn test_payload_without_fetch_failed_field_still_renders() {
    // precmd output produced by an older slick must not break the renderer.
    let legacy = r#"{"action":"","branch":"main","remote":["\u21e11"],"staged":false,"status":"","u_name":"","auth_failed":false}"#;
    let stdout = render_prompt(legacy, &MARKER_ENV);
    assert!(stdout.contains('\u{21e1}'), "got: {stdout}");
    assert!(!stdout.contains("OFFLINE"), "got: {stdout}");
}

#[test]
fn test_fetch_status_cache_round_trip() {
    for status in [
        FetchStatus::Ok,
        FetchStatus::AuthFailed,
        FetchStatus::Unreachable,
    ] {
        assert_eq!(
            FetchStatus::from_cache_value(status.as_cache_value()),
            status
        );
    }

    // Trailing whitespace from a cache file must not confuse parsing.
    assert_eq!(
        FetchStatus::from_cache_value("2\n"),
        FetchStatus::Unreachable
    );
    // A corrupt cache must never invent an error state.
    assert_eq!(FetchStatus::from_cache_value("garbage"), FetchStatus::Ok);
}

/// `SLICK_TEST_AUTH_CACHE_DIR` redirects the cache directory, and the env var is
/// global, so these tests serialise on a mutex.
static CACHE_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn with_test_cache_dir<F: FnOnce()>(f: F) {
    let _lock = CACHE_TEST_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let cache_dir = TempDir::new().unwrap();
    unsafe { std::env::set_var("SLICK_TEST_AUTH_CACHE_DIR", cache_dir.path()) };
    f();
    unsafe { std::env::remove_var("SLICK_TEST_AUTH_CACHE_DIR") };
}

#[test]
fn test_read_fetch_status_reports_unreachable_remote() {
    with_test_cache_dir(|| {
        let (_repo_dir, repo) = common::create_test_repo();
        let cache_path = git::get_auth_cache_path(&repo).unwrap();
        fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
        let now = git::unix_timestamp();
        fs::write(&cache_path, format!("{now}:2")).unwrap();

        assert_eq!(git::read_fetch_status(&repo), FetchStatus::Unreachable);
        assert!(
            !git::read_auth_status(&repo),
            "an unreachable remote is not an authentication failure"
        );
    });
}

#[test]
fn test_read_fetch_status_expires_with_the_cache() {
    with_test_cache_dir(|| {
        let (_repo_dir, repo) = common::create_test_repo();
        let cache_path = git::get_auth_cache_path(&repo).unwrap();
        fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
        let stale = git::unix_timestamp() - 301;
        fs::write(&cache_path, format!("{stale}:2")).unwrap();

        assert_eq!(git::read_fetch_status(&repo), FetchStatus::Ok);
    });
}

#[test]
fn test_remote_markers_are_empty_without_an_upstream() {
    let (_repo_dir, repo) = common::create_test_repo();
    common::create_commit(&repo, "initial");

    assert!(
        git::remote_markers(&repo).is_empty(),
        "a repository without an upstream has nothing to push or pull"
    );
}
