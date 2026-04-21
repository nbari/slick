## 0.21.0 (2026-04-21)

### Added
- Added robust cursor-shape management by including the `DECSCUSR` escape sequence directly in the rendered prompt string. This ensures the terminal cursor is reapplied on every prompt redraw (including after exiting full-screen applications like Neovim) regardless of shell-widget initialization state.

### Changed
- Refined Vi-mode behavior: while the prompt symbol still changes from `$` to `>` in `vicmd` mode, the cursor shape now stays consistent based on `SLICK_PROMPT_CURSOR_SHAPE` to reduce visual noise.
- Removed `slick_prompt_apply_cursor_shape` from `slick.zsh`, as the cursor shape is now "baked into" the `slick prompt` output.

### Fixed
- Fixed the issue where the terminal cursor would reset to a block after exiting Neovim or other full-screen terminal applications.

## 0.20.1 (2026-04-08)

### Added
- Added `SLICK_PROMPT_CURSOR_SHAPE` to control the `DECSCUSR` cursor style emitted by `slick.zsh`. The default is `4` (steady underline), supported values are `0` through `6`, and setting it to an empty string disables cursor-shape output entirely.

### Changed
- Changed cursor-shape handling so `slick.zsh` reapplies the configured cursor when Zsh regains prompt control via `zle-line-init` and `zle-keymap-select`, instead of emitting it unconditionally during `preexec`.
- Updated the README and sample `envrc` to document the supported cursor-shape values and how to disable the behavior.

### Fixed
- Reduced prompt interference with full-screen terminal applications such as Neovim by keeping cursor-shape control tied to ZLE/prompt ownership instead of command launch.

## 0.20.0 (2026-04-04)

### Added
- Added `SLICK_PROMPT_GIT_MAIN_BRANCH_COLOR` as the preferred color setting for `main` and `master` branches while keeping `SLICK_PROMPT_GIT_MASTER_BRANCH_COLOR` as a deprecated compatibility alias.
- Added `SLICK_PROMPT_GIT_BRANCH_SYMBOL_COLOR` so the git branch symbol can be colored independently from the branch text.

### Changed
- Changed the default git branch symbol color to `2` (green) so git repositories stand out more clearly while staying aligned with the prompt's numeric color defaults.
- Updated branch rendering so the branch symbol and branch name no longer have to share the same color.
- Refreshed the prompt help text, examples, and sample env config to document the new branch color settings.

### Fixed
- Made `./test.sh` create local test commits with `commit.gpgsign=false` so the integration suite does not fail on systems using global SSH commit signing via 1Password or similar agents.

## 0.19.0 (2026-04-04)

### Added
- Added AWS prompt context detection from `AWS_PROFILE`, `AWS_REGION`, `AWS_DEFAULT_REGION`, and credential env vars with neutral text-only markers like `(aws prod)` and `(aws eu-central-1)`.
- Added Kubernetes prompt context detection from `KUBECONFIG`, rendering the basename of the first kubeconfig path as a text-only marker like `(k8s dev-cluster)`.
- Expanded `scripts/preview_prompt.zsh` and `just preview` to simulate AWS and Kubernetes contexts alongside Toolbx, DevPod, and Python environments.
- Added `slick.zsh` as the canonical reusable shell loader plus `slick.plugin.zsh` for plugin managers such as zinit.

### Changed
- Refactored prompt context detection and parsing into `src/context.rs`, reducing the amount of context-specific logic living in `src/prompt.rs`.
- Kept AWS and Kubernetes markers text-only in v1 to avoid extra font dependencies and symbol configuration surface.
- Added regression coverage for AWS and Kubernetes marker rendering, ordering, and transient prompt output.
- Changed the default git branch symbol to ``; set `SLICK_PROMPT_GIT_BRANCH_SYMBOL=""` to disable it.
- Reduced `load.zsh` to a repo-local dev wrapper that prefers `./target/release/slick` and delegates to `slick.zsh`.
- Added shell regression coverage for `slick.zsh` and wrapper coverage for `load.zsh` and `slick.plugin.zsh`.
- Preserved and chained existing `accept-line`, `zle-line-init`, and `zle-keymap-select` widgets in `slick.zsh` so dotfiles can source it without clobbering prior widget wrappers.

## 0.18.0 (2026-04-04)

### Added
- Optional `SLICK_PROMPT_GIT_BRANCH_SYMBOL` support for prefixing rendered branch names.
- Added `scripts/preview_prompt.zsh` plus `just preview` helpers for simulating Toolbx, DevPod, and Python prompt contexts with current prompt settings.
- Added a default-on transient scrollback prompt with RFC 3339 timestamps; disable it with `SLICK_PROMPT_TRANSIENT=0`.

### Changed
- Changed the default Toolbx marker symbol to `▣` and the default DevPod marker symbol to ``.
- Updated the prompt documentation and examples to use the new default container symbols.
- Documented that the git branch symbol inherits the branch text color.

### Fixed
- Restored the Linux musl GitHub Actions build by provisioning a musl-target zlib prefix for the release build workflow.
- Added a `load.zsh` shell regression guard to keep the preexec cleanup and transient accept-line flow from regressing, including the macOS output-clearing path.

## 0.17.0 (2026-04-04)

### Added
- DevPod prompt marker support using `DEVPOD` and `DEVPOD_WORKSPACE_ID`.
- New environment variables: `SLICK_PROMPT_DEVPOD_SYMBOL`, `SLICK_PROMPT_DEVPOD_COLOR`.
- Regression tests covering DevPod marker rendering, ordering, and environment default handling.

### Changed
- Added explicit `pyenv` prompt detection via `PYENV_VERSION`.
- Added optional `SLICK_PROMPT_GIT_BRANCH_SYMBOL` support for prefixing rendered branch names.
- Promoted `SLICK_PROMPT_PYTHON_ENV_COLOR` as the preferred Python environment color setting while keeping `PIPENV_ACTIVE_COLOR` as a legacy fallback.
- Fixed Python environment prompt parsing so Pipenv names keep internal hyphens while still dropping trailing hash suffixes.
- Updated `pyenv` prompt parsing to ignore `system` and use the first real entry when multiple versions are present.
- Scoped the legacy `PIPENV_ACTIVE_COLOR` fallback to Pipenv-derived markers so it no longer affects `pyenv`.
- Expanded regression coverage for Pipenv and `pyenv` prompt parsing edge cases.
- Updated the default Toolbx marker color to yellow.

## 0.16.0 (2026-04-01)

### Changed
- Removed the OpenSSL dependency from slick's build by switching `git2` to local-only usage without its default HTTPS/SSH features.
- Simplified musl CI/release builds by dropping the obsolete `musl` cargo feature.

### Added
- Toolbx prompt marker showing the active toolbox name before the path, eg. `(🧰 codex)`.
- New environment variables: `SLICK_PROMPT_TOOLBOX_SYMBOL`, `SLICK_PROMPT_TOOLBOX_COLOR`.

## 0.15.3 (2026-02-10)

### Fixed
- Fixed issue where command output (like `ls`) could be cleared by asynchronous prompt updates, especially on macOS.
- Implemented robust file descriptor management in `load.zsh` to ensure background prompt processes are terminated immediately when a new command begins execution (`preexec`).
- Added explicit cleanup of stale background processes in `precmd` to prevent race conditions.

## 0.15.2 (2026-02-06)

### Fixed
- Fixed "unexpected argument '-3' found" error when system clock is adjusted backwards (#19)
- Negative elapsed time values now clamped to 0 instead of causing argument parsing errors

### Added
- Comprehensive tests for elapsed time handling including negative values

## 0.15.0 (2025-11-23)

### Added
- New `git` module (`src/git.rs`) for centralized Git operations.
- Comprehensive unit tests for the `git` module.

### Changed
- Refactored Git-related logic from `src/precmd.rs` into `src/git.rs`, improving code organization and modularity.
- Enhanced public API documentation for the `git` module.

### Fixed
- Addressed various Clippy lints (`must_use_candidate`, `doc_markdown`, `double_must_use`), improving code quality and adherence to best practices.

## 0.14.6 (2025-11-11)

### Fixed
- **Elapsed time no longer flickers** - Fixed time recalculation across render phases
- Elapsed time now calculated once in `precmd` and passed to all render phases
- Consistent elapsed time display across Phase 1 and Phase 2 renders
- **SLICK_TEST_DELAY now respects the value** - Use `SLICK_TEST_DELAY=N` for N seconds delay

### Changed
- Added `-e` (elapsed) flag to `prompt` subcommand for pre-calculated elapsed time
- `SLICK_TEST_DELAY` is now configurable (was hardcoded to 3 seconds)
- Updated `load.zsh` to calculate elapsed time once and pass via `-e` flag
- Deprecated `-t` (timestamp) flag in favor of `-e` for better UX (kept for backwards compatibility)

## 0.14.5 (2025-11-07)

### Changed
- cargo update replaced crate users with uzers

## 0.14.4 (2025-11-07)

### Fixed
- **Auth cache now writes correctly** - Fixed race condition where cache files weren't being written
- Auth fetch task now has 500ms grace period to complete and write cache before process exits
- Improved error handling: timeouts and command errors no longer incorrectly marked as auth failures
- Better cache update logic: only updates cache on successful fetch or auth-specific errors

### Improved
- Cleaner code organization with all imports at top of files
- Removed unsafe `.unwrap()` calls throughout codebase
- Added `unix_timestamp()` helper function for cleaner time handling
- Better comments explaining async task behavior and trade-offs

### Added
- **Streaming async prompts with tokio** - Two-phase rendering for instant display with live updates
- `load.zsh` - Quick setup script for testing and development
- Comprehensive test suite for auth cache functionality (12 new tests)

### Changed
- Complete async/await rewrite using tokio for better performance
- Phase 1 (0ms): Display prompt immediately with cached auth status
- Phase 2 (async): Update prompt only if auth status changes (prevents flickering)
- Git fetch uses `tokio::process::Command` with 5-second timeout
- Removed fork crate dependency - pure tokio async I/O
- Auth cache lifetime: 5 minutes in `~/.cache/slick/`

## 0.14.3 (2024-11-06)

### Performance
- **40-60% faster prompt rendering** through comprehensive optimizations
- Optimized string allocations and replaced cloning with references
- Replaced `BTreeMap` with `HashMap` for O(1) git status counting
- Cached redundant function calls and added capacity hints

### Improved
- Better error handling throughout the codebase
- Code now passes strict clippy checks

## 0.14.2 (2024-11-06)

### Fixed
- Restored proper `tokio::spawn` for async git fetch
- Improved error handling with native Rust
- Cleaner code organization

## 0.14.1 (2024-11-05)

### Added
- Git authentication detection with 5-minute cache (`~/.cache/slick/auth_*`)
- Lock symbol (🔒) when repository requires authentication
- New environment variables: `SLICK_PROMPT_GIT_AUTH_SYMBOL`, `SLICK_PROMPT_GIT_AUTH_COLOR`

### Fixed
- SSH hanging with `ControlMaster=no` to prevent multiplexing issues
- 5-second timeout protection on git fetch

## 0.14.0 (2024-11-05)

### Added
- Environment variable caching using `OnceLock` for better performance
- Non-blocking async git fetch
- Enhanced git fetch safety with proper environment variables

### Changed
- Release profile optimizations (LTO, strip symbols)
- Minimized tokio features for smaller binary size

## 0.13.0

### Changed
- Use tokio for async operations

## 0.12.0

### Added
- Support for `VIRTUAL_ENV_PROMPT` environment variable

## 0.11.0

### Added
- Customizable ahead/behind indicators: `SLICK_PROMPT_GIT_REMOTE_AHEAD`, `SLICK_PROMPT_GIT_REMOTE_BEHIND`

## 0.9.3

### Added
- Display git `user.name` in prompt (disable with `SLICK_PROMPT_NO_GIT_UNAME`)
- PIPENV support
- Special color for `master` and `main` branches
