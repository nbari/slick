use git2::{DiffOptions, ObjectType, Repository, StatusOptions, StatusShow};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{collections::BTreeMap, env, str};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Prompt {
    action: String,
    branch: String,
    remote: String,
    status: String,
    staged: bool,
}

pub fn render() {
    let path = env::current_dir().unwrap();
    match Repository::discover(path) {
        Ok(repo) => repo_status(&repo),
        Err(_) => {
            return;
        }
    }
}

fn repo_status(repo: &Repository) {
    let mut prompt = Prompt::default();

    prompt.staged = is_staged(repo);

    match repo.head() {
        Ok(head) => {
            prompt
                .branch
                .push_str(head.shorthand().unwrap_or("(no branch)"));
        }
        Err(_) => {
            return;
        }
    }

    match get_action(repo) {
        Some(action) => {
            prompt.action = action;
        }
        None => (),
    }

    let (ahead, behind) = is_ahead_behind_remote(repo);
    if behind > 0 {
        prompt.remote.push_str(format!("⇣ {}", behind).as_str());
    }
    if ahead > 0 {
        prompt.remote.push_str(format!(" ⇡ {}", ahead).as_str());
    }
    prompt.remote = prompt.remote.trim().to_string();

    let mut status_opt = StatusOptions::new();
    status_opt
        .show(StatusShow::IndexAndWorkdir)
        .include_untracked(true)
        .include_unmodified(false)
        .no_refresh(false);

    let statuses = repo.statuses(Some(&mut status_opt)).unwrap();
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
                _ => "X",
            };

            *map.entry(istatus).or_insert(0) += 1;
        }
        for (k, v) in map.iter() {
            prompt.status.push_str(format!("{} {}, ", k, v).as_str());
        }
        let len = prompt.status.len();
        if len > 2 {
            prompt.status.truncate(len - 2);
        }
    }

    let serialized = serde_json::to_string(&prompt).unwrap();
    println!("{}", serialized);
}

fn get_action(r: &Repository) -> Option<String> {
    let gitdir = r.path();

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
    let head = repo.revparse_single("HEAD").unwrap().id();
    if let Some((upstream, _)) = repo.revparse_ext("@{u}").ok() {
        return match repo.graph_ahead_behind(head, upstream.id()) {
            Ok((commits_ahead, commits_behind)) => (commits_ahead, commits_behind),
            Err(_) => (0, 0),
        };
    }
    (0, 0)
}

fn is_staged(repo: &Repository) -> bool {
    let mut opts = DiffOptions::new();
    opts.minimal(false);
    let obj = match repo.head() {
        Ok(obj) => obj,
        Err(_) => return false,
    };
    let tree = match obj.peel(ObjectType::Tree) {
        Ok(tree) => tree,
        Err(_) => return false,
    };
    let diff = match repo.diff_tree_to_index(tree.as_tree(), None, Some(&mut opts)) {
        Ok(d) => d,
        Err(_) => return false,
    };
    let stats = match diff.stats() {
        Ok(stats) => stats,
        Err(_) => return false,
    };
    if stats.files_changed() > 0 || stats.insertions() > 0 || stats.deletions() > 0 {
        return true;
    }
    return false;
    /*
     *  if ! git diff --cached --quiet; then echo staged; fi
     *
     * let format = git2::DiffStatsFormat::NUMBER;
     * let buf = stats.to_buf(format, 80).unwrap();
     * println!("diff: {}", str::from_utf8(&*buf).unwrap());
     */
}
