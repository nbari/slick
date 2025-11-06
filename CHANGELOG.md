## 0.14.2 (2024-11-06)

### Fixed
- **Regression**: Restored proper `tokio::spawn` for async git fetch (was using shell backgrounding)
- Improved error handling with native Rust instead of shell script parsing
- Cleaner code organization with consolidated imports

## 0.14.1 (2024-11-05)

### Added
- Git authentication detection with 5-minute cache (`~/.cache/slick/auth_*`)
- Lock symbol (🔒) when repository requires authentication or doesn't exist
- Environment variables: `SLICK_PROMPT_GIT_AUTH_SYMBOL`, `SLICK_PROMPT_GIT_AUTH_COLOR`

### Fixed
- SSH hanging with `ControlMaster=no` to prevent multiplexing issues
- 5-second timeout protection on git fetch to prevent indefinite hangs
- Auth detection via git fetch exit codes and error patterns

## 0.14.0 (2024-11-05)

### Added
- Environment variable caching using `OnceLock` for better performance
- Non-blocking async git fetch (fire-and-forget)
- Enhanced git fetch safety with `GIT_TERMINAL_PROMPT=0`, `GIT_ASKPASS=true`, `ssh -o BatchMode=yes`

### Changed
- Release profile optimizations: LTO, codegen-units=1, strip=true
- Minimized tokio features for smaller binary size

## 0.13

### Changed
- Use tokio for async git status

## 0.12

### Added
- Support for `VIRTUAL_ENV_PROMPT` environment variable

## 0.11

### Added
- Environment variables `SLICK_PROMPT_GIT_REMOTE_AHEAD` and `SLICK_PROMPT_GIT_REMOTE_BEHIND` to customize ahead/behind indicators

## 0.9.3

### Added
- Display git `user.name` in prompt (disable with `SLICK_PROMPT_NO_GIT_UNAME`)
- PIPENV support (#11)
- Special color for `master` and `main` branches (#10)
