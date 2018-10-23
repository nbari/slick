extern crate clap;

use clap::{App, AppSettings, SubCommand, Arg};

mod prompt;
mod precmd;

fn main() {
    let matches = App::new("slick")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name("precmd")
                    .about("precmd")
        )
        .subcommand(SubCommand::with_name("prompt")
                    .about("prompt")
                    .arg(
                        Arg::with_name("last_return_code")
                        .short("r")
                        .takes_value(true)
                    )
                    .arg(
                        Arg::with_name("keymap")
                        .short("k")
                        .takes_value(true)
                    )
        )
        .get_matches();

    match matches.subcommand() {
        ("precmd", Some(sub_matches)) => precmd::display(sub_matches),
        ("prompt", Some(sub_matches)) => prompt::display(sub_matches),
        _ => (),
    }
}