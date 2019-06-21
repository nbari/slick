use crate::envs::get_env;
use git2::{DiffOptions, Error, ObjectType, Repository, StatusOptions, StatusShow};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{collections::BTreeMap, env, fmt::Write, process::Command, str, thread};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Prompt {
    action: String,
    branch: String,
    remote: String,
    staged: bool,
    status: String,
}

pub fn render() {
    if let Ok(path) = env::current_dir() {
        match Repository::discover(path) {
            Ok(repo) => build_prompt(&repo),
            Err(_) => {
                return;
            }
        }
    }
}

fn build_prompt(repo: &Repository) {
    let mut prompt = Prompt::default();

    // get branch
    if let Ok(head) = repo.head() {
        drop(write!(
            prompt.branch,
            "{}",
            head.shorthand().unwrap_or("(no branch)")
        ))
    } else {
        return;
    }

    // git fetch
    if get_env("SLICK_PROMPT_GIT_FETCH") != "0" {
        thread::spawn(move || {
            Command::new("git")
                .arg("-c")
                .arg("gc.auto=0")
                .arg("fetch")
                .output()
                .expect("failed to execute process");
        });
    }

    // git remote
    let (ahead, behind) = is_ahead_behind_remote(repo);
    if behind > 0 {
        drop(write!(prompt.remote, "⇣ {}", behind))
    }
    if ahead > 0 {
        drop(write!(prompt.remote, " ⇡ {}", ahead))
    }
    prompt.remote = prompt.remote.trim().to_string();

    // git action
    if let Some(action) = get_action(repo) {
        prompt.action = action;
    }

    // git status
    if let Ok(status) = get_status(repo) {
        prompt.status = status
    }

    // git staged
    if let Ok(staged) = is_staged(repo) {
        prompt.staged = staged
    }

    // return prompt
    if let Ok(serialized) = serde_json::to_string(&prompt) {
        println!("{}", serialized);
    }
}

fn get_status(repo: &Repository) -> Result<String, Error> {
    let mut status = String::new();
    let mut status_opt = StatusOptions::new();
    status_opt
        .show(StatusShow::IndexAndWorkdir)
        .include_untracked(true)
        .include_unmodified(false)
        .no_refresh(false);

    let statuses = repo.statuses(Some(&mut status_opt))?;
    if statuses.len() != 0 {
        let mut map: BTreeMap<&str, u32> = BTreeMap::new();
        for entry in statuses.iter() {
            // println!("{:?}", entry.status());
            let istatus = match entry.status() {
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

            *map.entry(istatus).or_insert(0) += 1;
        }
        for (k, v) in map.iter() {
            drop(write!(status, "{} {}, ", k, v))
        }
        let len = status.len();
        if len > 2 {
            status.truncate(len - 2);
        }
        return Ok(status);
    }
    Ok(status)
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
        } else {
            return Some("cherry".to_string());
        }
    }

    if gitdir.join("sequencer").exists() {
        return Some("cherry-or-revert".to_string());
    }

    None
}

fn is_ahead_behind_remote(repo: &Repository) -> (usize, usize) {
    if let Ok(head) = repo.revparse_single("HEAD") {
        let head = head.id();
        if let Some((upstream, _)) = repo.revparse_ext("@{u}").ok() {
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
