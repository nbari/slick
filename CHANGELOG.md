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
