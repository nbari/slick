use crate::{get_env, get_env_var, get_env_var_or};
use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
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

// check if current user is root or not
fn is_root() -> bool {
    get_user_by_uid(get_current_uid()).is_some_and(|user| user.uid() == 0)
}

// check if current user is remote or not
fn is_remote() -> bool {
    env::var("SSH_CONNECTION").is_ok()
}

fn toolbox_env_path() -> PathBuf {
    PathBuf::from(get_env_var_or(
        "SLICK_TEST_TOOLBOXENV_PATH",
        "/run/.toolboxenv",
    ))
}

fn container_env_path() -> PathBuf {
    PathBuf::from(get_env_var_or(
        "SLICK_TEST_CONTAINERENV_PATH",
        "/run/.containerenv",
    ))
}

fn parse_toolbox_name(containerenv: &str) -> Option<String> {
    containerenv.lines().find_map(|line| {
        line.strip_prefix("name=\"")
            .and_then(|name| name.strip_suffix('"'))
            .filter(|name| !name.is_empty())
            .map(str::to_owned)
    })
}

fn get_toolbox_name_from_paths(toolboxenv_path: &Path, containerenv_path: &Path) -> Option<String> {
    if !toolboxenv_path.exists() {
        return None;
    }

    let containerenv = fs::read_to_string(containerenv_path).ok()?;
    parse_toolbox_name(&containerenv)
}

fn get_toolbox_name() -> Option<String> {
    get_toolbox_name_from_paths(&toolbox_env_path(), &container_env_path())
}

fn get_devpod_name_from_env_vars(devpod: &str, workspace_id: &str) -> Option<String> {
    if devpod.is_empty() {
        return None;
    }

    if workspace_id.is_empty() {
        Some("devpod".to_string())
    } else {
        Some(workspace_id.to_string())
    }
}

fn get_devpod_name() -> Option<String> {
    get_devpod_name_from_env_vars(&get_env_var("DEVPOD"), &get_env_var("DEVPOD_WORKSPACE_ID"))
}

fn format_context_marker(symbol: &str, name: &str) -> String {
    if symbol.is_empty() {
        format!("({name})")
    } else {
        format!("({symbol} {name})")
    }
}

fn format_branch_marker(symbol: &str, branch: &str) -> String {
    if symbol.is_empty() {
        branch.to_string()
    } else {
        format!("{symbol} {branch}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PythonEnvSource {
    VirtualEnv { pipenv_active: bool },
    Pyenv,
}

fn strip_pipenv_hash_suffix(name: &str) -> &str {
    let Some((prefix, suffix)) = name.rsplit_once('-') else {
        return name;
    };

    if suffix.len() == 8 && suffix.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        prefix
    } else {
        name
    }
}

fn parse_virtual_env_name(virtual_env: &str, pipenv_active: bool) -> Option<String> {
    let name = virtual_env
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())?;

    if pipenv_active {
        Some(strip_pipenv_hash_suffix(name).to_string())
    } else {
        Some(name.to_string())
    }
}

fn parse_pyenv_name(pyenv_version: &str) -> Option<String> {
    pyenv_version
        .split(':')
        .map(str::trim)
        .find(|version| !version.is_empty() && *version != "system")
        .map(|version| version.rsplit('/').next().unwrap_or(version).to_string())
}

fn get_python_env() -> Option<(String, PythonEnvSource)> {
    let pipenv_active = !get_env_var("PIPENV_ACTIVE").is_empty();

    let virtual_env_prompt = get_env_var("VIRTUAL_ENV_PROMPT");
    if !virtual_env_prompt.is_empty() {
        return Some((
            virtual_env_prompt,
            PythonEnvSource::VirtualEnv { pipenv_active },
        ));
    }

    let virtual_env = get_env_var("VIRTUAL_ENV");
    if !virtual_env.is_empty() {
        return parse_virtual_env_name(&virtual_env, pipenv_active)
            .map(|name| (name, PythonEnvSource::VirtualEnv { pipenv_active }));
    }

    parse_pyenv_name(&get_env_var("PYENV_VERSION")).map(|name| (name, PythonEnvSource::Pyenv))
}

fn get_python_env_color(source: PythonEnvSource) -> String {
    let python_env_color = get_env_var("SLICK_PROMPT_PYTHON_ENV_COLOR");
    if !python_env_color.is_empty() {
        return python_env_color;
    }

    match source {
        PythonEnvSource::VirtualEnv {
            pipenv_active: true,
        } => get_env_var_or(
            "PIPENV_ACTIVE_COLOR",
            get_env("SLICK_PROMPT_PYTHON_ENV_COLOR"),
        ),
        PythonEnvSource::VirtualEnv {
            pipenv_active: false,
        }
        | PythonEnvSource::Pyenv => get_env("SLICK_PROMPT_PYTHON_ENV_COLOR").to_string(),
    }
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
    if let Some(toolbox_name) = get_toolbox_name() {
        let _ = write!(
            prompt,
            "%F{{{}}}{} ",
            get_env("SLICK_PROMPT_TOOLBOX_COLOR"),
            format_context_marker(get_env("SLICK_PROMPT_TOOLBOX_SYMBOL"), &toolbox_name)
        );
    }

    if let Some(devpod_name) = get_devpod_name() {
        let _ = write!(
            prompt,
            "%F{{{}}}{} ",
            get_env("SLICK_PROMPT_DEVPOD_COLOR"),
            format_context_marker(get_env("SLICK_PROMPT_DEVPOD_SYMBOL"), &devpod_name)
        );
    }

    if let Some((python_env, source)) = get_python_env() {
        let _ = write!(
            prompt,
            "%F{{{}}}({}) ",
            get_python_env_color(source),
            python_env
        );
    }
}

fn append_branch(prompt: &mut String, branch: &str) {
    if branch.is_empty() {
        return;
    }

    let branch_marker = format_branch_marker(get_env("SLICK_PROMPT_GIT_BRANCH_SYMBOL"), branch);
    let branch_color = if branch == "master" || branch == "main" {
        get_env("SLICK_PROMPT_GIT_MASTER_BRANCH_COLOR")
    } else {
        get_env("SLICK_PROMPT_GIT_BRANCH_COLOR")
    };

    let _ = write!(prompt, "%F{{{branch_color}}}{branch_marker}");
}

fn build_transient_prompt(
    deserialized: &Prompt,
    is_root_user: bool,
    is_remote_user: bool,
    symbol: &str,
    prompt_symbol_color: &str,
    transient_timestamp: &str,
) -> String {
    let mut prompt = String::with_capacity(256);

    append_identity_prefix(&mut prompt, is_root_user, is_remote_user);

    if !transient_timestamp.is_empty() {
        let _ = write!(
            prompt,
            "%F{{{TRANSIENT_TIMESTAMP_COLOR}}}{transient_timestamp} "
        );
    }

    append_context_markers(&mut prompt);

    let _ = write!(prompt, "%F{{{}}}%~", get_env("SLICK_PROMPT_PATH_COLOR"));

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

#[allow(clippy::too_many_lines)]
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

    if transient {
        print!(
            "{}",
            build_transient_prompt(
                &deserialized,
                is_root_user,
                is_remote_user,
                symbol,
                prompt_symbol_color,
                transient_timestamp,
            )
        );
        return;
    }

    // get time elapsed
    // Prefer -e (elapsed) if provided (pre-calculated in zsh to avoid flickering)
    // Fallback to -t (timestamp) for backwards compatibility
    let time_elapsed: u64 = matches.get_one::<String>("elapsed").map_or_else(
        || {
            // Fallback: calculate from timestamp (legacy behavior)
            let epochtime: u64 = matches
                .get_one("time")
                .map_or(String::new(), String::clone)
                .parse::<u64>()
                .ok()
                .unwrap_or_else(|| {
                    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                        Ok(n) => n.as_secs(),
                        Err(e) => {
                            eprintln!("SystemTime before UNIX EPOCH!: {e}");
                            exit(1)
                        }
                    }
                });

            let d = SystemTime::UNIX_EPOCH + Duration::from_secs(epochtime);
            d.elapsed().map_or(0, |elapsed| elapsed.as_secs())
        },
        |elapsed_str| {
            // Parse as i64 first to handle negative values, then convert to u64
            // Negative values (from clock adjustments) are clamped to 0
            elapsed_str
                .parse::<i64>()
                .ok()
                .map_or(0, |val| if val < 0 { 0 } else { val.cast_unsigned() })
        },
    );

    // Use String builder instead of Vec for better performance
    // Estimate capacity: ~200 chars is typical for a prompt
    let mut prompt = String::with_capacity(256);

    append_identity_prefix(&mut prompt, is_root_user, is_remote_user);
    append_context_markers(&mut prompt);

    // git u_name (before path for consistency with zpty single-render mode)
    if get_env_var("SLICK_PROMPT_NO_GIT_UNAME").is_empty() && !deserialized.u_name.is_empty() {
        let _ = write!(
            prompt,
            "%F{{{}}}{}",
            get_env("SLICK_PROMPT_GIT_UNAME_COLOR"),
            deserialized.u_name
        );
        prompt.push(' ');
    }

    // current dir %~ (after u_name)
    let _ = write!(prompt, "%F{{{}}}%~ ", get_env("SLICK_PROMPT_PATH_COLOR"));

    // branch
    if !deserialized.branch.is_empty() {
        append_branch(&mut prompt, &deserialized.branch);
        prompt.push(' ');
    }

    // git status
    if !deserialized.status.is_empty() {
        let _ = write!(
            prompt,
            "%F{{{}}}[{}]",
            get_env("SLICK_PROMPT_GIT_STATUS_COLOR"),
            deserialized.status
        );
        prompt.push(' ');
    }

    // git remote
    if !deserialized.remote.is_empty() {
        let _ = write!(
            prompt,
            "%F{{{}}}{}",
            get_env("SLICK_PROMPT_GIT_REMOTE_COLOR"),
            deserialized.remote.join(" ")
        );
        prompt.push(' ');
    }

    // git action
    if !deserialized.action.is_empty() {
        let _ = write!(
            prompt,
            "%F{{{}}}{}",
            get_env("SLICK_PROMPT_GIT_ACTION_COLOR"),
            deserialized.action
        );
        prompt.push(' ');
    }

    // git staged
    if deserialized.staged {
        let _ = write!(
            prompt,
            "%F{{{}}}[staged]",
            get_env("SLICK_PROMPT_GIT_STAGED_COLOR"),
        );
        prompt.push(' ');
    }

    // authentication failed warning
    if deserialized.auth_failed {
        let _ = write!(
            prompt,
            "%F{{{}}}{}",
            get_env("SLICK_PROMPT_GIT_AUTH_COLOR"),
            get_env("SLICK_PROMPT_GIT_AUTH_SYMBOL")
        );
        prompt.push(' ');
    }

    // time elapsed
    let max_time: u64 = get_env("SLICK_PROMPT_CMD_MAX_EXEC_TIME")
        .parse()
        .unwrap_or(5);
    if time_elapsed > max_time {
        let _ = write!(
            prompt,
            "%F{{{}}}{}",
            get_env("SLICK_PROMPT_TIME_ELAPSED_COLOR"),
            compound_duration::format_dhms(time_elapsed)
        );
        prompt.push(' ');
    }

    // Remove trailing space if present
    if prompt.ends_with(' ') {
        prompt.pop();
    }

    // second prompt line
    let _ = write!(
        prompt,
        "\n%F{{{}}}{}%f{}",
        prompt_symbol_color,
        symbol,
        get_env("SLICK_PROMPT_NON_BREAKING_SPACE"),
    );

    print!("{prompt}");
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{
        format_branch_marker, format_context_marker, get_devpod_name_from_env_vars,
        get_toolbox_name_from_paths, parse_pyenv_name, parse_toolbox_name, parse_virtual_env_name,
        strip_pipenv_hash_suffix,
    };
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_parse_toolbox_name_returns_container_name() {
        let containerenv = "engine=\"podman\"\nname=\"codex\"\nid=\"abc\"\n";
        assert_eq!(parse_toolbox_name(containerenv), Some("codex".to_string()));
    }

    #[test]
    fn test_parse_toolbox_name_returns_none_when_name_is_missing() {
        let containerenv = "engine=\"podman\"\nimage=\"fedora-toolbox:43\"\n";
        assert_eq!(parse_toolbox_name(containerenv), None);
    }

    #[test]
    fn test_get_toolbox_name_from_paths_requires_toolboxenv_file() {
        let tempdir = tempdir().expect("tempdir should be created");
        let containerenv_path = tempdir.path().join(".containerenv");
        fs::write(&containerenv_path, "name=\"codex\"\n").expect("containerenv should be written");

        let missing_toolboxenv = tempdir.path().join(".toolboxenv");
        assert_eq!(
            get_toolbox_name_from_paths(&missing_toolboxenv, &containerenv_path),
            None
        );
    }

    #[test]
    fn test_get_toolbox_name_from_paths_returns_name_for_toolbox() {
        let tempdir = tempdir().expect("tempdir should be created");
        let toolboxenv_path = tempdir.path().join(".toolboxenv");
        let containerenv_path = tempdir.path().join(".containerenv");

        fs::write(&toolboxenv_path, "").expect("toolboxenv should be written");
        fs::write(&containerenv_path, "engine=\"podman\"\nname=\"codex\"\n")
            .expect("containerenv should be written");

        assert_eq!(
            get_toolbox_name_from_paths(&toolboxenv_path, &containerenv_path),
            Some("codex".to_string())
        );
    }

    #[test]
    fn test_format_context_marker_with_symbol() {
        assert_eq!(format_context_marker("▣", "codex"), "(▣ codex)");
    }

    #[test]
    fn test_format_context_marker_without_symbol() {
        assert_eq!(format_context_marker("", "codex"), "(codex)");
    }

    #[test]
    fn test_format_branch_marker_with_symbol() {
        assert_eq!(format_branch_marker("", "main"), " main");
    }

    #[test]
    fn test_format_branch_marker_without_symbol() {
        assert_eq!(format_branch_marker("", "main"), "main");
    }

    #[test]
    fn test_get_devpod_name_returns_workspace_id() {
        assert_eq!(
            get_devpod_name_from_env_vars("true", "hfile"),
            Some("hfile".to_string())
        );
    }

    #[test]
    fn test_get_devpod_name_falls_back_to_devpod() {
        assert_eq!(
            get_devpod_name_from_env_vars("true", ""),
            Some("devpod".to_string())
        );
    }

    #[test]
    fn test_get_devpod_name_returns_none_when_unset() {
        assert_eq!(get_devpod_name_from_env_vars("", "hfile"), None);
    }

    #[test]
    fn test_parse_virtual_env_name_returns_last_path_segment() {
        assert_eq!(
            parse_virtual_env_name("/tmp/venvs/project", false),
            Some("project".to_string())
        );
    }

    #[test]
    fn test_parse_virtual_env_name_strips_pipenv_hash() {
        assert_eq!(
            parse_virtual_env_name("/tmp/venvs/project-a1b2c3d4", true),
            Some("project".to_string())
        );
    }

    #[test]
    fn test_strip_pipenv_hash_suffix_preserves_internal_hyphens() {
        assert_eq!(strip_pipenv_hash_suffix("my-app-a1b2c3d4"), "my-app");
    }

    #[test]
    fn test_strip_pipenv_hash_suffix_keeps_custom_names() {
        assert_eq!(strip_pipenv_hash_suffix("custom-env"), "custom-env");
    }

    #[test]
    fn test_parse_virtual_env_name_keeps_non_pipenv_hyphenated_name() {
        assert_eq!(
            parse_virtual_env_name("/tmp/venvs/custom-env", false),
            Some("custom-env".to_string())
        );
    }

    #[test]
    fn test_parse_pyenv_name_handles_virtualenv_style_names() {
        assert_eq!(
            parse_pyenv_name("3.12.1/envs/project"),
            Some("project".to_string())
        );
    }

    #[test]
    fn test_parse_pyenv_name_handles_multiple_versions() {
        assert_eq!(
            parse_pyenv_name("3.12.1:system"),
            Some("3.12.1".to_string())
        );
    }

    #[test]
    fn test_parse_pyenv_name_skips_system_when_real_env_exists() {
        assert_eq!(
            parse_pyenv_name("system:3.12.1/envs/project"),
            Some("project".to_string())
        );
    }

    #[test]
    fn test_parse_pyenv_name_returns_none_for_system_only() {
        assert_eq!(parse_pyenv_name("system"), None);
    }

    #[test]
    fn test_parse_pyenv_name_ignores_empty_entries_and_whitespace() {
        assert_eq!(
            parse_pyenv_name(" : system : 3.11.8 "),
            Some("3.11.8".to_string())
        );
    }
}
