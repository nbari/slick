use clap::{App, AppSettings, Arg, SubCommand};

mod precmd;
mod prompt;

fn main() {
    let matches = App::new("slick")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name("precmd").about("precmd"))
        .subcommand(
            SubCommand::with_name("prompt")
                .about("prompt")
                .arg(
                    Arg::with_name("last_return_code")
                        .short("r")
                        .takes_value(true),
                )
                .arg(Arg::with_name("keymap").short("k").takes_value(true))
                .arg(Arg::with_name("data").short("d").takes_value(true)),
        )
        .get_matches();

    match matches.subcommand() {
        ("precmd", Some(_)) => precmd::render(),
        ("prompt", Some(sub_matches)) => prompt::display(sub_matches),
        _ => (),
    }
}
