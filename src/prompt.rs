use clap::{ArgMatches};

const COMMAND_KEYMAP:&str = "vicmd";
// const INSERT_SYMBOL:&str = "%F{yellow}>%f";
const INSERT_SYMBOL:&str = ">";
const PROMPT_COLOR:i32 = 074;
const PROMPT_SYMBOL:&str = "$";
const NO_ERROR:&str = "O";


pub fn display(sub_matches: &ArgMatches) {
    let keymap = sub_matches.value_of("keymap").unwrap_or("US");
    let last_return_code = sub_matches.value_of("last_return_code").unwrap_or("0");

    let symbol = match keymap {
        COMMAND_KEYMAP  => INSERT_SYMBOL,
        _ => PROMPT_SYMBOL,
    };

    let prompt_color = match (symbol, last_return_code) {
        (PROMPT_SYMBOL, _)  =>  PROMPT_COLOR,
        (_,NO_ERROR) => 5,
        _ =>9,
    };

    print!("%F{{{}}}%~\n{}%f ", prompt_color, symbol)
}
