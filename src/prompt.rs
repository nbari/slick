use clap::{ArgMatches};

const COMMAND_KEYMAP:&str = "vicmd";
const NON_BREAKING_SPACE:&str = " ";
const NO_ERROR:&str = "0";
const PROMPT_COLOR:i32 = 74;
const PROMPT_SYMBOL_COLOR:i32 = 5;
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

    let prompt_symbol_color = match (symbol, last_return_code) {
        (PROMPT_VICMD_SYMBOL, _) => PROMPT_VICMD_COLOR,
        (_, NO_ERROR)            => PROMPT_SYMBOL_COLOR,
        _                        => PROMPT_ERROR_COLOR,
    };

    // print!("%F{{{}}}{}%f{}", prompt_color, symbol, NON_BREAKING_SPACE)
    print!("%F{{{}}}%~{}\n%F{{{}}}{}%f{}", PROMPT_COLOR, " [test sleep here]", prompt_symbol_color, symbol, NON_BREAKING_SPACE)
}
