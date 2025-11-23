use crate::get_env;
use crate::git;
use git2::Repository;
use std::{
    env,
    fs, // Added back std::fs
    io::{self, Write},
    thread::sleep,
    time::Duration,
};
use tokio::{spawn, task::spawn_blocking, time::timeout};

pub async fn render() {
    // Check if we're in a git repository
    let repo_result = env::current_dir()
        .ok()
        .and_then(|path| Repository::discover(path).ok());

    if let Some(repo) = repo_result {
        // Inside git repo: Output git info in 2 phases
        // Phase 1: Output all fast/local git info immediately (no blocking)
        let mut prompt = git::build_prompt_fast(&repo);

        if let Ok(serialized) = serde_json::to_string(&prompt) {
            // Ignore broken pipe errors (happens when zsh closes the pipe early)
            let _ = writeln!(io::stdout(), "{serialized}");
            // Force flush to ensure immediate output before Phase 2 starts
            let _ = io::stdout().flush();
        }

        // Phase 2a: Spawn blocking task for slow git status (CPU-bound)
        let repo_path = repo.path().to_path_buf();
        let status_handle = spawn_blocking(move || -> Option<String> {
            // TEST: Simulate slow git status (for testing non-blocking behavior)
            // Set SLICK_TEST_DELAY=N to add N seconds delay (e.g., SLICK_TEST_DELAY=1)
            // Note: Using thread::sleep here (not tokio::time::sleep) because spawn_blocking
            // runs in a blocking thread pool where synchronous sleep is appropriate
            if let Ok(delay_str) = env::var("SLICK_TEST_DELAY")
                && let Ok(delay_secs) = delay_str.parse::<u64>()
                && delay_secs > 0
            {
                sleep(Duration::from_secs(delay_secs));
            }

            // Re-open repository in the blocking thread pool
            if let Ok(repo) = Repository::open(&repo_path)
                && let Ok(status) = git::get_status(&repo)
            {
                return Some(status);
            }
            None
        });

        // Phase 2b: Async git fetch with auth detection and cache update
        // This spawns a tokio task that checks auth status and updates cache
        let fetch_handle = if get_env("SLICK_PROMPT_GIT_FETCH") == "0" {
            None
        } else {
            let cache_path = git::get_auth_cache_path(&repo);

            Some(spawn(async move {
                // Create cache directory if cache path exists
                if let Some(ref cache) = cache_path
                    && let Some(parent) = cache.parent()
                {
                    let _ = fs::create_dir_all(parent);
                }

                let mut cmd = tokio::process::Command::new("git");
                cmd.env("GIT_TERMINAL_PROMPT", "0")
                    .env(
                        "GIT_SSH_COMMAND",
                        "ssh -o BatchMode=yes -o ControlMaster=no",
                    )
                    .env("GIT_ASKPASS", "true")
                    .arg("-c")
                    .arg("gc.auto=0")
                    .arg("fetch")
                    .arg("--quiet")
                    .arg("--no-tags")
                    .arg("--no-recurse-submodules");

                // Use tokio timeout for 5 second limit
                let result = timeout(Duration::from_secs(5), cmd.output()).await;

                // Write auth status to cache if we have a cache path
                if let Some(cache) = cache_path {
                    let now = git::unix_timestamp();

                    let auth_failed = match result {
                        Ok(Ok(output)) => {
                            if output.status.success() {
                                false
                            } else {
                                // Check stderr for auth-related errors (convert to lowercase once)
                                let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
                                stderr.contains("permission denied")
                                    || stderr.contains("authentication failed")
                                    || stderr.contains("could not read")
                                    || stderr.contains("repository not found")
                                    || stderr.contains("access denied")
                            }
                        }
                        Ok(Err(_)) | Err(_) => {
                            // Command spawn/execution error or timeout - don't update cache
                            // This could be a transient issue, keep existing cache value
                            return;
                        }
                    };

                    let status = if auth_failed { "1" } else { "0" };
                    let _ = fs::write(cache, format!("{now}:{status}"));
                }
            }))
        };

        // Wait for git status (fast ~10-50ms), output immediately
        if let Some(status) = status_handle.await.ok().flatten() {
            prompt.status = status;
            if let Ok(serialized) = serde_json::to_string(&prompt) {
                let _ = writeln!(io::stdout(), "{serialized}");
                let _ = io::stdout().flush();
            }
        }

        // Give fetch task a grace period to complete and write cache
        // Balance between fast prompt and cache write completion
        if let Some(handle) = fetch_handle {
            // Wait up to 500ms for fetch task to complete and write cache
            // Most fetches complete within this time, even with auth failures
            // Slower fetches will update cache on subsequent prompts (eventual consistency)
            let _ = timeout(Duration::from_millis(500), handle).await;
        }
    } else {
        // Outside git repo: Output empty prompt data (ensures handler fires for elapsed time)
        let prompt = git::Prompt::default();
        if let Ok(serialized) = serde_json::to_string(&prompt) {
            let _ = writeln!(io::stdout(), "{serialized}");
            let _ = io::stdout().flush();
        }
    }
}
