use clap::ArgMatches;
use git2::{self, Repository, StatusOptions};
use std::env;

fn repo_status(r: &Repository) -> Option<String> {
    let mut res = String::new();
    res.push_str("hello");
    Some(res)
}

pub fn display(_sub_matches: &ArgMatches) {
    let path = env::current_dir().unwrap();

    let branch = match Repository::discover(path) {
        Ok(repo) => repo_status(&repo),
        Err(_e) => None,
    };

    println!("{:?}", branch);
}
