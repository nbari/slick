use crate::{context::collect_context_markers, get_env, get_env_var};
use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::Write as _,
    fs,
    path::{Component, Path, PathBuf},
    process::exit,
    time::{Duration, SystemTime},
};
use uzers::get_current_uid;

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
const INTERNAL_DOLLAR_PSVAR_ENV: &str = "_SLICK_PROMPT_PSVAR_DOLLAR";
const INTERNAL_BACKTICK_PSVAR_ENV: &str = "_SLICK_PROMPT_PSVAR_BACKTICK";
const INTERNAL_BACKSLASH_PSVAR_ENV: &str = "_SLICK_PROMPT_PSVAR_BACKSLASH";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PromptLiteralEncoding {
    Backslash,
    Psvar {
        dollar: usize,
        backtick: usize,
        backslash: usize,
    },
}

fn is_root() -> bool {
    get_current_uid() == 0
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

fn append_context_markers(prompt: &mut String, encoding: PromptLiteralEncoding) {
    let short = get_env("SLICK_PROMPT_SHORT_CONTEXT") == "1";
    for marker in collect_context_markers(short) {
        let _ = write!(
            prompt,
            "%F{{{}}}{} ",
            marker.color,
            escape_prompt_literal(&marker.text, encoding)
        );
    }
}

fn parse_psvar_index(value: &str) -> Option<usize> {
    value.parse::<usize>().ok().filter(|index| *index > 0)
}

fn prompt_literal_encoding(dollar: &str, backtick: &str, backslash: &str) -> PromptLiteralEncoding {
    match (
        parse_psvar_index(dollar),
        parse_psvar_index(backtick),
        parse_psvar_index(backslash),
    ) {
        (Some(dollar), Some(backtick), Some(backslash)) => PromptLiteralEncoding::Psvar {
            dollar,
            backtick,
            backslash,
        },
        _ => PromptLiteralEncoding::Backslash,
    }
}

fn current_prompt_literal_encoding() -> PromptLiteralEncoding {
    prompt_literal_encoding(
        &get_env_var(INTERNAL_DOLLAR_PSVAR_ENV),
        &get_env_var(INTERNAL_BACKTICK_PSVAR_ENV),
        &get_env_var(INTERNAL_BACKSLASH_PSVAR_ENV),
    )
}

fn escape_prompt_literal(segment: &str, encoding: PromptLiteralEncoding) -> String {
    let mut escaped = String::with_capacity(segment.len());
    for character in segment.chars() {
        match character {
            '%' => escaped.push_str("%%"),
            '\\' | '$' | '`' if encoding == PromptLiteralEncoding::Backslash => {
                escaped.push('\\');
                escaped.push(character);
            }
            '$' => {
                if let PromptLiteralEncoding::Psvar { dollar, .. } = encoding {
                    let _ = write!(escaped, "%{dollar}v");
                }
            }
            '`' => {
                if let PromptLiteralEncoding::Psvar { backtick, .. } = encoding {
                    let _ = write!(escaped, "%{backtick}v");
                }
            }
            '\\' => {
                if let PromptLiteralEncoding::Psvar { backslash, .. } = encoding {
                    let _ = write!(escaped, "%{backslash}v");
                }
            }
            _ => escaped.push(character),
        }
    }
    escaped
}

fn compact_path_segments<'a>(
    segments: impl Iterator<Item = &'a str>,
    encoding: PromptLiteralEncoding,
) -> String {
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
            compacted.push_str(&escape_prompt_literal(part, encoding));
        } else if let Some(ch) = part.chars().next() {
            compacted.push_str(&escape_prompt_literal(&ch.to_string(), encoding));
        }
    }

    compacted
}

fn compact_path(path: &Path, home: Option<&Path>, encoding: PromptLiteralEncoding) -> String {
    if let Some(home) = home
        && let Ok(relative) = path.strip_prefix(home)
    {
        let rendered = compact_path_segments(
            relative
                .iter()
                .filter_map(|segment| segment.to_str())
                .filter(|segment| !segment.is_empty()),
            encoding,
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

    let rendered = compact_path_segments(segments.into_iter(), encoding);
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

fn current_path_symbol(encoding: PromptLiteralEncoding) -> String {
    if get_env("SLICK_PROMPT_SHORT_PATH") != "1" {
        return "%~".to_string();
    }

    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let current_dir = fs::canonicalize(&current_dir).unwrap_or(current_dir);
    let home_dir = env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| fs::canonicalize(&home).unwrap_or(home));

    compact_path(&current_dir, home_dir.as_deref(), encoding)
}

fn append_branch(prompt: &mut String, branch: &str, encoding: PromptLiteralEncoding) {
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

    let _ = write!(
        prompt,
        "%F{{{branch_color}}}{}",
        escape_prompt_literal(branch, encoding)
    );
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

    let color = if keymap == "vicmd" {
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
            elapsed
                .parse::<i64>()
                .ok()
                .and_then(|value| u64::try_from(value).ok())
                .unwrap_or(0)
        },
    )
}

/// `1`, `true`, `yes`, and `on` hide the name; every other value safely leaves it visible.
fn git_user_name_is_hidden(value: &str) -> bool {
    ["1", "true", "yes", "on"]
        .iter()
        .any(|enabled| value.eq_ignore_ascii_case(enabled))
}

fn append_git_user_name(
    prompt: &mut String,
    deserialized: &Prompt,
    encoding: PromptLiteralEncoding,
) {
    if !git_user_name_is_hidden(&get_env_var("SLICK_PROMPT_NO_GIT_UNAME"))
        && !deserialized.u_name.is_empty()
    {
        let _ = write!(
            prompt,
            "%F{{{}}}{} ",
            get_env("SLICK_PROMPT_GIT_UNAME_COLOR"),
            escape_prompt_literal(&deserialized.u_name, encoding)
        );
    }
}

fn append_git_metadata(
    prompt: &mut String,
    deserialized: &Prompt,
    encoding: PromptLiteralEncoding,
) {
    if !deserialized.branch.is_empty() {
        append_branch(prompt, &deserialized.branch, encoding);
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
    let encoding = current_prompt_literal_encoding();

    append_cursor_shape(&mut prompt, keymap);
    append_identity_prefix(&mut prompt, is_root_user, is_remote_user);

    if !transient_timestamp.is_empty() {
        let _ = write!(
            prompt,
            "%F{{{TRANSIENT_TIMESTAMP_COLOR}}}{transient_timestamp} "
        );
    }

    append_context_markers(&mut prompt, encoding);
    let path_symbol = current_path_symbol(encoding);
    let _ = write!(
        prompt,
        "%F{{{}}}{path_symbol}",
        get_env("SLICK_PROMPT_PATH_COLOR")
    );

    if !deserialized.branch.is_empty() {
        prompt.push(' ');
        append_branch(&mut prompt, &deserialized.branch, encoding);
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
    let encoding = current_prompt_literal_encoding();

    append_cursor_shape(&mut prompt, keymap);
    append_identity_prefix(&mut prompt, is_root_user, is_remote_user);

    append_context_markers(&mut prompt, encoding);
    append_git_user_name(&mut prompt, deserialized, encoding);

    let path_symbol = current_path_symbol(encoding);
    let _ = write!(
        prompt,
        "%F{{{}}}{path_symbol} ",
        get_env("SLICK_PROMPT_PATH_COLOR")
    );

    append_git_metadata(&mut prompt, deserialized, encoding);
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
    use super::{
        PromptLiteralEncoding, append_branch, compact_path, compact_path_segments,
        escape_prompt_literal, git_user_name_is_hidden, prompt_literal_encoding,
    };
    use std::path::Path;

    #[test]
    fn test_escape_prompt_literal() {
        let psvar = PromptLiteralEncoding::Psvar {
            dollar: 11,
            backtick: 12,
            backslash: 13,
        };
        let cases = [
            ("plain text", "plain text", "plain text"),
            ("%n", "%%n", "%%n"),
            ("$(id)", r"\$(id)", "%11v(id)"),
            ("`id`", r"\`id\`", "%12vid%12v"),
            (r"\$(id)", r"\\\$(id)", "%13v%11v(id)"),
            (
                r"before %n $(id) `id` \ after",
                r"before %%n \$(id) \`id\` \\ after",
                r"before %%n %11v(id) %12vid%12v %13v after",
            ),
        ];

        for (input, direct, canonical) in cases {
            assert_eq!(
                escape_prompt_literal(input, PromptLiteralEncoding::Backslash),
                direct,
                "direct caller, input: {input:?}"
            );
            assert_eq!(
                escape_prompt_literal(input, psvar),
                canonical,
                "canonical loader, input: {input:?}"
            );
        }
    }

    #[test]
    fn test_prompt_literal_encoding_defaults_safely() {
        let cases = [
            ("", "", "", PromptLiteralEncoding::Backslash),
            ("1", "2", "", PromptLiteralEncoding::Backslash),
            ("1", "invalid", "3", PromptLiteralEncoding::Backslash),
            (
                "11",
                "12",
                "13",
                PromptLiteralEncoding::Psvar {
                    dollar: 11,
                    backtick: 12,
                    backslash: 13,
                },
            ),
        ];

        for (dollar, backtick, backslash, expected) in cases {
            assert_eq!(
                prompt_literal_encoding(dollar, backtick, backslash),
                expected,
                "values: {dollar:?}, {backtick:?}, {backslash:?}"
            );
        }
    }

    #[test]
    fn test_compact_path_segments_escape_prompt_syntax() {
        let psvar = PromptLiteralEncoding::Psvar {
            dollar: 11,
            backtick: 12,
            backslash: 13,
        };
        assert_eq!(
            compact_path_segments(
                ["%parent", r"$(id)%n`id`\"].into_iter(),
                PromptLiteralEncoding::Backslash,
            ),
            r"%%/\$(id)%%n\`id\`\\"
        );
        assert_eq!(
            compact_path_segments(["%parent", r"$(id)%n`id`\"].into_iter(), psvar),
            r"%%/%11v(id)%%n%12vid%12v%13v"
        );
    }

    #[test]
    fn test_git_user_name_hidden_boolean_values() {
        let cases = [
            ("1", true),
            ("true", true),
            ("TRUE", true),
            ("YeS", true),
            ("oN", true),
            ("0", false),
            ("false", false),
            ("NO", false),
            ("Off", false),
            ("", false),
            ("invalid", false),
        ];

        for (value, expected) in cases {
            assert_eq!(git_user_name_is_hidden(value), expected, "value: {value:?}");
        }
    }

    #[test]
    fn test_append_branch_uses_separate_symbol_color() {
        let mut prompt = String::new();
        append_branch(&mut prompt, "main", PromptLiteralEncoding::Backslash);
        assert_eq!(prompt, "%F{2} %F{160}main");
    }

    #[test]
    fn test_compact_path_for_home_nested_path() {
        let path = Path::new("/var/home/nbari/projects/rust/slick");
        let home = Path::new("/var/home/nbari");

        assert_eq!(
            compact_path(path, Some(home), PromptLiteralEncoding::Backslash),
            "~/p/r/slick"
        );
    }

    #[test]
    fn test_compact_path_for_absolute_path_outside_home() {
        let path = Path::new("/var/home/nbari/projects/rust/slick");
        let home = Path::new("/tmp/home");

        assert_eq!(
            compact_path(path, Some(home), PromptLiteralEncoding::Backslash),
            "/v/h/n/p/r/slick"
        );
    }

    #[test]
    fn test_compact_path_for_home_root() {
        let path = Path::new("/var/home/nbari");
        let home = Path::new("/var/home/nbari");

        assert_eq!(
            compact_path(path, Some(home), PromptLiteralEncoding::Backslash),
            "~"
        );
    }
}
