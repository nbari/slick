use clap::{App, SubCommand, Arg};
use std::time::Duration;
use std::thread;

pub fn display(){
    thread::sleep(Duration::from_secs(0));
    println!("%F{{074}}%~%f")
}

pub fn arguments<'a>() ->App<'a, 'a> {
    SubCommand::with_name("precmd")
        .arg(
            Arg::with_name("precmd")
            .long("precmd")
            .help("precmd")
        )
}
