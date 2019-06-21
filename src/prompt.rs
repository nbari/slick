use crate::envs::get_env;
use clap::ArgMatches;
use compound_duration;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    env,
    fmt::Write,
    time::{Duration, SystemTime},
};
use users::{get_current_uid, get_user_by_uid};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Prompt {
    action: String,
    branch: String,
    remote: String,
    status: String,
    staged: bool,
}

fn is_root() -> bool {
    let user = get_user_by_uid(get_current_uid()).unwrap();
    if user.uid() == 0 {
        return true;
    }
    return false;
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

    let mut symbol = get_env("SLICK_PROMPT_SYMBOL");
    if is_root() {
        symbol = get_env("SLICK_PROMPT_ROOT_SYMBOL");
    }
    if keymap == "vicmd" {
        symbol = get_env("SLICK_PROMPT_VICMD_SYMBOL");
    }

    let mut prompt_symbol_color = get_env("SLICK_PROMPT_ERROR_COLOR");
    if symbol == get_env("SLICK_PROMPT_VICMD_SYMBOL") {
        prompt_symbol_color = get_env("SLICK_PROMPT_VICMD_COLOR")
    } else if last_return_code == "0" {
        prompt_symbol_color = get_env("SLICK_PROMPT_SYMBOL_COLOR")
    }

    let mut prompt = String::new();

    // prefix with "root" if UID = 0
    if is_root() {
        drop(write!(
            &mut prompt,
            "%F{{{}}}%n ",
            get_env("SLICK_PROMPT_ROOT_COLOR")
        ));
    }

    // if SSH_CONNECTION environment var found
    if let Ok(_) = env::var("SSH_CONNECTION") {
        drop(write!(
            &mut prompt,
            "%F{{{}}}%n@%m ",
            get_env("SLICK_PROMPT_SSH_COLOR")
        ));
    }

    // start the prompt with the current dir %~
    drop(write!(
        &mut prompt,
        "%F{{{}}}%~",
        get_env("SLICK_PROMPT_PATH_COLOR")
    ));

    // branch
    if deserialized.branch == "master" {
        drop(write!(
            &mut prompt,
            " %F{{{}}}{}",
            get_env("SLICK_PROMPT_GIT_MASTER_BRANCH_COLOR"),
            deserialized.branch
        ))
    } else {
        drop(write!(
            &mut prompt,
            " %F{{{}}}{}",
            get_env("SLICK_PROMPT_GIT_BRANCH_COLOR"),
            deserialized.branch
        ))
    }

    // git status
    if !deserialized.status.is_empty() {
        drop(write!(
            &mut prompt,
            " %F{{{}}}[{}]",
            get_env("SLICK_PROMPT_GIT_STATUS_COLOR"),
            deserialized.status
        ))
    }

    // git remote
    if !deserialized.remote.is_empty() {
        drop(write!(
            &mut prompt,
            " %F{{{}}}{}",
            get_env("SLICK_PROMPT_GIT_REMOTE_COLOR"),
            deserialized.remote
        ))
    }

    // git action
    if !deserialized.action.is_empty() {
        drop(write!(
            &mut prompt,
            " %F{{{}}}{}",
            get_env("SLICK_PROMPT_GIT_ACTION_COLOR"),
            deserialized.action
        ))
    }

    // git staged
    if deserialized.staged {
        drop(write!(
            &mut prompt,
            " %F{{{}}}[staged]",
            get_env("SLICK_PROMPT_GIT_STAGED_COLOR"),
        ))
    }

    // time elapsed
    let max_time: usize = match get_env("SLICK_PROMPT_CMD_MAX_EXEC_TIME").parse() {
        Ok(n) => n,
        Err(_) => 5,
    };
    if time_elapsed > max_time {
        drop(write!(
            &mut prompt,
            " %F{{{}}}{}",
            get_env("SLICK_PROMPT_TIME_ELAPSED_COLOR"),
            compound_duration::format_dhms(time_elapsed)
        ))
    }

    // second line
    drop(write!(
        &mut prompt,
        "\n%F{{{}}}{}%f{}",
        prompt_symbol_color,
        symbol,
        get_env("SLICK_PROMPT_NON_BREAKING_SPACE"),
    ));

    print!("{}", prompt);
}
