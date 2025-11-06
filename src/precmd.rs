use crate::get_env;
use git2::{DiffOptions, Error, ObjectType, Repository, StatusOptions, StatusShow};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env, fs,
    path::PathBuf,
    str,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{process::Command, time::timeout};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Prompt {
    action: String,
    branch: String,
    remote: Vec<String>,
    staged: bool,
    status: String,
    u_name: String,
    auth_failed: bool,
}

pub fn render() {
    if let Ok(path) = env::current_dir()
        && let Ok(repo) = Repository::discover(path)
    {
        build_prompt(&repo);
    }
}

fn build_prompt(repo: &Repository) {
    let mut prompt = Prompt::default();

    // get user.name
    if let Ok(config) = repo.config() {
        prompt.u_name = config
            .get_string("user.name")
            .unwrap_or_else(|_| String::new());
    }

    // get branch
    if let Ok(head) = repo.head() {
        prompt.branch = head.shorthand().unwrap_or("(no branch)").to_string();
    } else {
        prompt.branch = "(no branch)".into();
    }

    // Check for cached auth status (synchronous, fast)
    prompt.auth_failed = read_auth_status(repo);

    // git fetch and auth check (truly async, fire-and-forget)
    if get_env("SLICK_PROMPT_GIT_FETCH") != "0" {
        let cache_path = get_auth_cache_path(repo);

        tokio::spawn(async move {
            // Create cache directory if cache path exists
            if let Some(ref cache) = cache_path
                && let Some(parent) = cache.parent()
            {
                let _ = fs::create_dir_all(parent);
            }

            let mut cmd = Command::new("git");
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
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let auth_failed = match result {
                    Ok(Ok(output)) => {
                        if output.status.success() {
                            false
                        } else {
                            // Check stderr for auth-related errors
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            stderr.to_lowercase().contains("permission denied")
                                || stderr.to_lowercase().contains("authentication failed")
                                || stderr.to_lowercase().contains("could not read")
                                || stderr.to_lowercase().contains("repository not found")
                                || stderr.to_lowercase().contains("access denied")
                        }
                    }
                    Ok(Err(_)) | Err(_) => true, // Command error or timeout
                };

                let status = if auth_failed { "1" } else { "0" };
                let _ = fs::write(cache, format!("{}:{}", now, status));
            }
        });
    }

    // git remote
    let (ahead, behind) = is_ahead_behind_remote(repo);
    if behind > 0 {
        prompt.remote.push(format!(
            "{}{}",
            get_env("SLICK_PROMPT_GIT_REMOTE_BEHIND"),
            behind
        ));
    }
    if ahead > 0 {
        prompt.remote.push(format!(
            "{}{}",
            get_env("SLICK_PROMPT_GIT_REMOTE_AHEAD"),
            ahead
        ));
    }

    // git action
    if let Some(action) = get_action(repo) {
        prompt.action = action;
    }

    // git status
    if let Ok(status) = get_status(repo) {
        prompt.status = status;
    }

    // git staged
    if let Ok(staged) = is_staged(repo) {
        prompt.staged = staged;
    }

    // return prompt
    if let Ok(serialized) = serde_json::to_string(&prompt) {
        println!("{serialized}");
    }
}

fn get_status(repo: &Repository) -> Result<String, Error> {
    let mut status: Vec<String> = Vec::new();
    let mut status_opt = StatusOptions::new();
    status_opt
        .show(StatusShow::IndexAndWorkdir)
        .include_untracked(true)
        .include_unmodified(false)
        .no_refresh(false); // Keep false to get real-time status

    let statuses = repo.statuses(Some(&mut status_opt))?;
    if !statuses.is_empty() {
        let mut map: BTreeMap<&str, u32> = BTreeMap::new();
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
            status.push(format!("{k} {v}"));
        }
    }
    Ok(status.join(" "))
}

fn get_auth_cache_path(repo: &Repository) -> Option<PathBuf> {
    let repo_path = repo.path().to_str()?;
    let cache_dir = env::var("XDG_CACHE_HOME")
        .or_else(|_| env::var("HOME").map(|h| format!("{}/.cache", h)))
        .ok()?;

    // Create a hash of the repo path for the cache filename
    let hash = repo_path
        .bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));

    let cache_path = PathBuf::from(cache_dir).join("slick");
    Some(cache_path.join(format!("auth_{:x}", hash)))
}

fn read_auth_status(repo: &Repository) -> bool {
    if let Some(cache_path) = get_auth_cache_path(repo)
        && let Ok(content) = fs::read_to_string(&cache_path)
        && let Some((ts_str, status)) = content.split_once(':')
        && let Ok(cached_time) = ts_str.parse::<u64>()
    {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

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
            return Some("rebase".to_string());
        }
        if tmp.join("applying").exists() {
            return Some("am".to_string());
        }
        if tmp.exists() {
            return Some("am/rebase".to_string());
        }
    }

    for tmp in &[
        gitdir.join("rebase-merge").join("interactive"),
        gitdir.join(".dotest-merge").join("interactive"),
    ] {
        if tmp.exists() {
            return Some("rebase-i".to_string());
        }
    }

    for tmp in &[gitdir.join("rebase-merge"), gitdir.join(".dotest-merge")] {
        if tmp.exists() {
            return Some("rebase-m".to_string());
        }
    }

    if gitdir.join("MERGE_HEAD").exists() {
        return Some("merge".to_string());
    }

    if gitdir.join("BISECT_LOG").exists() {
        return Some("bisect".to_string());
    }

    if gitdir.join("CHERRY_PICK_HEAD").exists() {
        if gitdir.join("sequencer").exists() {
            return Some("cherry-seq".to_string());
        }
        return Some("cherry".to_string());
    }

    if gitdir.join("sequencer").exists() {
        return Some("cherry-or-revert".to_string());
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
