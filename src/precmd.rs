use git2::{Repository, StatusOptions, StatusShow};
use std::collections::HashMap;
use std::env;

fn repo_status(repo: &Repository) {
    let mut prompt = String::new();

    match repo.head() {
        Ok(head) => {
            prompt.push_str(head.shorthand().unwrap_or("(no branch)"));
        }
        Err(_) => {
            return;
        }
    }

    let mut status_opt = StatusOptions::new();

    status_opt
        .show(StatusShow::IndexAndWorkdir)
        .include_untracked(true)
        .include_unmodified(false)
        .no_refresh(false);

    let statuses = repo.statuses(Some(&mut status_opt)).unwrap();

    if statuses.len() == 0 {
        print!("{}", prompt);
        return;
    }

    let mut map: HashMap<&str, u32> = HashMap::new();
    for entry in statuses.iter() {
        println!(
            "branch: {} {:?} {:?}",
            prompt,
            entry.path(),
            entry.status()
        );

        let istatus = match entry.status() {
            s if s.contains(git2::Status::INDEX_NEW) && s.contains(git2::Status::WT_MODIFIED) => "AM",
            s if s.contains(git2::Status::INDEX_NEW) => "A",
            s if s.contains(git2::Status::WT_NEW) => "??",
            s if s.contains(git2::Status::INDEX_MODIFIED) || s.contains(git2::Status::WT_MODIFIED) => "M",
            s if s.contains(git2::Status::INDEX_DELETED) || s.contains(git2::Status::WT_DELETED) => "D",
            s if s.contains(git2::Status::INDEX_RENAMED) || s.contains(git2::Status::WT_RENAMED) => "R",
            s if s.contains(git2::Status::INDEX_TYPECHANGE) || s.contains(git2::Status::WT_TYPECHANGE) => "T",
            _ => ""
        };

        *map.entry(istatus).or_insert(0) += 1;
    }


    println!("{:?}", map);
}

pub fn display() {
    let path = env::current_dir().unwrap();
    match Repository::discover(path) {
        Ok(repo) => repo_status(&repo),
        Err(_) => {
            return;
        }
    }
}
