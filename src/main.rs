extern crate clap;

use clap::{App, AppSettings};

mod prompt;
mod precmd;

fn main() {
    let matches = App::new("slick")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::SubcommandRequired)
        .subcommand(precmd::arguments())
        .subcommand(prompt::arguments())
        .get_matches();

    match matches.subcommand() {
        ("precmd", Some(sub_matches)) => precmd::display(),
        ("prompt", Some(sub_matches)) => prompt::display(),
        _ => (),
    }

    //let p = prompt::display();
    //let c = precmd::display();
    //print!("{}\n{}", c, p);
}
