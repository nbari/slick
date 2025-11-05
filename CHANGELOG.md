## 0.14.1 (2024-11-05)

### Features
- **Git repository authentication detection**: Automatically detects when git fetch fails due to authentication/authorization issues
  - Non-blocking background check runs with git fetch (when `SLICK_PROMPT_GIT_FETCH` enabled)
  - Cache-based system: Auth status cached for 5 minutes in `~/.cache/slick/auth_*`
  - Detects authentication failures by checking git fetch exit code and error messages
  - Uses `timeout 5` to prevent hanging on auth prompts
  - Displays last known auth status instantly, updates cache asynchronously after each fetch
  - Shows lock symbol (🔒) when repository requires authentication or doesn't exist

### Fixed
- **Critical**: All git operations remain fully synchronous for instant prompt display (< 0.02s)
  - User name, path, branch, and ALL git status info display instantly
  - Only git fetch runs asynchronously in background (fire-and-forget)
  - Auth status cache is read synchronously (instant), updated by background fetch
- **SSH hanging**: Added `ControlMaster=no` to prevent SSH multiplexing issues that caused hangs
- **Timeout protection**: Added 5-second timeout to git fetch to prevent indefinite hangs
- Auth status now detected from actual git fetch errors (more reliable than SSH test)

### Technical Details
- Auth check integrated with git fetch - no separate SSH authentication test needed
- Cache file format: `timestamp:status` (1=auth/access failed, 0=success)
- Cache automatically expires after 5 minutes (300 seconds)
- Error detection patterns: "permission denied", "authentication failed", "could not read", "repository not found", "access denied"
- Exit code 124 (timeout) also treated as auth failure
- Prompt reads cached auth status synchronously (~instant), git fetch updates cache in background
- Auth status appears in JSON output as `auth_failed` boolean field
- Lock symbol displayed when `auth_failed=true` (configurable via `SLICK_PROMPT_GIT_AUTH_SYMBOL` and `SLICK_PROMPT_GIT_AUTH_COLOR`)

### Performance
- **Prompt display**: ~0.01-0.02s (instant, fully synchronous)
- **Background git fetch**: 1-5s depending on network (does not block prompt)
- **With fetch disabled** (`SLICK_PROMPT_GIT_FETCH=0`): ~0.01s (no background tasks, uses existing cache)

## 0.14.0 (2024-11-05)

### Security & Stability  
- **Enhanced git fetch safety**: Added `GIT_SSH_COMMAND="ssh -o BatchMode=yes"` and `GIT_ASKPASS=true` to prevent interactive prompts
- **Non-blocking async fetch**: Fire-and-forget git fetch to avoid blocking prompt display
- **Real-time git status**: Kept `no_refresh=false` to accurately reflect repository state
- Environment variables to prevent credential prompts: `GIT_TERMINAL_PROMPT=0`, `GIT_ASKPASS=true`

### Performance Improvements
- **Environment variable caching**: Reduced syscalls using `OnceLock` for repeated env lookups
- **Binary size optimization**: Minimized tokio features (only `rt-multi-thread`, `process`, `macros`)
- Added release profile optimizations: LTO, codegen-units=1, strip=true

### Infrastructure
- Added `auth_failed` field to Prompt struct (prepared for future auth detection)
- Updated dependencies and Cargo.lock

### Testing
- All unit and integration tests passing
- Zero clippy warnings

### Documentation
- Updated `envrc` with comprehensive examples (Nerd Font, Unicode, Emoji configurations)
- Added Monoid Nerd Font examples
- Documented all environment variables

## 0.13

- use tokio for async git status

## 0.12

- use VIRTUAL_ENV_PROMPT if set

## 0.11

- env to modify character for ahead/behind `SLICK_PROMPT_GIT_REMOTE_AHEAD`, `SLICK_PROMPT_GIT_REMOTE_BEHIND`

## 0.9.3

- git `user.name` added to the display, it can be disabled by using the env `SLICK_PROMPT_NO_GIT_UNAME`
- Support for PIPENV #11
- `master` & `main` #10
