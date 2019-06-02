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

    let status_result = repo.statuses(Some(&mut status_opt)).unwrap();

    if status_result.len() == 0 {
        print!("{}", prompt);
        return;
    }

    //let mut map: HashMap<&str, u32> = HashMap::new();
    let mut map: HashMap<git2::Status, u32> = HashMap::new();

    for status_entry in status_result.iter() {
        println!(
            "branch: {} {:?} {:?}",
            prompt,
            status_entry.path(),
            status_entry.status()
        );
        *map.entry(status_entry.status()).or_insert(0) += 1;
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
