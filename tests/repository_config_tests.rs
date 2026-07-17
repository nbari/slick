const CARGO_MANIFEST: &str = include_str!("../Cargo.toml");
const CARGO_LOCK: &str = include_str!("../Cargo.lock");
const LOAD_ZSH: &str = include_str!("../load.zsh");
const README: &str = include_str!("../README.md");
const SLICK_ZSH: &str = include_str!("../slick.zsh");
const TEST_INTERACTIVE_ZSH: &str = include_str!("../test_interactive.zsh");
const TEST_WORKFLOW: &str = include_str!("../.github/workflows/test.yml");
const COVERAGE_WORKFLOW: &str = include_str!("../.github/workflows/coverage.yml");
const BUILD_WORKFLOW: &str = include_str!("../.github/workflows/build.yml");
const RELEASE_WORKFLOW: &str = include_str!("../.github/workflows/release.yml");

const ZLIB_URL: &str = "https://github.com/madler/zlib/releases/download/v1.3.1/zlib-1.3.1.tar.gz";
const CURL_RETRIES: &str = "--retry 3 --retry-all-errors";

fn workflow_job<'a>(workflow: &'a str, name: &str, next_name: Option<&str>) -> &'a str {
    let Some((_, job_and_rest)) = workflow.split_once(name) else {
        return "";
    };

    next_name.map_or(job_and_rest, |next| {
        job_and_rest
            .split_once(next)
            .map_or(job_and_rest, |(job, _)| job)
    })
}

#[test]
fn test_shell_regressions_run_in_ci() {
    assert!(TEST_WORKFLOW.contains("run: ./test.sh"));
    assert!(TEST_WORKFLOW.contains("sudo apt-get install -y jq zsh"));
}

#[test]
fn test_workflows_use_current_action_majors() {
    for workflow in [
        TEST_WORKFLOW,
        COVERAGE_WORKFLOW,
        BUILD_WORKFLOW,
        RELEASE_WORKFLOW,
    ] {
        for line in workflow.lines() {
            let Some((_, action)) = line.split_once("uses:") else {
                continue;
            };
            let action = action.trim();
            if action.starts_with("./") {
                continue;
            }

            assert!(
                matches!(
                    action,
                    "actions/checkout@v7"
                        | "dtolnay/rust-toolchain@stable"
                        | "codecov/codecov-action@v7"
                        | "coverallsapp/github-action@v2"
                        | "softprops/action-gh-release@v3"
                ),
                "unexpected or outdated action reference: {action}"
            );
        }
    }
}

#[test]
fn test_zsh_entrypoints_use_portable_shebangs() {
    for script in [SLICK_ZSH, LOAD_ZSH, TEST_INTERACTIVE_ZSH] {
        assert!(script.starts_with("#!/usr/bin/env zsh\n"));
    }
}

#[test]
fn test_platform_cargo_test_jobs_run_zsh_regressions() {
    let matrix = workflow_job(TEST_WORKFLOW, "\n  test:", None);

    assert!(matrix.contains("- ubuntu-latest"));
    assert!(matrix.lines().any(|line| line.trim() == "- macos-15"));
    assert!(matrix.lines().any(|line| line.trim() == "- macos-15-intel"));

    let before_test = matrix
        .split_once("run: cargo test")
        .map_or("", |(before, _)| before);
    assert!(before_test.contains("if: runner.os == 'Linux'"));
    assert!(before_test.contains("sudo apt-get install -y zsh"));
    assert!(matrix.contains("run: cargo build --release --locked"));
    assert!(matrix.contains(r#"zsh -dfi -c "source tests/slick_zsh_regression_test.zsh""#));
    assert!(matrix.contains(r#"zsh -dfi -c "source tests/load_zsh_regression_test.zsh""#));

    let before_coverage_test = COVERAGE_WORKFLOW
        .split_once("run: cargo test")
        .map_or("", |(before, _)| before);
    assert!(COVERAGE_WORKFLOW.contains("runs-on: ubuntu-latest"));
    assert!(before_coverage_test.contains("sudo apt-get install -y zsh"));
}

#[test]
fn test_git2_does_not_enable_openssl() {
    let git2_dependency = CARGO_MANIFEST
        .lines()
        .find(|line| line.starts_with("git2 = "))
        .unwrap_or_default();

    assert!(git2_dependency.contains("default-features = false"));
    assert!(git2_dependency.contains(r#"features = ["vendored-libgit2"]"#));
    assert!(!git2_dependency.contains("https"));
    assert!(!git2_dependency.contains("openssl"));
    assert!(!README.contains("libssl-dev"));

    for package in ["openssl", "openssl-sys", "openssl-probe"] {
        let package_header = format!(r#"name = "{package}""#);
        assert!(
            !CARGO_LOCK.lines().any(|line| line == package_header),
            "{package} must not be present in the resolved dependency graph"
        );
    }
}

#[test]
fn test_musl_workflows_use_reliable_zlib_download() {
    for workflow in [BUILD_WORKFLOW, RELEASE_WORKFLOW] {
        assert!(workflow.contains(ZLIB_URL));
        assert!(workflow.contains(CURL_RETRIES));
    }
}
