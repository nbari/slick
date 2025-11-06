use crate::{get_env, get_env_var, get_env_var_or};
use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::Write as _,
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
    auth_failed: bool,
}

// check if current user is root or not
fn is_root() -> bool {
    get_user_by_uid(get_current_uid())
        .map(|user| user.uid() == 0)
        .unwrap_or(false)
}

// check if current user is remote or not
fn is_remote() -> bool {
    env::var("SSH_CONNECTION").is_ok()
}

pub fn display(matches: &ArgMatches) {
    let keymap = matches
        .get_one("keymap")
        .map_or_else(|| "main".to_string(), |s: &String| s.to_string());
    let last_return_code = matches
        .get_one("last_return_code")
        .map_or_else(|| "0".to_string(), |s: &String| s.to_string());
    let serialized = matches
        .get_one("data")
        .map_or_else(String::new, |s: &String| s.to_string());
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

    // Cache frequently used values
    let is_root_user = is_root();
    let is_remote_user = is_remote();
    let vicmd_symbol = get_env("SLICK_PROMPT_VICMD_SYMBOL");

    // define symbol
    let symbol = if keymap == "vicmd" {
        vicmd_symbol
    } else if is_root_user {
        get_env("SLICK_PROMPT_ROOT_SYMBOL")
    } else {
        get_env("SLICK_PROMPT_SYMBOL")
    };

    // symbol color
    let prompt_symbol_color = if symbol == vicmd_symbol {
        get_env("SLICK_PROMPT_VICMD_COLOR")
    } else if last_return_code == "0" {
        get_env("SLICK_PROMPT_SYMBOL_COLOR")
    } else {
        get_env("SLICK_PROMPT_ERROR_COLOR")
    };

    // Use String builder instead of Vec for better performance
    // Estimate capacity: ~200 chars is typical for a prompt
    let mut prompt = String::with_capacity(256);

    if is_remote_user {
        if is_root_user {
            // prefix with "root" if UID = 0
            write!(
                prompt,
                "%F{{{}}}%n%F{{{}}}@%m ",
                get_env("SLICK_PROMPT_ROOT_COLOR"),
                get_env("SLICK_PROMPT_SSH_COLOR")
            )
            .unwrap();
        } else {
            write!(prompt, "%F{{{}}}%n@%m ", get_env("SLICK_PROMPT_SSH_COLOR")).unwrap();
        }
    } else if is_root_user {
        // prefix with "root" if UID = 0
        write!(prompt, "%F{{{}}}%n ", get_env("SLICK_PROMPT_ROOT_COLOR")).unwrap();
    }

    // PIPENV - optimized with rsplit_once
    let pipenv_active = get_env_var("PIPENV_ACTIVE");
    let virtual_env = get_env_var("VIRTUAL_ENV");
    if !pipenv_active.is_empty() || !virtual_env.is_empty() {
        // Check if env VIRTUAL_ENV_PROMPT if set else use VIRTUAL_ENV
        let venv = env::var("VIRTUAL_ENV_PROMPT").unwrap_or_else(|_| {
            // Use rsplit_once for better performance
            if let Some((_, last)) = virtual_env.rsplit_once('/') {
                if pipenv_active.is_empty() {
                    last.to_string()
                } else {
                    // Get first part before '-' for pipenv
                    last.split_once('-')
                        .map_or(last, |(first, _)| first)
                        .to_string()
                }
            } else {
                String::new()
            }
        });

        if !venv.is_empty() {
            write!(
                prompt,
                "%F{{{}}}({}) ",
                get_env_var_or("PIPENV_ACTIVE_COLOR", "7"),
                venv
            )
            .unwrap();
        }
    }

    // git u_name
    if get_env_var("SLICK_PROMPT_NO_GIT_UNAME").is_empty() && !deserialized.u_name.is_empty() {
        write!(
            prompt,
            "%F{{{}}}{}",
            get_env("SLICK_PROMPT_GIT_UNAME_COLOR"),
            deserialized.u_name
        )
        .unwrap();
        prompt.push(' ');
    }

    // start the prompt with the current dir %~
    write!(prompt, "%F{{{}}}%~ ", get_env("SLICK_PROMPT_PATH_COLOR")).unwrap();

    // branch
    if !deserialized.branch.is_empty() {
        if deserialized.branch == "master" || deserialized.branch == "main" {
            write!(
                prompt,
                "%F{{{}}}{}",
                get_env("SLICK_PROMPT_GIT_MASTER_BRANCH_COLOR"),
                deserialized.branch
            )
            .unwrap();
        } else {
            write!(
                prompt,
                "%F{{{}}}{}",
                get_env("SLICK_PROMPT_GIT_BRANCH_COLOR"),
                deserialized.branch
            )
            .unwrap();
        }
        prompt.push(' ');
    }

    // git status
    if !deserialized.status.is_empty() {
        write!(
            prompt,
            "%F{{{}}}[{}]",
            get_env("SLICK_PROMPT_GIT_STATUS_COLOR"),
            deserialized.status
        )
        .unwrap();
        prompt.push(' ');
    }

    // git remote
    if !deserialized.remote.is_empty() {
        write!(
            prompt,
            "%F{{{}}}{}",
            get_env("SLICK_PROMPT_GIT_REMOTE_COLOR"),
            deserialized.remote.join(" ")
        )
        .unwrap();
        prompt.push(' ');
    }

    // git action
    if !deserialized.action.is_empty() {
        write!(
            prompt,
            "%F{{{}}}{}",
            get_env("SLICK_PROMPT_GIT_ACTION_COLOR"),
            deserialized.action
        )
        .unwrap();
        prompt.push(' ');
    }

    // git staged
    if deserialized.staged {
        write!(
            prompt,
            "%F{{{}}}[staged]",
            get_env("SLICK_PROMPT_GIT_STAGED_COLOR"),
        )
        .unwrap();
        prompt.push(' ');
    }

    // authentication failed warning
    if deserialized.auth_failed {
        write!(
            prompt,
            "%F{{{}}}{}",
            get_env("SLICK_PROMPT_GIT_AUTH_COLOR"),
            get_env("SLICK_PROMPT_GIT_AUTH_SYMBOL")
        )
        .unwrap();
        prompt.push(' ');
    }

    // time elapsed
    let max_time: u64 = get_env("SLICK_PROMPT_CMD_MAX_EXEC_TIME")
        .parse()
        .unwrap_or(5);
    if time_elapsed > max_time {
        write!(
            prompt,
            "%F{{{}}}{}",
            get_env("SLICK_PROMPT_TIME_ELAPSED_COLOR"),
            compound_duration::format_dhms(time_elapsed)
        )
        .unwrap();
        prompt.push(' ');
    }

    // Remove trailing space if present
    if prompt.ends_with(' ') {
        prompt.pop();
    }

    // second prompt line
    write!(
        prompt,
        "\n%F{{{}}}{}%f{}",
        prompt_symbol_color,
        symbol,
        get_env("SLICK_PROMPT_NON_BREAKING_SPACE"),
    )
    .unwrap();

    print!("{prompt}");
}
