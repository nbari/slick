use clap::ArgMatches;
use compound_duration;
use serde::{Deserialize, Serialize};
use serde_json;
use std::time::{Duration, SystemTime};

const COMMAND_KEYMAP: &str = "vicmd";
const NON_BREAKING_SPACE: &str = " ";
const NO_ERROR: &str = "0";
const PROMPT_ERROR_COLOR: i32 = 196;
const PROMPT_PATH_COLOR: i32 = 74;
const PROMPT_SYMBOL: &str = "$";
const PROMPT_SYMBOL_COLOR: i32 = 5;
const PROMPT_VICMD_COLOR: i32 = 3;
const PROMPT_VICMD_SYMBOL: &str = ">";
const PROMPT_GIT_MASTER_BRANCH_COLOR: i32 = 160;
const PROMPT_GIT_BRANCH_COLOR: &str = "yellow";
const PROMPT_GIT_ACTION_COLOR: &str = "yellow";
const PROMPT_TIME_ELAPSED_COLOR: &str = "yellow";
const PROMPT_GIT_STATUS_COLOR: i32 = 5;
const PROMPT_GIT_REMOTE_COLOR: &str = "cyan";

#[derive(Serialize, Deserialize, Debug, Default)]
struct Prompt {
    action: String,
    branch: String,
    remote: String,
    status: String,
}

pub fn display(sub_matches: &ArgMatches) {
    let keymap = sub_matches.value_of("keymap").unwrap_or("main");
    let last_return_code = sub_matches.value_of("last_return_code").unwrap_or("0");
    let serialized = sub_matches.value_of("data").unwrap_or("");
    let deserialized: Prompt = match serde_json::from_str(&serialized) {
        Ok(ok) => ok,
        Err(_) => Prompt::default(),
    };

    // get time elapsed
    let epochtime: u64 = match sub_matches
        .value_of("time")
        .unwrap_or("")
        .parse::<u64>()
        .ok()
    {
        Some(v) => v,
        None => match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        },
    };
    let d = SystemTime::UNIX_EPOCH + Duration::from_secs(epochtime);
    let time_elapsed = match d.elapsed() {
        Ok(elapsed) => elapsed.as_secs() as usize,
        Err(_) => 0,
    };

    let symbol = match keymap {
        COMMAND_KEYMAP => PROMPT_VICMD_SYMBOL,
        _ => PROMPT_SYMBOL,
    };

    let prompt_symbol_color = match (symbol, last_return_code) {
        (PROMPT_VICMD_SYMBOL, _) => PROMPT_VICMD_COLOR,
        (_, NO_ERROR) => PROMPT_SYMBOL_COLOR,
        _ => PROMPT_ERROR_COLOR,
    };

    let mut prompt = String::new();
    prompt.push_str(format!("%F{{{}}}%~", PROMPT_PATH_COLOR).as_str());

    // branch
    if deserialized.branch == "master" {
        prompt.push_str(
            format!(
                " %F{{{}}}{}",
                PROMPT_GIT_MASTER_BRANCH_COLOR, deserialized.branch
            )
            .as_str(),
        );
    } else {
        prompt.push_str(
            format!(" %F{{{}}}{}", PROMPT_GIT_BRANCH_COLOR, deserialized.branch).as_str(),
        );
    }

    // git status
    if !deserialized.status.is_empty() {
        prompt.push_str(
            format!(
                " %F{{{}}}[{}]",
                PROMPT_GIT_STATUS_COLOR, deserialized.status
            )
            .as_str(),
        );
    }

    // git remote
    if !deserialized.remote.is_empty() {
        prompt.push_str(
            format!(" %F{{{}}}{}", PROMPT_GIT_REMOTE_COLOR, deserialized.remote).as_str(),
        );
    }

    // git action
    if !deserialized.action.is_empty() {
        prompt.push_str(
            format!(" %F{{{}}}{}", PROMPT_GIT_ACTION_COLOR, deserialized.action).as_str(),
        );
    }

    // time elapsed
    if time_elapsed > 3 {
        prompt.push_str(
            format!(
                " %F{{{}}}{}",
                PROMPT_TIME_ELAPSED_COLOR,
                compound_duration::format_dhms(time_elapsed)
            )
            .as_str(),
        );
    }

    // second line
    prompt.push_str(
        format!(
            "\n%F{{{}}}{}%f{}",
            prompt_symbol_color, symbol, NON_BREAKING_SPACE
        )
        .as_str(),
    );

    print!("{}", prompt);
}
