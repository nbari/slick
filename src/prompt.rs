use clap::{ArgMatches};

const COMMAND_KEYMAP:&str = "vicmd";
const NO_ERROR:&str = "0";
const PROMPT_COLOR:i32 = 5;
const PROMPT_ERROR_COLOR:i32 = 196;
const PROMPT_SYMBOL:&str = "$";
const PROMPT_VICMD_COLOR:i32 = 3;
const PROMPT_VICMD_SYMBOL:&str = ">";


pub fn display(sub_matches: &ArgMatches) {
    let keymap = sub_matches.value_of("keymap").unwrap_or("US");
    let last_return_code = sub_matches.value_of("last_return_code").unwrap_or("0");

    let symbol = match keymap {
        COMMAND_KEYMAP  => PROMPT_VICMD_SYMBOL,
        _ => PROMPT_SYMBOL,
    };

    let prompt_color = match (symbol, last_return_code) {
        (PROMPT_VICMD_SYMBOL, _) => PROMPT_VICMD_COLOR,
        (_, NO_ERROR)            => PROMPT_COLOR,
        _                        => PROMPT_ERROR_COLOR,
    };

    print!("%F{{{}}}{}%f ", prompt_color, symbol)
}
