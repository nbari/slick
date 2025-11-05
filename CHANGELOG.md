## 0.14.0 (2024-11-05)

### Security & Stability
- **Enhanced authentication prompt prevention**: Added `GIT_SSH_COMMAND="ssh -o BatchMode=yes"` and `GIT_ASKPASS=true`
- **No password prompts**: Complete protection against interactive credential requests
- **Git status accuracy**: Kept `no_refresh=false` for real-time repository status
- Disabled all password prompts: `GIT_TERMINAL_PROMPT=0`, `GIT_ASKPASS=true`, `SSH_ASKPASS=echo`
- SSH batch mode to prevent interactive prompts
- Disabled git editor prompts
- Fire-and-forget async fetch (non-blocking)

### Performance Improvements
- **Environment variable caching**: Reduced syscalls from ~20 to 1 using `OnceLock` (-95%)
- **String optimization**: Pre-allocated buffers, reduced allocations from 3 to 1 per status (-67%)
- **Binary size reduction**: 1.9MB → 1.2MB using LTO and minimal tokio features (-37%)
- **Conditional operations**: Only read git config when needed

### Timeout Protection
- Added 15-second hard timeout on git fetch operations
- 10-second git `--timeout` flag
- 5-second SSH connection timeout (`ConnectTimeout=5`)
- SSH keep-alive detection (`ServerAliveInterval=5`, `ServerAliveCountMax=1`)
- Multiple layers prevent hanging on unreachable remotes

### Build & Dependencies
- Optimized tokio features: only use `rt-multi-thread`, `process`, `macros`, `time`
- Added release profile optimizations: LTO, codegen-units=1, strip=true
- Updated Cargo.lock with optimized dependency tree

### Testing
- Simplified test infrastructure: Single `test.sh` script for all scenarios
- 18+ automated tests (unit + integration)
- Added authentication test scenarios
- Updated `.justfile` with comprehensive test commands
- All tests passing with zero clippy warnings

### Bug Fixes
- Fixed clippy warnings in `src/prompt.rs` (use `map_or_else`)
- Fixed potential race conditions in git operations
- Improved error handling in all git commands

### Documentation
- Updated `envrc` with Monoid Nerd Font examples
- Updated README.md with comprehensive environment variable documentation
- Added Monoid Nerd Font as recommended minimalist option
- Removed redundant documentation files
- All environment variables documented in README.md

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
