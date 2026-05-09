use crate::{context::collect_context_markers, get_env, get_env_var};
use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::Write as _,
    path::{Component, Path, PathBuf},
    process::exit,
    time::{Duration, SystemTime},
};
use uzers::{get_current_uid, get_user_by_uid};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
struct Prompt {
    action: String,
    branch: String,
    remote: Vec<String>,
    staged: bool,
    status: String,
    u_name: String,
    auth_failed: bool,
}

const TRANSIENT_TIMESTAMP_COLOR: &str = "8";

fn is_root() -> bool {
    get_user_by_uid(get_current_uid()).is_some_and(|user| user.uid() == 0)
}

fn is_remote() -> bool {
    env::var("SSH_CONNECTION").is_ok()
}

fn append_identity_prefix(prompt: &mut String, is_root_user: bool, is_remote_user: bool) {
    if is_remote_user {
        if is_root_user {
            let _ = write!(
                prompt,
                "%F{{{}}}%n%F{{{}}}@%m ",
                get_env("SLICK_PROMPT_ROOT_COLOR"),
                get_env("SLICK_PROMPT_SSH_COLOR")
            );
        } else {
            let _ = write!(prompt, "%F{{{}}}%n@%m ", get_env("SLICK_PROMPT_SSH_COLOR"));
        }
    } else if is_root_user {
        let _ = write!(prompt, "%F{{{}}}%n ", get_env("SLICK_PROMPT_ROOT_COLOR"));
    }
}

fn append_context_markers(prompt: &mut String) {
    let short = get_env("SLICK_PROMPT_SHORT_CONTEXT") == "1";
    for marker in collect_context_markers(short) {
        let _ = write!(prompt, "%F{{{}}}{} ", marker.color, marker.text);
    }
}

fn escape_prompt_literal(segment: &str) -> String {
    segment.replace('%', "%%")
}

fn compact_path_segments<'a>(segments: impl Iterator<Item = &'a str>) -> String {
    let parts: Vec<&str> = segments.collect();
    if parts.is_empty() {
        return String::new();
    }

    let mut compacted = String::new();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            compacted.push('/');
        }

        if index + 1 == parts.len() {
            compacted.push_str(&escape_prompt_literal(part));
        } else if let Some(ch) = part.chars().next() {
            compacted.push(ch);
        }
    }

    compacted
}

fn compact_path(path: &Path, home: Option<&Path>) -> String {
    if let Some(home) = home
        && let Ok(relative) = path.strip_prefix(home)
    {
        let rendered = compact_path_segments(
            relative
                .iter()
                .filter_map(|segment| segment.to_str())
                .filter(|segment| !segment.is_empty()),
        );

        return if rendered.is_empty() {
            "~".to_string()
        } else {
            format!("~/{rendered}")
        };
    }

    let mut prefix = String::new();
    let mut segments = Vec::new();

    for component in path.components() {
        match component {
            Component::RootDir => prefix.push('/'),
            Component::Normal(segment) => {
                if let Some(segment) = segment.to_str() {
                    segments.push(segment);
                }
            }
            Component::CurDir => segments.push("."),
            Component::ParentDir => segments.push(".."),
            Component::Prefix(prefix_component) => {
                prefix.push_str(&prefix_component.as_os_str().to_string_lossy());
            }
        }
    }

    let rendered = compact_path_segments(segments.into_iter());
    if rendered.is_empty() {
        if prefix.is_empty() {
            ".".to_string()
        } else {
            prefix
        }
    } else if prefix.is_empty() {
        rendered
    } else if prefix.ends_with('/') {
        format!("{prefix}{rendered}")
    } else {
        format!("{prefix}/{rendered}")
    }
}

fn current_path_symbol() -> String {
    if get_env("SLICK_PROMPT_SHORT_PATH") != "1" {
        return "%~".to_string();
    }

    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let home_dir = env::var_os("HOME").map(PathBuf::from);

    compact_path(&current_dir, home_dir.as_deref())
}

fn append_branch(prompt: &mut String, branch: &str) {
    if branch.is_empty() {
        return;
    }

    let branch_color = if branch == "master" || branch == "main" {
        get_env("SLICK_PROMPT_GIT_MAIN_BRANCH_COLOR")
    } else {
        get_env("SLICK_PROMPT_GIT_BRANCH_COLOR")
    };
    let branch_symbol = get_env("SLICK_PROMPT_GIT_BRANCH_SYMBOL");

    if !branch_symbol.is_empty() {
        let _ = write!(
            prompt,
            "%F{{{}}}{} ",
            get_env("SLICK_PROMPT_GIT_BRANCH_SYMBOL_COLOR"),
            branch_symbol
        );
    }

    let _ = write!(prompt, "%F{{{branch_color}}}{branch}");
}

fn prompt_symbol(keymap: &str, last_return_code: &str, is_root_user: bool) -> (String, String) {
    let vicmd_symbol = get_env("SLICK_PROMPT_VICMD_SYMBOL");
    let symbol = if keymap == "vicmd" {
        vicmd_symbol
    } else if is_root_user {
        get_env("SLICK_PROMPT_ROOT_SYMBOL")
    } else {
        get_env("SLICK_PROMPT_SYMBOL")
    };

    let color = if symbol == vicmd_symbol {
        get_env("SLICK_PROMPT_VICMD_COLOR")
    } else if last_return_code == "0" {
        get_env("SLICK_PROMPT_SYMBOL_COLOR")
    } else {
        get_env("SLICK_PROMPT_ERROR_COLOR")
    };

    (symbol.to_string(), color.to_string())
}

fn elapsed_from_timestamp(matches: &ArgMatches) -> u64 {
    let epochtime = matches
        .get_one("time")
        .map_or(String::new(), String::clone)
        .parse::<u64>()
        .ok()
        .unwrap_or_else(
            || match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(now) => now.as_secs(),
                Err(error) => {
                    eprintln!("SystemTime before UNIX EPOCH!: {error}");
                    exit(1)
                }
            },
        );

    let duration = SystemTime::UNIX_EPOCH + Duration::from_secs(epochtime);
    duration.elapsed().map_or(0, |elapsed| elapsed.as_secs())
}

fn parse_time_elapsed(matches: &ArgMatches) -> u64 {
    matches.get_one::<String>("elapsed").map_or_else(
        || elapsed_from_timestamp(matches),
        |elapsed| {
            elapsed.parse::<i64>().ok().map_or(
                0,
                |value| {
                    if value < 0 { 0 } else { value.cast_unsigned() }
                },
            )
        },
    )
}

fn append_git_user_name(prompt: &mut String, deserialized: &Prompt) {
    if get_env_var("SLICK_PROMPT_NO_GIT_UNAME").is_empty() && !deserialized.u_name.is_empty() {
        let _ = write!(
            prompt,
            "%F{{{}}}{} ",
            get_env("SLICK_PROMPT_GIT_UNAME_COLOR"),
            deserialized.u_name
        );
    }
}

fn append_git_metadata(prompt: &mut String, deserialized: &Prompt) {
    if !deserialized.branch.is_empty() {
        append_branch(prompt, &deserialized.branch);
        prompt.push(' ');
    }

    if !deserialized.status.is_empty() {
        let _ = write!(
            prompt,
            "%F{{{}}}[{}] ",
            get_env("SLICK_PROMPT_GIT_STATUS_COLOR"),
            deserialized.status
        );
    }

    if !deserialized.remote.is_empty() {
        let _ = write!(
            prompt,
            "%F{{{}}}{} ",
            get_env("SLICK_PROMPT_GIT_REMOTE_COLOR"),
            deserialized.remote.join(" ")
        );
    }

    if !deserialized.action.is_empty() {
        let _ = write!(
            prompt,
            "%F{{{}}}{} ",
            get_env("SLICK_PROMPT_GIT_ACTION_COLOR"),
            deserialized.action
        );
    }

    if deserialized.staged {
        let _ = write!(
            prompt,
            "%F{{{}}}[staged] ",
            get_env("SLICK_PROMPT_GIT_STAGED_COLOR"),
        );
    }

    if deserialized.auth_failed {
        let _ = write!(
            prompt,
            "%F{{{}}}{} ",
            get_env("SLICK_PROMPT_GIT_AUTH_COLOR"),
            get_env("SLICK_PROMPT_GIT_AUTH_SYMBOL")
        );
    }
}

fn append_elapsed(prompt: &mut String, time_elapsed: u64) {
    let max_time = get_env("SLICK_PROMPT_CMD_MAX_EXEC_TIME")
        .parse()
        .unwrap_or(5);
    if time_elapsed > max_time {
        let _ = write!(
            prompt,
            "%F{{{}}}{} ",
            get_env("SLICK_PROMPT_TIME_ELAPSED_COLOR"),
            compound_duration::format_dhms(time_elapsed)
        );
    }
}

fn trim_trailing_space(prompt: &mut String) {
    if prompt.ends_with(' ') {
        prompt.pop();
    }
}

fn append_cursor_shape(prompt: &mut String, _keymap: &str) {
    let cursor_shape = get_env("SLICK_PROMPT_CURSOR_SHAPE");

    if !cursor_shape.is_empty() && (0..=6).contains(&cursor_shape.parse::<u8>().unwrap_or(255)) {
        let _ = write!(prompt, "%{{\x1b[{cursor_shape} q%}}");
    }
}

fn build_transient_prompt(
    deserialized: &Prompt,
    is_root_user: bool,
    is_remote_user: bool,
    symbol: &str,
    prompt_symbol_color: &str,
    transient_timestamp: &str,
    keymap: &str,
) -> String {
    let mut prompt = String::with_capacity(256);

    append_cursor_shape(&mut prompt, keymap);
    append_identity_prefix(&mut prompt, is_root_user, is_remote_user);

    if !transient_timestamp.is_empty() {
        let _ = write!(
            prompt,
            "%F{{{TRANSIENT_TIMESTAMP_COLOR}}}{transient_timestamp} "
        );
    }

    append_context_markers(&mut prompt);
    let path_symbol = current_path_symbol();
    let _ = write!(
        prompt,
        "%F{{{}}}{path_symbol}",
        get_env("SLICK_PROMPT_PATH_COLOR")
    );

    if !deserialized.branch.is_empty() {
        prompt.push(' ');
        append_branch(&mut prompt, &deserialized.branch);
    }

    let _ = write!(
        prompt,
        " %F{{{}}}{}%f{}",
        prompt_symbol_color,
        symbol,
        get_env("SLICK_PROMPT_NON_BREAKING_SPACE"),
    );

    prompt
}

fn build_full_prompt(
    deserialized: &Prompt,
    is_root_user: bool,
    is_remote_user: bool,
    symbol: &str,
    prompt_symbol_color: &str,
    time_elapsed: u64,
    keymap: &str,
) -> String {
    let mut prompt = String::with_capacity(256);

    append_cursor_shape(&mut prompt, keymap);
    append_identity_prefix(&mut prompt, is_root_user, is_remote_user);

    append_context_markers(&mut prompt);
    append_git_user_name(&mut prompt, deserialized);

    let path_symbol = current_path_symbol();
    let _ = write!(
        prompt,
        "%F{{{}}}{path_symbol} ",
        get_env("SLICK_PROMPT_PATH_COLOR")
    );

    append_git_metadata(&mut prompt, deserialized);
    append_elapsed(&mut prompt, time_elapsed);
    trim_trailing_space(&mut prompt);

    let _ = write!(
        prompt,
        "\n%F{{{}}}{}%f{}",
        prompt_symbol_color,
        symbol,
        get_env("SLICK_PROMPT_NON_BREAKING_SPACE"),
    );

    prompt
}

pub fn display(matches: &ArgMatches) {
    let keymap = matches
        .get_one("keymap")
        .map_or_else(|| "main".to_string(), String::clone);
    let last_return_code = matches
        .get_one("last_return_code")
        .map_or_else(|| "0".to_string(), String::clone);
    let serialized = matches
        .get_one("data")
        .map_or_else(String::new, String::clone);
    let deserialized: Prompt =
        serde_json::from_str(&serialized).unwrap_or_else(|_| Prompt::default());
    let transient = matches.get_flag("transient");
    let transient_timestamp = matches
        .get_one::<String>("transient_timestamp")
        .map_or("", String::as_str);

    let is_root_user = is_root();
    let is_remote_user = is_remote();
    let (symbol, prompt_symbol_color) = prompt_symbol(&keymap, &last_return_code, is_root_user);

    if transient {
        print!(
            "{}",
            build_transient_prompt(
                &deserialized,
                is_root_user,
                is_remote_user,
                &symbol,
                &prompt_symbol_color,
                transient_timestamp,
                &keymap,
            )
        );
        return;
    }

    print!(
        "{}",
        build_full_prompt(
            &deserialized,
            is_root_user,
            is_remote_user,
            &symbol,
            &prompt_symbol_color,
            parse_time_elapsed(matches),
            &keymap,
        )
    );
}

#[cfg(test)]
mod tests {
    use super::{append_branch, compact_path};
    use std::path::Path;

    #[test]
    fn test_append_branch_uses_separate_symbol_color() {
        let mut prompt = String::new();
        append_branch(&mut prompt, "main");
        assert_eq!(prompt, "%F{2} %F{160}main");
    }

    #[test]
    fn test_compact_path_for_home_nested_path() {
        let path = Path::new("/var/home/nbari/projects/rust/slick");
        let home = Path::new("/var/home/nbari");

        assert_eq!(compact_path(path, Some(home)), "~/p/r/slick");
    }

    #[test]
    fn test_compact_path_for_absolute_path_outside_home() {
        let path = Path::new("/var/home/nbari/projects/rust/slick");
        let home = Path::new("/tmp/home");

        assert_eq!(compact_path(path, Some(home)), "/v/h/n/p/r/slick");
    }

    #[test]
    fn test_compact_path_for_home_root() {
        let path = Path::new("/var/home/nbari");
        let home = Path::new("/var/home/nbari");

        assert_eq!(compact_path(path, Some(home)), "~");
    }
}
