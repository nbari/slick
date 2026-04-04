use crate::{get_env, get_env_var, get_env_var_or};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextMarker {
    pub color: String,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonEnvSource {
    VirtualEnv { pipenv_active: bool },
    Pyenv,
}

#[must_use]
pub fn format_context_marker(symbol: &str, name: &str) -> String {
    if symbol.is_empty() {
        format!("({name})")
    } else {
        format!("({symbol} {name})")
    }
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

#[must_use]
pub fn get_python_env_color(source: PythonEnvSource) -> String {
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

fn get_aws_label_from_env_vars(
    profile: &str,
    region: &str,
    default_region: &str,
    access_key: &str,
    secret_key: &str,
    session_token: &str,
) -> Option<String> {
    if !profile.is_empty() {
        return Some(format!("aws {profile}"));
    }

    if !region.is_empty() {
        return Some(format!("aws {region}"));
    }

    if !default_region.is_empty() {
        return Some(format!("aws {default_region}"));
    }

    if !access_key.is_empty() || !secret_key.is_empty() || !session_token.is_empty() {
        return Some("aws".to_string());
    }

    None
}

fn get_aws_label() -> Option<String> {
    get_aws_label_from_env_vars(
        &get_env_var("AWS_PROFILE"),
        &get_env_var("AWS_REGION"),
        &get_env_var("AWS_DEFAULT_REGION"),
        &get_env_var("AWS_ACCESS_KEY_ID"),
        &get_env_var("AWS_SECRET_ACCESS_KEY"),
        &get_env_var("AWS_SESSION_TOKEN"),
    )
}

fn get_k8s_label_from_kubeconfig(kubeconfig: &str) -> Option<String> {
    if kubeconfig.is_empty() {
        return None;
    }

    let first_path = kubeconfig.split(':').next().unwrap_or_default().trim();
    if first_path.is_empty() {
        return Some("k8s".to_string());
    }

    let name = Path::new(first_path)
        .file_name()
        .and_then(|segment| segment.to_str())
        .filter(|segment| !segment.is_empty());

    Some(name.map_or_else(|| "k8s".to_string(), |segment| format!("k8s {segment}")))
}

fn get_k8s_label() -> Option<String> {
    get_k8s_label_from_kubeconfig(&get_env_var("KUBECONFIG"))
}

#[must_use]
pub fn collect_context_markers() -> Vec<ContextMarker> {
    let mut markers = Vec::with_capacity(5);

    if let Some(toolbox_name) = get_toolbox_name() {
        markers.push(ContextMarker {
            color: get_env("SLICK_PROMPT_TOOLBOX_COLOR").to_string(),
            text: format_context_marker(get_env("SLICK_PROMPT_TOOLBOX_SYMBOL"), &toolbox_name),
        });
    }

    if let Some(devpod_name) = get_devpod_name() {
        markers.push(ContextMarker {
            color: get_env("SLICK_PROMPT_DEVPOD_COLOR").to_string(),
            text: format_context_marker(get_env("SLICK_PROMPT_DEVPOD_SYMBOL"), &devpod_name),
        });
    }

    if let Some(aws_label) = get_aws_label() {
        markers.push(ContextMarker {
            color: get_env("SLICK_PROMPT_AWS_COLOR").to_string(),
            text: format_context_marker("", &aws_label),
        });
    }

    if let Some(k8s_label) = get_k8s_label() {
        markers.push(ContextMarker {
            color: get_env("SLICK_PROMPT_K8S_COLOR").to_string(),
            text: format_context_marker("", &k8s_label),
        });
    }

    if let Some((python_env, source)) = get_python_env() {
        markers.push(ContextMarker {
            color: get_python_env_color(source),
            text: format!("({python_env})"),
        });
    }

    markers
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{
        format_context_marker, get_aws_label_from_env_vars, get_devpod_name_from_env_vars,
        get_k8s_label_from_kubeconfig, get_toolbox_name_from_paths, parse_pyenv_name,
        parse_toolbox_name, parse_virtual_env_name, strip_pipenv_hash_suffix,
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

    #[test]
    fn test_get_aws_label_prefers_profile() {
        assert_eq!(
            get_aws_label_from_env_vars("prod", "eu-central-1", "", "", "", ""),
            Some("aws prod".to_string())
        );
    }

    #[test]
    fn test_get_aws_label_falls_back_to_region() {
        assert_eq!(
            get_aws_label_from_env_vars("", "eu-central-1", "", "", "", ""),
            Some("aws eu-central-1".to_string())
        );
    }

    #[test]
    fn test_get_aws_label_falls_back_to_default_region() {
        assert_eq!(
            get_aws_label_from_env_vars("", "", "us-east-1", "", "", ""),
            Some("aws us-east-1".to_string())
        );
    }

    #[test]
    fn test_get_aws_label_falls_back_to_generic_marker() {
        assert_eq!(
            get_aws_label_from_env_vars("", "", "", "AKIA...", "secret", ""),
            Some("aws".to_string())
        );
    }

    #[test]
    fn test_get_aws_label_returns_none_when_unset() {
        assert_eq!(get_aws_label_from_env_vars("", "", "", "", "", ""), None);
    }

    #[test]
    fn test_get_k8s_label_uses_first_kubeconfig_basename() {
        assert_eq!(
            get_k8s_label_from_kubeconfig("/tmp/dev:/tmp/prod"),
            Some("k8s dev".to_string())
        );
    }

    #[test]
    fn test_get_k8s_label_falls_back_when_first_entry_is_empty() {
        assert_eq!(
            get_k8s_label_from_kubeconfig(":/tmp/prod"),
            Some("k8s".to_string())
        );
    }

    #[test]
    fn test_get_k8s_label_falls_back_when_basename_is_missing() {
        assert_eq!(get_k8s_label_from_kubeconfig("/"), Some("k8s".to_string()));
    }

    #[test]
    fn test_get_k8s_label_returns_none_when_unset() {
        assert_eq!(get_k8s_label_from_kubeconfig(""), None);
    }
}
