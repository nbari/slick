use crate::envs::get_env;
use clap::ArgMatches;
use compound_duration;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    env,
    time::{Duration, SystemTime},
};
use users::{get_current_uid, get_user_by_uid};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Prompt {
    action: String,
    branch: String,
    remote: Vec<String>,
    status: String,
    staged: bool,
}

fn is_root() -> bool {
    let user = get_user_by_uid(get_current_uid()).unwrap();
    if user.uid() == 0 {
        return true;
    }
    false
}

fn is_remote() -> bool {
    env::var("SSH_CONNECTION").is_ok()
}

pub fn display(sub_matches: &ArgMatches) {
    let keymap = sub_matches.value_of("keymap").unwrap_or("main");
    let last_return_code = sub_matches.value_of("last_return_code").unwrap_or("0");
    let serialized = sub_matches.value_of("data").unwrap_or("");
    let deserialized: Prompt = match serde_json::from_str(serialized) {
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
            _ => panic!("SystemTime before UNIX EPOCH!"),
        },
    };
    let d = SystemTime::UNIX_EPOCH + Duration::from_secs(epochtime);
    let time_elapsed = match d.elapsed() {
        Ok(elapsed) => elapsed.as_secs() as usize,
        Err(_) => 0,
    };

    // define symbol
    let symbol = if keymap == "vicmd" {
        get_env("SLICK_PROMPT_VICMD_SYMBOL")
    } else if is_root() {
        get_env("SLICK_PROMPT_ROOT_SYMBOL")
    } else {
        get_env("SLICK_PROMPT_SYMBOL")
    };

    // symbol color
    let mut prompt_symbol_color = get_env("SLICK_PROMPT_ERROR_COLOR");
    if symbol == get_env("SLICK_PROMPT_VICMD_SYMBOL") {
        prompt_symbol_color = get_env("SLICK_PROMPT_VICMD_COLOR")
    } else if last_return_code == "0" {
        prompt_symbol_color = get_env("SLICK_PROMPT_SYMBOL_COLOR")
    }

    let mut prompt: Vec<String> = Vec::new();

    if is_remote() {
        if is_root() {
            // prefix with "root" if UID = 0
            prompt.push(format!(
                "%F{{{}}}%n%F{{{}}}@%m",
                get_env("SLICK_PROMPT_ROOT_COLOR"),
                get_env("SLICK_PROMPT_SSH_COLOR")
            ))
        } else {
            prompt.push(format!("%F{{{}}}%n@%m", get_env("SLICK_PROMPT_SSH_COLOR")))
        }
    } else if is_root() {
        // prefix with "root" if UID = 0
        prompt.push(format!("%F{{{}}}%n", get_env("SLICK_PROMPT_ROOT_COLOR")))
    }

    // start the prompt with the current dir %~
    prompt.push(format!("%F{{{}}}%~", get_env("SLICK_PROMPT_PATH_COLOR")));

    // branch
    if !deserialized.branch.is_empty() {
        if deserialized.branch == "master" {
            prompt.push(format!(
                "%F{{{}}}{}",
                get_env("SLICK_PROMPT_GIT_MASTER_BRANCH_COLOR"),
                deserialized.branch
            ))
        } else {
            prompt.push(format!(
                "%F{{{}}}{}",
                get_env("SLICK_PROMPT_GIT_BRANCH_COLOR"),
                deserialized.branch
            ))
        }
    }

    // git status
    if !deserialized.status.is_empty() {
        prompt.push(format!(
            "%F{{{}}}[{}]",
            get_env("SLICK_PROMPT_GIT_STATUS_COLOR"),
            deserialized.status
        ))
    }

    // git remote
    if !deserialized.remote.is_empty() {
        prompt.push(format!(
            "%F{{{}}}{}",
            get_env("SLICK_PROMPT_GIT_REMOTE_COLOR"),
            deserialized.remote.join(" ")
        ))
    }

    // git action
    if !deserialized.action.is_empty() {
        prompt.push(format!(
            "%F{{{}}}{}",
            get_env("SLICK_PROMPT_GIT_ACTION_COLOR"),
            deserialized.action
        ))
    }

    // git staged
    if deserialized.staged {
        prompt.push(format!(
            "%F{{{}}}[staged]",
            get_env("SLICK_PROMPT_GIT_STAGED_COLOR"),
        ))
    }

    // time elapsed
    let max_time: usize = match get_env("SLICK_PROMPT_CMD_MAX_EXEC_TIME").parse() {
        Ok(n) => n,
        Err(_) => 5,
    };
    if time_elapsed > max_time {
        prompt.push(format!(
            "%F{{{}}}{}",
            get_env("SLICK_PROMPT_TIME_ELAPSED_COLOR"),
            compound_duration::format_dhms(time_elapsed)
        ))
    }

    // second prompt line
    prompt.push(format!(
        "\n%F{{{}}}{}%f{}",
        prompt_symbol_color,
        symbol,
        get_env("SLICK_PROMPT_NON_BREAKING_SPACE"),
    ));

    print!("{}", prompt.join(" "));
}
