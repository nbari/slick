//! Regression guard for the post-fetch ahead/behind refresh (`precmd` phase 3).
//!
//! History: the 0.14.1 two-phase refactor moved the ahead/behind calculation into
//! phase 1, which runs *before* the background `git fetch`. Nothing recomputed the
//! counts afterwards, so `cd`-ing into a repository showed remote-tracking data from
//! the previous fetch: you were never told you had commits to pull. These tests drive
//! the real `slick precmd` binary against a real local remote and assert that the
//! final emitted phase reflects the state *after* the fetch.

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::panic)]

use serde_json::Value;
use std::{
    path::Path,
    process::{Command, Stdio},
};
use tempfile::TempDir;

/// Sentinels keep assertions independent of the developer's own
/// `SLICK_PROMPT_GIT_REMOTE_*` settings leaking in from the environment.
const AHEAD: &str = "A";
const BEHIND: &str = "B";

fn git(dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(dir)
        .args(["-c", "user.name=slick test"])
        .args(["-c", "user.email=test@example.com"])
        .args(["-c", "commit.gpgsign=false"])
        .args(["-c", "init.defaultBranch=main"])
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .unwrap_or_else(|error| panic!("failed to run git {args:?}: {error}"));

    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_is_available() -> bool {
    Command::new("git")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Builds a repository that is 1 commit ahead and 2 commits behind its upstream,
/// **without** fetching, so the local remote-tracking ref is deliberately stale.
///
/// Returns the tempdir guard and the path of the working clone.
fn repo_ahead_1_behind_2() -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    let work = root.join("work");
    let other = root.join("other");

    git(root, &["init", "--quiet", "--bare", "remote.git"]);

    // Working clone with one pushed commit.
    git(root, &["init", "--quiet", "work"]);
    std::fs::write(work.join("a.txt"), "a\n").unwrap();
    git(&work, &["add", "a.txt"]);
    git(&work, &["commit", "--quiet", "-m", "initial"]);
    git(&work, &["remote", "add", "origin", "../remote.git"]);
    git(
        &work,
        &["push", "--quiet", "-u", "origin", "HEAD:refs/heads/main"],
    );

    // A second clone pushes two commits, so the remote moves ahead of `work`.
    git(root, &["clone", "--quiet", "remote.git", "other"]);
    for message in ["remote-1", "remote-2"] {
        std::fs::write(other.join("a.txt"), format!("{message}\n")).unwrap();
        git(&other, &["commit", "--quiet", "-am", message]);
    }
    git(&other, &["push", "--quiet"]);

    // `work` adds one local commit and never fetches: 1 ahead, 2 behind.
    std::fs::write(work.join("z.txt"), "z\n").unwrap();
    git(&work, &["add", "z.txt"]);
    git(&work, &["commit", "--quiet", "-m", "local-1"]);

    (dir, work)
}

/// Runs `slick precmd` in `work` and returns the `remote` array of every emitted phase.
fn precmd_phases(work: &Path, cache_dir: &Path, fetch: &str) -> Vec<Vec<String>> {
    let output = Command::new(env!("CARGO_BIN_EXE_slick"))
        .arg("precmd")
        .current_dir(work)
        .env("SLICK_PROMPT_GIT_REMOTE_AHEAD", AHEAD)
        .env("SLICK_PROMPT_GIT_REMOTE_BEHIND", BEHIND)
        .env("SLICK_PROMPT_GIT_FETCH", fetch)
        // Generous so a slow CI runner never turns this into a flaky timeout.
        .env("SLICK_PROMPT_GIT_FETCH_TIMEOUT", "60")
        .env("SLICK_TEST_AUTH_CACHE_DIR", cache_dir)
        .output()
        .expect("slick precmd should run");

    assert!(output.status.success(), "slick precmd should succeed");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let value: Value = serde_json::from_str(line)
                .unwrap_or_else(|error| panic!("bad JSON {line}: {error}"));
            value["remote"]
                .as_array()
                .expect("remote should be an array")
                .iter()
                .map(|entry| entry.as_str().unwrap_or_default().to_string())
                .collect()
        })
        .collect()
}

/// The regression itself: the last phase must report the commits the fetch just
/// discovered. Before the fix this emitted `["A1"]` forever - no behind count.
#[test]
fn test_precmd_reports_behind_commits_discovered_by_its_own_fetch() {
    if !git_is_available() {
        return;
    }

    let (_dir, work) = repo_ahead_1_behind_2();
    let cache = TempDir::new().unwrap();
    let phases = precmd_phases(&work, cache.path(), "1");

    assert!(
        phases.len() >= 2,
        "precmd should emit at least the fast and status phases, got {phases:?}"
    );

    // Phase 1 only knows the stale local refs: ahead, but not yet behind.
    assert_eq!(
        phases[0],
        vec![format!("{AHEAD}1")],
        "phase 1 should show the pre-fetch state"
    );

    // The final phase must include BOTH counts, ordered behind-then-ahead.
    let last = phases.last().expect("at least one phase");
    assert_eq!(
        *last,
        vec![format!("{BEHIND}2"), format!("{AHEAD}1")],
        "final phase must report 2 commits to pull and 1 to push after the fetch"
    );
}

/// The refresh must not spam the shell with a redundant redraw when the fetch
/// changed nothing, otherwise every prompt in an up-to-date repo repaints twice.
#[test]
fn test_precmd_does_not_re_emit_when_fetch_changes_nothing() {
    if !git_is_available() {
        return;
    }

    let (_dir, work) = repo_ahead_1_behind_2();
    let cache = TempDir::new().unwrap();

    // First run fetches and settles the repository view.
    precmd_phases(&work, cache.path(), "1");
    git(&work, &["fetch", "--quiet"]);

    // Second run fetches again but finds nothing new, so no extra phase.
    let phases = precmd_phases(&work, cache.path(), "1");
    assert_eq!(
        phases.len(),
        2,
        "an unchanged fetch must not trigger a third phase, got {phases:?}"
    );
    assert_eq!(
        *phases.last().unwrap(),
        vec![format!("{BEHIND}2"), format!("{AHEAD}1")]
    );
}

/// `SLICK_PROMPT_GIT_FETCH=0` must keep opting out of all network access, which
/// also means no refresh phase.
#[test]
fn test_precmd_with_fetch_disabled_stays_on_local_refs() {
    if !git_is_available() {
        return;
    }

    let (_dir, work) = repo_ahead_1_behind_2();
    let cache = TempDir::new().unwrap();
    let phases = precmd_phases(&work, cache.path(), "0");

    assert_eq!(
        phases.len(),
        2,
        "fetch is disabled, so there is nothing to refresh, got {phases:?}"
    );
    for phase in &phases {
        assert_eq!(
            *phase,
            vec![format!("{AHEAD}1")],
            "without fetching, only the stale local ahead count is known"
        );
    }
}
