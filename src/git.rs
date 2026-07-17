// src/git.rs
use crate::get_env; // Assuming get_env is in lib.rs or another common module
use git2::{DiffOptions, Error, ErrorCode, Repository, Status, StatusOptions, StatusShow};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::Write as _,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

/// Returns the current Unix timestamp (seconds since `UNIX_EPOCH`).
///
/// Returns 0 if the system time is somehow before `UNIX_EPOCH` (an extremely rare occurrence).
#[must_use]
pub fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Constants representing various Git actions or states.
/// These are used to provide short, descriptive labels for the current Git operation.
pub const ACTION_REBASE: &str = "rebase";
pub const ACTION_AM: &str = "am";
pub const ACTION_AM_REBASE: &str = "am/rebase";
pub const ACTION_REBASE_I: &str = "rebase-i";
pub const ACTION_REBASE_M: &str = "rebase-m";
pub const ACTION_MERGE: &str = "merge";
pub const ACTION_BISECT: &str = "bisect";
pub const ACTION_CHERRY_SEQ: &str = "cherry-seq";
pub const ACTION_CHERRY: &str = "cherry";
pub const ACTION_CHERRY_OR_REVERT: &str = "cherry-or-revert";
pub const NO_BRANCH: &str = "(no branch)";

#[derive(Default)]
struct StatusCounts {
    conflicted: u32,
    added_modified: u32,
    modified_modified: u32,
    modified: u32,
    deleted: u32,
    renamed: u32,
    typechanged: u32,
    added: u32,
    untracked: u32,
    ignored: u32,
    fallback: u32,
}

impl StatusCounts {
    const fn increment(&mut self, status: Status) {
        if status.contains(Status::CONFLICTED) {
            self.conflicted += 1;
        } else if status.contains(Status::INDEX_RENAMED) || status.contains(Status::WT_RENAMED) {
            self.renamed += 1;
        } else if status.contains(Status::INDEX_NEW) && status.contains(Status::WT_MODIFIED) {
            self.added_modified += 1;
        } else if status.contains(Status::INDEX_MODIFIED) && status.contains(Status::WT_MODIFIED) {
            self.modified_modified += 1;
        } else if status.contains(Status::INDEX_MODIFIED) || status.contains(Status::WT_MODIFIED) {
            self.modified += 1;
        } else if status.contains(Status::INDEX_DELETED) || status.contains(Status::WT_DELETED) {
            self.deleted += 1;
        } else if status.contains(Status::INDEX_TYPECHANGE)
            || status.contains(Status::WT_TYPECHANGE)
        {
            self.typechanged += 1;
        } else if status.contains(Status::INDEX_NEW) {
            self.added += 1;
        } else if status.contains(Status::WT_NEW) {
            self.untracked += 1;
        } else if status.contains(Status::IGNORED) {
            self.ignored += 1;
        } else {
            self.fallback += 1;
        }
    }
}

/// Represents the collected Git information for rendering the prompt.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Prompt {
    /// The current Git action (e.g., "rebase", "merge", "am/rebase").
    pub action: String,
    /// The current branch name (or "(no branch)" for detached HEAD).
    pub branch: String,
    /// Information about remote tracking branches (e.g., "⇡1", "⇣2").
    pub remote: Vec<String>,
    /// True if there are staged changes.
    pub staged: bool,
    /// A summary of the repository's status (e.g., "M 1", "?? 2").
    pub status: String,
    /// The Git user name from the repository configuration.
    pub u_name: String,
    /// True if the last `git fetch` resulted in an authentication failure.
    pub auth_failed: bool,
}

/// Builds a `Prompt` struct containing fast/synchronous local Git information.
///
/// This function gathers information that does not require slow operations like network access
/// or extensive filesystem traversal (e.g., `git status`).
///
/// The collected information includes:
/// - Current branch name (`branch`)
/// - Git user name (`u_name`)
/// - Cached authentication status (`auth_failed`)
/// - Remote ahead/behind status (`remote`) based on local graph traversal
/// - Current Git action (`action`)
/// - Presence of staged changes (`staged`)
///
/// # Arguments
///
/// * `repo` - A reference to the `git2::Repository` object.
///
/// # Returns
///
/// A `Prompt` struct populated with available fast Git information.
#[must_use]
pub fn build_prompt_fast(repo: &Repository) -> Prompt {
    // get branch (instant - just reading HEAD)
    let branch = match repo.head() {
        Ok(head) => head
            .shorthand()
            .map_or_else(|_| NO_BRANCH.to_owned(), str::to_owned),
        Err(error) if error.code() == ErrorCode::UnbornBranch => repo
            .find_reference("HEAD")
            .ok()
            .and_then(|head| {
                head.symbolic_target()
                    .ok()
                    .flatten()
                    .and_then(|target| target.strip_prefix("refs/heads/"))
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| NO_BRANCH.to_owned()),
        Err(_) => NO_BRANCH.to_owned(),
    };
    let mut prompt = Prompt {
        branch,
        ..Prompt::default()
    };

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

/// Returns a string summarizing the git status of the repository.
///
/// # Errors
///
/// This function will return a `git2::Error` if it fails to get the repository statuses.
pub fn get_status(repo: &Repository) -> Result<String, Error> {
    // Pre-allocate with estimated capacity for common status types
    let mut status: Vec<String> = Vec::with_capacity(8);
    let mut status_opt = StatusOptions::new();
    status_opt
        .show(StatusShow::IndexAndWorkdir)
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_unmodified(false)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true)
        .no_refresh(false); // Keep false to get real-time status

    let statuses = repo.statuses(Some(&mut status_opt))?;
    if !statuses.is_empty() {
        let mut counts = StatusCounts::default();
        for entry in statuses.iter() {
            counts.increment(entry.status());
        }

        for (label, count) in [
            ("UU", counts.conflicted),
            ("AM", counts.added_modified),
            ("MM", counts.modified_modified),
            ("M", counts.modified),
            ("D", counts.deleted),
            ("R", counts.renamed),
            ("T", counts.typechanged),
            ("A", counts.added),
            ("??", counts.untracked),
            ("!", counts.ignored),
            ("X", counts.fallback),
        ] {
            if count > 0 {
                status.push(format!("{label} {count}"));
            }
        }
    }
    Ok(status.join(" "))
}

/// Generates the path for the Git authentication cache file for a given repository.
///
/// The cache path is determined based on the repository's working directory,
/// hashed to create a unique filename within the system's cache directory
/// ( `XDG_CACHE_HOME` or ~/.cache/slick).
///
/// # Arguments
///
/// * `repo` - A reference to the `git2::Repository` object.
///
/// # Returns
///
/// An `Option<PathBuf>` containing the full path to the cache file if it can be determined,
/// otherwise `None`.
#[must_use]
pub fn get_auth_cache_path(repo: &Repository) -> Option<PathBuf> {
    // Use workdir (repo root) instead of .git path for stable cache key
    // Canonicalize to get absolute path and resolve symlinks
    let repo_path = repo
        .workdir()
        .and_then(|p| p.canonicalize().ok())?
        .to_str()?
        .to_string();

    let cache_dir = env::var("SLICK_TEST_AUTH_CACHE_DIR")
        .or_else(|_| env::var("XDG_CACHE_HOME"))
        .or_else(|_| env::var("HOME").map(|h| format!("{h}/.cache")))
        .ok()?;

    // Create a hash of the canonicalized repo path for the cache filename
    let hash = repo_path.bytes().fold(0u64, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(u64::from(b))
    });

    let cache_path = PathBuf::from(cache_dir).join("slick");
    Some(cache_path.join(format!("auth_{hash:x}")))
}

/// Reads the cached Git authentication status for a given repository.
///
/// The cache file stores a timestamp and a status (0 for success, 1 for failure).
/// The cache is considered valid for 5 minutes.
///
/// # Arguments
///
/// * `repo` - A reference to the `git2::Repository` object.
///
/// # Returns
///
/// `true` if authentication previously failed and the cache is still fresh, `false` otherwise.
#[must_use]
pub fn read_auth_status(repo: &Repository) -> bool {
    if let Some(cache_path) = get_auth_cache_path(repo)
        && let Ok(content) = fs::read_to_string(&cache_path)
        && let Some((ts_str, status)) = content.split_once(':')
        && let Ok(cached_time) = ts_str.parse::<u64>()
    {
        let now = unix_timestamp();

        // Cache valid for 5 minutes
        if now.saturating_sub(cached_time) < 300 {
            return status.trim() == "1";
        }
    }
    false
}

/// Determines the current Git action (e.g., rebase, merge, cherry-pick) by checking
/// for the presence of specific files in the `.git` directory.
///
/// This is a fast operation as it only involves filesystem checks.
///
/// # Arguments
///
/// * `repo` - A reference to the `git2::Repository` object.
///
/// # Returns
///
/// An `Option<String>` containing the name of the current Git action if one is detected,
/// otherwise `None`.
#[must_use]
pub fn get_action(repo: &Repository) -> Option<String> {
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

/// Determines how many commits the current HEAD is ahead or behind its upstream remote.
///
/// This is a fast operation as it involves local graph traversal.
///
/// # Arguments
///
/// * `repo` - A reference to the `git2::Repository` object.
///
/// # Returns
///
/// A tuple `(ahead, behind)` where:
/// - `ahead` is the number of commits HEAD is ahead of the upstream.
/// - `behind` is the number of commits HEAD is behind the upstream.
///
/// Returns `(0, 0)` if no upstream is configured or in case of an error.
#[must_use]
pub fn is_ahead_behind_remote(repo: &Repository) -> (usize, usize) {
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

/// Checks if there are any staged changes in the repository.
///
/// # Errors
///
/// This function will return a `git2::Error` if it fails to get the repository head or diff.
pub fn is_staged(repo: &Repository) -> Result<bool, Error> {
    let mut opts = DiffOptions::new();
    let tree = match repo.head() {
        Ok(head) => Some(head.peel_to_tree()?),
        Err(error) if error.code() == ErrorCode::UnbornBranch => None,
        Err(error) => return Err(error),
    };
    let diff = repo.diff_tree_to_index(tree.as_ref(), None, Some(&mut opts))?;
    let stats = diff.stats()?;
    if stats.files_changed() > 0 || stats.insertions() > 0 || stats.deletions() > 0 {
        return Ok(true);
    }
    Ok(false)
}
