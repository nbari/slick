use clap::{App, SubCommand, Arg};

pub fn display() {
    print!("$ ");
}

pub fn arguments<'a>() ->App<'a, 'a> {
    SubCommand::with_name("prompt")
        .arg(
            Arg::with_name("prompt")
            .long("return prompt")
            .help("Create PROMPT")
        )
}
