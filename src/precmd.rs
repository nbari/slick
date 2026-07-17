use crate::get_env;
use crate::git;
use git2::Repository;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::{
    env,
    ffi::OsStr,
    fs,
    io::{self, Write},
    path::Path,
    process::{Command as StdCommand, Output, Stdio},
    thread::sleep,
    time::Duration,
};
use tokio::{
    process::Command,
    spawn,
    task::{JoinHandle, spawn_blocking},
    time::timeout,
};

const GIT_FETCH_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug, Eq, PartialEq)]
enum GitFetchOutcome {
    Completed,
    SpawnFailed,
    TimedOut,
}

#[cfg(unix)]
struct ProcessGroupGuard {
    id: libc::pid_t,
    armed: bool,
}

#[cfg(unix)]
impl ProcessGroupGuard {
    const fn new(id: libc::pid_t) -> Self {
        Self { id, armed: true }
    }

    const fn disarm(&mut self) {
        self.armed = false;
    }

    fn terminate(&mut self) {
        if self.armed {
            // SAFETY: The fetch child is created as the leader of this process group.
            let _ = unsafe { libc::killpg(self.id, libc::SIGKILL) };
            self.armed = false;
        }
    }
}

#[cfg(unix)]
impl Drop for ProcessGroupGuard {
    fn drop(&mut self) {
        self.terminate();
    }
}

fn git_fetch_command(program: &OsStr, repo_path: &Path) -> Command {
    let mut command = StdCommand::new(program);
    command
        .current_dir(repo_path)
        .env("GIT_TERMINAL_PROMPT", "0")
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
    #[cfg(unix)]
    command.process_group(0);
    Command::from(command)
}

fn write_fetch_auth_cache(cache: &Path, output: &Output) {
    let auth_failed = if output.status.success() {
        false
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
        stderr.contains("permission denied")
            || stderr.contains("authentication failed")
            || stderr.contains("could not read")
            || stderr.contains("repository not found")
            || stderr.contains("access denied")
    };

    let status = if auth_failed { "1" } else { "0" };
    let _ = fs::write(cache, format!("{}:{status}", git::unix_timestamp()));
}

async fn run_git_fetch(
    mut command: Command,
    cache_path: Option<&Path>,
    fetch_timeout: Duration,
) -> GitFetchOutcome {
    command
        .kill_on_drop(true)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let Ok(mut child) = command.spawn() else {
        return GitFetchOutcome::SpawnFailed;
    };

    #[cfg(unix)]
    let process_group_id = match child.id().and_then(|id| libc::pid_t::try_from(id).ok()) {
        Some(id) if id > 0 => id,
        _ => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            return GitFetchOutcome::SpawnFailed;
        }
    };

    let mut wait = Box::pin(child.wait_with_output());
    #[cfg(unix)]
    let mut process_group = ProcessGroupGuard::new(process_group_id);

    let output = match timeout(fetch_timeout, wait.as_mut()).await {
        Ok(Ok(output)) => {
            #[cfg(unix)]
            process_group.disarm();
            output
        }
        Ok(Err(_)) => return GitFetchOutcome::SpawnFailed,
        Err(_) => {
            #[cfg(unix)]
            {
                process_group.terminate();
                let _ = wait.await;
            }
            #[cfg(not(unix))]
            drop(wait);
            return GitFetchOutcome::TimedOut;
        }
    };

    if let Some(cache) = cache_path {
        write_fetch_auth_cache(cache, &output);
    }
    GitFetchOutcome::Completed
}

async fn join_git_fetch(handle: JoinHandle<GitFetchOutcome>) -> Option<GitFetchOutcome> {
    handle.await.ok()
}

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
        let fetch_handle = if matches!(
            get_env("SLICK_PROMPT_GIT_FETCH"),
            "0" | "false" | "no" | "off"
        ) {
            None
        } else {
            let cache_path = git::get_auth_cache_path(&repo);
            let fetch_path = repo.workdir().unwrap_or_else(|| repo.path()).to_path_buf();

            Some(spawn(async move {
                // Create cache directory if cache path exists
                if let Some(ref cache) = cache_path
                    && let Some(parent) = cache.parent()
                {
                    let _ = fs::create_dir_all(parent);
                }

                let command = git_fetch_command(OsStr::new("git"), &fetch_path);
                run_git_fetch(command, cache_path.as_deref(), GIT_FETCH_TIMEOUT).await
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

        // The deadline is enforced inside the task so timeout cleanup can kill and reap the child.
        if let Some(handle) = fetch_handle {
            let _ = join_git_fetch(handle).await;
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

#[cfg(all(test, unix))]
mod tests {
    use super::{
        GIT_FETCH_TIMEOUT, GitFetchOutcome, git_fetch_command, join_git_fetch, run_git_fetch,
    };
    use std::{
        error::Error,
        fs, io,
        os::unix::fs::PermissionsExt,
        path::Path,
        process::Stdio,
        time::{Duration, Instant},
    };
    use tokio::{process::Command, time::sleep};

    const POLL_INTERVAL: Duration = Duration::from_millis(10);

    fn test_directory(prefix: &str) -> io::Result<tempfile::TempDir> {
        let target_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
        fs::create_dir_all(&target_dir)?;
        tempfile::Builder::new()
            .prefix(prefix)
            .tempdir_in(target_dir)
    }

    fn write_executable(path: &Path, contents: &str) -> io::Result<()> {
        fs::write(path, contents)?;
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)
    }

    async fn wait_for_pid(path: &Path, wait: Duration) -> io::Result<u32> {
        let deadline = Instant::now() + wait;
        loop {
            match fs::read_to_string(path) {
                Ok(pid) => {
                    return pid
                        .trim()
                        .parse()
                        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error));
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }

            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "fake fetch did not record its pid",
                ));
            }
            sleep(POLL_INTERVAL).await;
        }
    }

    async fn process_exists(pid: u32) -> io::Result<bool> {
        let status = Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await?;
        Ok(status.success())
    }

    async fn wait_for_process_exit(pid: u32, wait: Duration) -> io::Result<()> {
        let deadline = Instant::now() + wait;
        loop {
            if !process_exists(pid).await? {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("fake fetch process {pid} was not terminated"),
                ));
            }
            sleep(POLL_INTERVAL).await;
        }
    }

    #[tokio::test]
    async fn test_production_fetch_deadline_kills_process_group_and_preserves_auth_cache()
    -> Result<(), Box<dyn Error>> {
        let test_dir = test_directory("slick-fetch-process-group-")?;
        let worktree = test_dir.path().join("worktree");
        fs::create_dir(&worktree)?;

        let fake_git = test_dir.path().join("fake-git");
        write_executable(
            &fake_git,
            "#!/bin/sh\n\
             pwd > \"$SLICK_FETCH_TEST_CWD_FILE\"\n\
             printf '%s\\n' \"$$\" > \"$SLICK_FETCH_TEST_PARENT_PID_FILE\"\n\
             sleep 30 &\n\
             child_pid=$!\n\
             printf '%s\\n' \"$child_pid\" > \"$SLICK_FETCH_TEST_CHILD_PID_FILE\"\n\
             wait \"$child_pid\"\n",
        )?;

        let parent_pid_path = test_dir.path().join("fetch-parent.pid");
        let child_pid_path = test_dir.path().join("fetch-child.pid");
        let cwd_path = test_dir.path().join("fetch.cwd");
        let cache_path = test_dir.path().join("auth-cache");
        fs::write(&cache_path, "123:1")?;

        let mut command = git_fetch_command(fake_git.as_os_str(), &worktree);
        command
            .env("SLICK_FETCH_TEST_PARENT_PID_FILE", &parent_pid_path)
            .env("SLICK_FETCH_TEST_CHILD_PID_FILE", &child_pid_path)
            .env("SLICK_FETCH_TEST_CWD_FILE", &cwd_path);

        let task_cache_path = cache_path.clone();
        let fetch_handle = tokio::spawn(async move {
            run_git_fetch(command, Some(&task_cache_path), GIT_FETCH_TIMEOUT).await
        });
        let (outcome, parent_pid, child_pid) = tokio::join!(
            join_git_fetch(fetch_handle),
            wait_for_pid(&parent_pid_path, Duration::from_secs(2)),
            wait_for_pid(&child_pid_path, Duration::from_secs(2))
        );
        let parent_pid = parent_pid?;
        let child_pid = child_pid?;

        assert_eq!(outcome, Some(GitFetchOutcome::TimedOut));
        tokio::try_join!(
            wait_for_process_exit(parent_pid, Duration::from_secs(5)),
            wait_for_process_exit(child_pid, Duration::from_secs(5))
        )?;
        assert_eq!(
            fs::canonicalize(fs::read_to_string(cwd_path)?.trim())?,
            fs::canonicalize(worktree)?
        );

        let cache = fs::read_to_string(cache_path)?;
        assert_eq!(cache, "123:1");
        assert!(!cache.ends_with(":0"));
        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_success_and_spawn_error_cache_behavior() -> Result<(), Box<dyn Error>> {
        let test_dir = test_directory("slick-fetch-outcomes-")?;
        let worktree = test_dir.path().join("worktree");
        fs::create_dir(&worktree)?;
        let cache_path = test_dir.path().join("auth-cache");

        let successful_git = test_dir.path().join("successful-git");
        write_executable(&successful_git, "#!/bin/sh\nexit 0\n")?;
        fs::write(&cache_path, "123:1")?;
        let outcome = run_git_fetch(
            git_fetch_command(successful_git.as_os_str(), &worktree),
            Some(&cache_path),
            Duration::from_secs(2),
        )
        .await;
        assert_eq!(outcome, GitFetchOutcome::Completed);
        assert!(fs::read_to_string(&cache_path)?.ends_with(":0"));

        fs::write(&cache_path, "123:1")?;
        let outcome = run_git_fetch(
            git_fetch_command(test_dir.path().join("missing-git").as_os_str(), &worktree),
            Some(&cache_path),
            Duration::from_secs(2),
        )
        .await;
        assert_eq!(outcome, GitFetchOutcome::SpawnFailed);
        assert_eq!(fs::read_to_string(cache_path)?, "123:1");
        Ok(())
    }
}
