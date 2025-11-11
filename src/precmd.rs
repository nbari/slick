use crate::get_env;
use git2::{DiffOptions, Error, ObjectType, Repository, StatusOptions, StatusShow};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    fmt::Write as _,
    fs,
    io::{self, Write},
    path::PathBuf,
    str,
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{spawn, task::spawn_blocking, time::timeout};

// Helper function to get current unix timestamp
// Returns 0 if system time is before UNIX_EPOCH (extremely rare)
fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// Git action constants to avoid repeated string allocations
const ACTION_REBASE: &str = "rebase";
const ACTION_AM: &str = "am";
const ACTION_AM_REBASE: &str = "am/rebase";
const ACTION_REBASE_I: &str = "rebase-i";
const ACTION_REBASE_M: &str = "rebase-m";
const ACTION_MERGE: &str = "merge";
const ACTION_BISECT: &str = "bisect";
const ACTION_CHERRY_SEQ: &str = "cherry-seq";
const ACTION_CHERRY: &str = "cherry";
const ACTION_CHERRY_OR_REVERT: &str = "cherry-or-revert";
const NO_BRANCH: &str = "(no branch)";

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct Prompt {
    action: String,
    branch: String,
    remote: Vec<String>,
    staged: bool,
    status: String,
    u_name: String,
    auth_failed: bool,
}

pub async fn render() {
    // Check if we're in a git repository
    let repo_result = env::current_dir()
        .ok()
        .and_then(|path| Repository::discover(path).ok());

    if let Some(repo) = repo_result {
        // Inside git repo: Output git info in 2 phases
        // Phase 1: Output all fast/local git info immediately (no blocking)
        let mut prompt = build_prompt_fast(&repo);

        if let Ok(serialized) = serde_json::to_string(&prompt) {
            // Ignore broken pipe errors (happens when zsh closes the pipe early)
            let _ = writeln!(io::stdout(), "{serialized}");
            // Force flush to ensure immediate output before Phase 2 starts
            let _ = io::stdout().flush();
        }

        // Phase 2a: Spawn blocking task for slow git status (CPU-bound)
        let repo_path = repo.path().to_path_buf();
        let status_handle = spawn_blocking(move || -> Option<String> {
            // TEST: Simulate slow git status (for testing non-blocking behavior)
            // Set SLICK_TEST_DELAY=N to add N seconds delay (e.g., SLICK_TEST_DELAY=1)
            // Note: Using thread::sleep here (not tokio::time::sleep) because spawn_blocking
            // runs in a blocking thread pool where synchronous sleep is appropriate
            if let Ok(delay_str) = env::var("SLICK_TEST_DELAY")
                && let Ok(delay_secs) = delay_str.parse::<u64>()
                && delay_secs > 0
            {
                sleep(Duration::from_secs(delay_secs));
            }

            // Re-open repository in the blocking thread pool
            if let Ok(repo) = Repository::open(&repo_path)
                && let Ok(status) = get_status(&repo)
            {
                return Some(status);
            }
            None
        });

        // Phase 2b: Async git fetch with auth detection and cache update
        // This spawns a tokio task that checks auth status and updates cache
        let fetch_handle = if get_env("SLICK_PROMPT_GIT_FETCH") != "0" {
            let cache_path = get_auth_cache_path(&repo);

            Some(spawn(async move {
                // Create cache directory if cache path exists
                if let Some(ref cache) = cache_path
                    && let Some(parent) = cache.parent()
                {
                    let _ = fs::create_dir_all(parent);
                }

                let mut cmd = tokio::process::Command::new("git");
                cmd.env("GIT_TERMINAL_PROMPT", "0")
                    .env(
                        "GIT_SSH_COMMAND",
                        "ssh -o BatchMode=yes -o ControlMaster=no",
                    )
                    .env("GIT_ASKPASS", "true")
                    .arg("-c")
                    .arg("gc.auto=0")
                    .arg("fetch")
                    .arg("--quiet")
                    .arg("--no-tags")
                    .arg("--no-recurse-submodules");

                // Use tokio timeout for 5 second limit
                let result = timeout(Duration::from_secs(5), cmd.output()).await;

                // Write auth status to cache if we have a cache path
                if let Some(cache) = cache_path {
                    let now = unix_timestamp();

                    let auth_failed = match result {
                        Ok(Ok(output)) => {
                            if output.status.success() {
                                false
                            } else {
                                // Check stderr for auth-related errors (convert to lowercase once)
                                let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
                                stderr.contains("permission denied")
                                    || stderr.contains("authentication failed")
                                    || stderr.contains("could not read")
                                    || stderr.contains("repository not found")
                                    || stderr.contains("access denied")
                            }
                        }
                        Ok(Err(_)) => {
                            // Command spawn/execution error - don't update cache
                            // This could be a transient issue, keep existing cache value
                            return;
                        }
                        Err(_) => {
                            // Timeout - don't update cache
                            // We couldn't complete the check, keep existing cache value
                            return;
                        }
                    };

                    let status = if auth_failed { "1" } else { "0" };
                    let _ = fs::write(cache, format!("{}:{}", now, status));
                }
            }))
        } else {
            None
        };

        // Wait for git status (fast ~10-50ms), output immediately
        if let Some(status) = status_handle.await.ok().flatten() {
            prompt.status = status;
            if let Ok(serialized) = serde_json::to_string(&prompt) {
                let _ = writeln!(io::stdout(), "{serialized}");
                let _ = io::stdout().flush();
            }
        }

        // Give fetch task a grace period to complete and write cache
        // Balance between fast prompt and cache write completion
        if let Some(handle) = fetch_handle {
            // Wait up to 500ms for fetch task to complete and write cache
            // Most fetches complete within this time, even with auth failures
            // Slower fetches will update cache on subsequent prompts (eventual consistency)
            let _ = timeout(Duration::from_millis(500), handle).await;
        }
    } else {
        // Outside git repo: Output empty prompt data (ensures handler fires for elapsed time)
        let prompt = Prompt::default();
        if let Ok(serialized) = serde_json::to_string(&prompt) {
            let _ = writeln!(io::stdout(), "{serialized}");
            let _ = io::stdout().flush();
        }
    }
}

// Build fast/synchronous prompt with all local git info (no slow operations)
// This includes: branch, user.name, remote ahead/behind, action, staged, auth status
// Excludes: git status (slow on large repos - moved to phase 2)
fn build_prompt_fast(repo: &Repository) -> Prompt {
    let mut prompt = Prompt::default();

    // get branch (instant - just reading HEAD)
    if let Ok(head) = repo.head() {
        prompt.branch = head.shorthand().unwrap_or(NO_BRANCH).to_string();
    } else {
        prompt.branch = NO_BRANCH.into();
    }

    // get user.name (fast - just reading git config)
    if let Ok(config) = repo.config() {
        prompt.u_name = config
            .get_string("user.name")
            .unwrap_or_else(|_| String::new());
    }

    // Check for cached auth status (synchronous, fast - just reads cache file)
    prompt.auth_failed = read_auth_status(repo);

    // git remote ahead/behind (fast - local graph traversal)
    let (ahead, behind) = is_ahead_behind_remote(repo);
    if behind > 0 {
        let mut s = String::with_capacity(8);
        // Writing to String never fails - ignore result
        let _ = write!(s, "{}{}", get_env("SLICK_PROMPT_GIT_REMOTE_BEHIND"), behind);
        prompt.remote.push(s);
    }
    if ahead > 0 {
        let mut s = String::with_capacity(8);
        // Writing to String never fails - ignore result
        let _ = write!(s, "{}{}", get_env("SLICK_PROMPT_GIT_REMOTE_AHEAD"), ahead);
        prompt.remote.push(s);
    }

    // git action (instant - file existence checks)
    if let Some(action) = get_action(repo) {
        prompt.action = action;
    }

    // git staged (fast - index diff)
    if let Ok(staged) = is_staged(repo) {
        prompt.staged = staged;
    }

    prompt
}

fn get_status(repo: &Repository) -> Result<String, Error> {
    // Pre-allocate with estimated capacity for common status types
    let mut status: Vec<String> = Vec::with_capacity(8);
    let mut status_opt = StatusOptions::new();
    status_opt
        .show(StatusShow::IndexAndWorkdir)
        .include_untracked(true)
        .include_unmodified(false)
        .no_refresh(false); // Keep false to get real-time status

    let statuses = repo.statuses(Some(&mut status_opt))?;
    if !statuses.is_empty() {
        // Use HashMap for O(1) operations instead of BTreeMap's O(log n)
        let mut map: HashMap<&str, u32> = HashMap::new();
        for entry in statuses.iter() {
            // println!("{:#?}, {:#?}", entry.path(), entry.status());
            let status = match entry.status() {
                s if s.contains(git2::Status::INDEX_NEW)
                    && s.contains(git2::Status::WT_MODIFIED) =>
                {
                    "AM"
                }
                s if s.contains(git2::Status::INDEX_MODIFIED)
                    && s.contains(git2::Status::WT_MODIFIED) =>
                {
                    "MM"
                }
                s if s.contains(git2::Status::INDEX_MODIFIED)
                    || s.contains(git2::Status::WT_MODIFIED) =>
                {
                    "M"
                }
                s if s.contains(git2::Status::INDEX_DELETED)
                    || s.contains(git2::Status::WT_DELETED) =>
                {
                    "D"
                }
                s if s.contains(git2::Status::INDEX_RENAMED)
                    || s.contains(git2::Status::WT_RENAMED) =>
                {
                    "R"
                }
                s if s.contains(git2::Status::INDEX_TYPECHANGE)
                    || s.contains(git2::Status::WT_TYPECHANGE) =>
                {
                    "T"
                }
                s if s.contains(git2::Status::INDEX_NEW) => "A",
                s if s.contains(git2::Status::WT_NEW) => "??",
                s if s.contains(git2::Status::CONFLICTED) => "UU",
                s if s.contains(git2::Status::IGNORED) => "!",
                _ => "X",
            };

            *map.entry(status).or_insert(0) += 1;
        }
        for (k, v) in &map {
            let mut s = String::with_capacity(8);
            // Writing to String never fails - ignore result
            let _ = write!(s, "{k} {v}");
            status.push(s);
        }
    }
    Ok(status.join(" "))
}

fn get_auth_cache_path(repo: &Repository) -> Option<PathBuf> {
    // Use workdir (repo root) instead of .git path for stable cache key
    // Canonicalize to get absolute path and resolve symlinks
    let repo_path = repo
        .workdir()
        .and_then(|p| p.canonicalize().ok())?
        .to_str()?
        .to_string();

    let cache_dir = env::var("XDG_CACHE_HOME")
        .or_else(|_| env::var("HOME").map(|h| format!("{h}/.cache")))
        .ok()?;

    // Create a hash of the canonicalized repo path for the cache filename
    let hash = repo_path.bytes().fold(0u64, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(u64::from(b))
    });

    let cache_path = PathBuf::from(cache_dir).join("slick");
    Some(cache_path.join(format!("auth_{hash:x}")))
}

fn read_auth_status(repo: &Repository) -> bool {
    if let Some(cache_path) = get_auth_cache_path(repo)
        && let Ok(content) = fs::read_to_string(&cache_path)
        && let Some((ts_str, status)) = content.split_once(':')
        && let Ok(cached_time) = ts_str.parse::<u64>()
    {
        let now = unix_timestamp();

        // Cache valid for 5 minutes
        if now - cached_time < 300 {
            return status.trim() == "1";
        }
    }
    false
}

fn get_action(repo: &Repository) -> Option<String> {
    let gitdir = repo.path();

    for tmp in &[
        gitdir.join("rebase-apply"),
        gitdir.join("rebase"),
        gitdir.join("..").join(".dotest"),
    ] {
        if tmp.join("rebasing").exists() {
            return Some(ACTION_REBASE.to_string());
        }
        if tmp.join("applying").exists() {
            return Some(ACTION_AM.to_string());
        }
        if tmp.exists() {
            return Some(ACTION_AM_REBASE.to_string());
        }
    }

    for tmp in &[
        gitdir.join("rebase-merge").join("interactive"),
        gitdir.join(".dotest-merge").join("interactive"),
    ] {
        if tmp.exists() {
            return Some(ACTION_REBASE_I.to_string());
        }
    }

    for tmp in &[gitdir.join("rebase-merge"), gitdir.join(".dotest-merge")] {
        if tmp.exists() {
            return Some(ACTION_REBASE_M.to_string());
        }
    }

    if gitdir.join("MERGE_HEAD").exists() {
        return Some(ACTION_MERGE.to_string());
    }

    if gitdir.join("BISECT_LOG").exists() {
        return Some(ACTION_BISECT.to_string());
    }

    if gitdir.join("CHERRY_PICK_HEAD").exists() {
        if gitdir.join("sequencer").exists() {
            return Some(ACTION_CHERRY_SEQ.to_string());
        }
        return Some(ACTION_CHERRY.to_string());
    }

    if gitdir.join("sequencer").exists() {
        return Some(ACTION_CHERRY_OR_REVERT.to_string());
    }

    None
}

fn is_ahead_behind_remote(repo: &Repository) -> (usize, usize) {
    if let Ok(head) = repo.revparse_single("HEAD") {
        let head = head.id();
        if let Ok((upstream, _)) = repo.revparse_ext("@{u}") {
            return match repo.graph_ahead_behind(head, upstream.id()) {
                Ok((commits_ahead, commits_behind)) => (commits_ahead, commits_behind),
                Err(_) => (0, 0),
            };
        }
    }
    (0, 0)
}

fn is_staged(repo: &Repository) -> Result<bool, Error> {
    let mut opts = DiffOptions::new();
    let obj = repo.head()?;
    let tree = obj.peel(ObjectType::Tree)?;
    let diff = repo.diff_tree_to_index(tree.as_tree(), None, Some(&mut opts))?;
    let stats = diff.stats()?;
    if stats.files_changed() > 0 || stats.insertions() > 0 || stats.deletions() > 0 {
        return Ok(true);
    }
    Ok(false)
    /*
     *  if ! git diff --cached --quiet; then echo staged; fi
     *
     * let format = git2::DiffStatsFormat::NUMBER;
     * let buf = stats.to_buf(format, 80).unwrap();
     * println!("diff: {}", str::from_utf8(&*buf).unwrap());
     */
}
