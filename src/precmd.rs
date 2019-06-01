use clap::ArgMatches;
use git2::Repository;
use pty::fork::*;
use std::env;
use std::io::Read;
use std::process::Command;

fn repo_status(_r: &Repository) -> Option<String> {
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

    let fork = Fork::from_ptmx().unwrap();

    if let Some(mut master) = fork.is_parent().ok() {
        // Read output via PTY master
        let mut output = String::new();

        match master.read_to_string(&mut output) {
            Ok(_nread) => println!("child tty is: {} {:?}", output.trim(), branch),
            Err(e) => panic!("read error: {}", e),
        }
    } else {
        // Child process just exec `tty`
        Command::new("sleep")
            .arg("3")
            .status()
            .expect("could not execute tty");
    }
}
