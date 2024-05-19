use crate::get_env;
use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use std::{
    env,
    process::exit,
    time::{Duration, SystemTime},
};
use users::{get_current_uid, get_user_by_uid};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Prompt {
    action: String,
    branch: String,
    remote: Vec<String>,
    staged: bool,
    status: String,
    u_name: String,
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

pub fn display(matches: &ArgMatches) {
    let keymap = matches
        .get_one("keymap")
        .map_or("main".to_string(), |s: &String| s.to_string());
    let last_return_code = matches
        .get_one("last_return_code")
        .map_or("0".to_string(), |s: &String| s.to_string());
    let serialized = matches
        .get_one("data")
        .map_or(String::new(), |s: &String| s.to_string());
    let deserialized: Prompt =
        serde_json::from_str(&serialized).unwrap_or_else(|_| Prompt::default());

    // get time elapsed
    let epochtime: u64 = matches
        .get_one("time")
        .map_or(String::new(), |s: &String| s.to_string())
        .parse::<u64>()
        .ok()
        .map_or_else(
            || match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(n) => n.as_secs(),
                Err(e) => {
                    eprintln!("SystemTime before UNIX EPOCH!: {e}");
                    exit(1)
                }
            },
            |v| v,
        );

    let d = SystemTime::UNIX_EPOCH + Duration::from_secs(epochtime);
    let time_elapsed = d.elapsed().map_or(0, |elapsed| elapsed.as_secs());

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
        prompt_symbol_color = get_env("SLICK_PROMPT_VICMD_COLOR");
    } else if last_return_code == "0" {
        prompt_symbol_color = get_env("SLICK_PROMPT_SYMBOL_COLOR");
    }

    let mut prompt: Vec<String> = Vec::new();

    if is_remote() {
        if is_root() {
            // prefix with "root" if UID = 0
            prompt.push(format!(
                "%F{{{}}}%n%F{{{}}}@%m",
                get_env("SLICK_PROMPT_ROOT_COLOR"),
                get_env("SLICK_PROMPT_SSH_COLOR")
            ));
        } else {
            prompt.push(format!("%F{{{}}}%n@%m", get_env("SLICK_PROMPT_SSH_COLOR")));
        }
    } else if is_root() {
        // prefix with "root" if UID = 0
        prompt.push(format!("%F{{{}}}%n", get_env("SLICK_PROMPT_ROOT_COLOR")));
    }

    // PIPENV
    if !get_env("PIPENV_ACTIVE").is_empty() || !get_env("VIRTUAL_ENV").is_empty() {
        // Check if env VIRTUAL_ENV_PROMPT if set else use VIRTUAL_ENV
        let venv = env::var("VIRTUAL_ENV_PROMPT").unwrap_or_else(|_| {
            get_env("VIRTUAL_ENV")
                .split('/')
                .last()
                .map_or_else(String::new, |s| {
                    if get_env("PIPENV_ACTIVE").is_empty() {
                        s.to_string()
                    } else {
                        s.split('-').next().unwrap_or("").to_string()
                    }
                })
        });

        if !venv.is_empty() {
            prompt.push(format!(
                "%F{{{}}}({})",
                get_env("PIPENV_ACTIVE_COLOR"),
                venv
            ));
        }
    }

    // git u_name
    if get_env("SLICK_PROMPT_NO_GIT_UNAME").is_empty() && !deserialized.u_name.is_empty() {
        prompt.push(format!(
            "%F{{{}}}{}",
            get_env("SLICK_PROMPT_GIT_UNAME_COLOR"),
            deserialized.u_name
        ));
    }

    // start the prompt with the current dir %~
    prompt.push(format!("%F{{{}}}%~", get_env("SLICK_PROMPT_PATH_COLOR")));

    // branch
    if !deserialized.branch.is_empty() {
        if deserialized.branch == "master" || deserialized.branch == "main" {
            prompt.push(format!(
                "%F{{{}}}{}",
                get_env("SLICK_PROMPT_GIT_MASTER_BRANCH_COLOR"),
                deserialized.branch
            ));
        } else {
            prompt.push(format!(
                "%F{{{}}}{}",
                get_env("SLICK_PROMPT_GIT_BRANCH_COLOR"),
                deserialized.branch
            ));
        }
    }

    // git status
    if !deserialized.status.is_empty() {
        prompt.push(format!(
            "%F{{{}}}[{}]",
            get_env("SLICK_PROMPT_GIT_STATUS_COLOR"),
            deserialized.status
        ));
    }

    // git remote
    if !deserialized.remote.is_empty() {
        prompt.push(format!(
            "%F{{{}}}{}",
            get_env("SLICK_PROMPT_GIT_REMOTE_COLOR"),
            deserialized.remote.join(" ")
        ));
    }

    // git action
    if !deserialized.action.is_empty() {
        prompt.push(format!(
            "%F{{{}}}{}",
            get_env("SLICK_PROMPT_GIT_ACTION_COLOR"),
            deserialized.action
        ));
    }

    // git staged
    if deserialized.staged {
        prompt.push(format!(
            "%F{{{}}}[staged]",
            get_env("SLICK_PROMPT_GIT_STAGED_COLOR"),
        ));
    }

    // time elapsed
    let max_time: u64 = get_env("SLICK_PROMPT_CMD_MAX_EXEC_TIME")
        .parse()
        .unwrap_or(5);
    if time_elapsed > max_time {
        prompt.push(format!(
            "%F{{{}}}{}",
            get_env("SLICK_PROMPT_TIME_ELAPSED_COLOR"),
            compound_duration::format_dhms(time_elapsed)
        ));
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
