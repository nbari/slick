use clap::{ArgMatches};
use std::thread;
use std::time::Duration;

pub fn display(sub_matches: &ArgMatches) {
    thread::sleep(Duration::from_secs(0));
    println!(" - ")
}