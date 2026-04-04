# slick - async ZSH prompt

[![crates.io](https://img.shields.io/crates/v/slick.svg)](https://crates.io/crates/slick)
[![Test & Build](https://github.com/nbari/slick/actions/workflows/build.yml/badge.svg)](https://github.com/nbari/slick/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/nbari/slick/graph/badge.svg?token=BJYTNUUJ5O)](https://codecov.io/gh/nbari/slick)

[![example](https://img.youtube.com/vi/ZFQ2bykpm6s/0.jpg)](https://www.youtube.com/watch?v=ZFQ2bykpm6s)

## How to use

Install:

    cargo install slick

To install cargo:

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

If in Linux you may need install this:

    apt install -y build-essential libssl-dev pkg-config

check your PATH `$HOME/.cargo/bin/slick`

### Quick Test (Development)

For quick testing or development:

```sh
# Build and test
cargo build --release
source load.zsh
```

The `load.zsh` script automatically detects the slick binary and sets up the prompt.

### Production Setup

Add this to your `.zshrc` or adapt [`load.zsh`](load.zsh):

```sh
zmodload zsh/datetime
autoload -Uz add-zsh-hook

typeset -g slick_prompt_data
typeset -g slick_prompt_fd
typeset -g slick_prompt_timestamp
typeset -g slick_prompt_elapsed

SLICK_PATH=$HOME/.cargo/bin/slick

function slick_prompt_transient_enabled {
    [[ "${SLICK_PROMPT_TRANSIENT:-1}" != "0" ]]
}

function slick_prompt_rfc3339_timestamp {
    local timestamp

    strftime -s timestamp '%Y-%m-%dT%H:%M:%S%z' $EPOCHSECONDS
    print -r -- "${timestamp[1,-3]}:${timestamp[-2,-1]}"
}

function slick_prompt_render {
    local exit_status=${1:-0}
    local transient=${2:-0}
    local transient_timestamp=${3:-}
    local -a args

    args=(
        "$SLICK_PATH"
        prompt
        -k "${KEYMAP:-main}"
        -r "$exit_status"
        -d "${slick_prompt_data:-}"
    )

    if [[ -n "${slick_prompt_elapsed:-}" ]]; then
        args+=(-e "$slick_prompt_elapsed")
    fi

    if [[ "$transient" == 1 ]]; then
        args+=(--transient)
        if [[ -n "$transient_timestamp" ]]; then
            args+=(--transient-timestamp "$transient_timestamp")
        fi
    fi

    "${args[@]}"
}

function slick_prompt_refresh {
    local exit_status=$?
    local line

    if read -r -u $1 line; then
        slick_prompt_data="$line"
        PROMPT=$(slick_prompt_render "$exit_status")
        zle && zle reset-prompt
        return
    fi

    unset slick_prompt_timestamp
    unset slick_prompt_elapsed
    zle -F $1
    exec {1}<&-
}

function zle-line-init zle-keymap-select {
    PROMPT=$(slick_prompt_render 0)
    zle && zle reset-prompt
}

function slick_prompt_accept_line {
    local exit_status=$?
    local transient_timestamp

    if slick_prompt_transient_enabled; then
        transient_timestamp=$(slick_prompt_rfc3339_timestamp)
        PROMPT=$(slick_prompt_render "$exit_status" 1 "$transient_timestamp")
        zle reset-prompt
    fi

    zle .accept-line
}

function slick_prompt_precmd {
    slick_prompt_data=""

    if [[ -n "$slick_prompt_fd" ]]; then
        zle -F $slick_prompt_fd
        exec {slick_prompt_fd}<&-
        unset slick_prompt_fd
    fi

    if [[ -n "$slick_prompt_timestamp" ]]; then
        slick_prompt_elapsed=$(( EPOCHSECONDS - slick_prompt_timestamp ))
        [[ $slick_prompt_elapsed -lt 0 ]] && slick_prompt_elapsed=0
    else
        unset slick_prompt_elapsed
    fi

    exec {slick_prompt_fd}< <($SLICK_PATH precmd)
    zle -F $slick_prompt_fd slick_prompt_refresh
}

function slick_prompt_preexec {
    if [[ -n "$slick_prompt_fd" ]]; then
        zle -F $slick_prompt_fd
        exec {slick_prompt_fd}<&-
        unset slick_prompt_fd
    fi

    slick_prompt_timestamp=$EPOCHSECONDS
    echo -ne "\e[4 q"
}

add-zsh-hook precmd slick_prompt_precmd
add-zsh-hook preexec slick_prompt_preexec
zle -N zle-keymap-select
zle -N zle-line-init
zle -N accept-line slick_prompt_accept_line
```

## 🔤 Font Setup

**Seeing boxes (□) instead of symbols?** You need a Nerd Font.

**Quick fix:**
1. Download a Nerd Font: https://www.nerdfonts.com/font-downloads
   - Recommended: **Monoid Nerd Font** (clean, minimalist), JetBrainsMono Nerd Font, FiraCode Nerd Font
2. Install the font on your system
3. Configure your terminal to use it
4. Restart terminal

**Using Monoid Nerd Font?** Works great with slick's minimalist design!

**Don't want to install fonts?** Use Unicode or ASCII alternatives:
```bash
export SLICK_PROMPT_SYMBOL="→"    # Unicode arrow
# or
export SLICK_PROMPT_SYMBOL=">"    # ASCII
```

## Customizations

Slick can be customized using environment variables.

### Quick Start

```bash
# Disable git fetch for faster prompts (removes ~500ms auth check on first run)
export SLICK_PROMPT_GIT_FETCH=0

# Custom symbols
export SLICK_PROMPT_SYMBOL="❯"
export SLICK_PROMPT_VICMD_SYMBOL="❮"

# Custom colors
export SLICK_PROMPT_PATH_COLOR=blue
export SLICK_PROMPT_SYMBOL_COLOR=magenta
export SLICK_PROMPT_PYTHON_ENV_COLOR=7
export SLICK_PROMPT_DEVPOD_COLOR=7

# Toolbx marker
export SLICK_PROMPT_TOOLBOX_SYMBOL="▣"
export SLICK_PROMPT_TOOLBOX_COLOR=yellow

# Optional git branch prefix
export SLICK_PROMPT_GIT_BRANCH_SYMBOL=$'\ue0a0'
```

### All Environment Variables

#### General Settings
```bash
export SLICK_PROMPT_CMD_MAX_EXEC_TIME=5        # Max command time to display (seconds)
export SLICK_PROMPT_GIT_FETCH=1                # Enable git fetch (1=yes, 0=no)
export SLICK_PROMPT_NO_GIT_UNAME=0             # Hide git username (1=hide, 0=show)
export SLICK_PROMPT_NON_BREAKING_SPACE=" "     # Non-breaking space character
export SLICK_PROMPT_TRANSIENT=1                # Compact previous prompt in scrollback (0=disable)
```

#### Prompt Symbols
```bash
export SLICK_PROMPT_SYMBOL="$"                 # Main prompt symbol
export SLICK_PROMPT_VICMD_SYMBOL=">"           # Vi command mode symbol
export SLICK_PROMPT_ROOT_SYMBOL="#"            # Root user symbol
export SLICK_PROMPT_GIT_REMOTE_AHEAD="⇡"       # Git ahead symbol
export SLICK_PROMPT_GIT_REMOTE_BEHIND="⇣"      # Git behind symbol
export SLICK_PROMPT_GIT_AUTH_SYMBOL="🔒"       # Git auth failed symbol
export SLICK_PROMPT_GIT_BRANCH_SYMBOL=""       # Optional prefix before branch name, e.g. $'\ue0a0'
export SLICK_PROMPT_TOOLBOX_SYMBOL="▣"         # Toolbx marker symbol
export SLICK_PROMPT_DEVPOD_SYMBOL=""          # DevPod marker symbol
```

#### Colors
```bash
# Colors can be named (red, blue, etc.) or numbers (0-255)
export SLICK_PROMPT_ERROR_COLOR=196            # Error message color
export SLICK_PROMPT_DEVPOD_COLOR=7             # DevPod marker color
export SLICK_PROMPT_PATH_COLOR=74              # Directory path color
export SLICK_PROMPT_PYTHON_ENV_COLOR=7         # Virtualenv/pyenv color
export SLICK_PROMPT_SYMBOL_COLOR=5             # Prompt symbol color
export SLICK_PROMPT_VICMD_COLOR=3              # Vi command mode color
export SLICK_PROMPT_ROOT_COLOR=1               # Root user color
export SLICK_PROMPT_SSH_COLOR=8                # SSH session color
export SLICK_PROMPT_TIME_ELAPSED_COLOR=3       # Command time color
export SLICK_PROMPT_TOOLBOX_COLOR=3            # Toolbx marker color
```

#### Git Colors
```bash
export SLICK_PROMPT_GIT_BRANCH_COLOR=3         # Branch name color
export SLICK_PROMPT_GIT_MASTER_BRANCH_COLOR=160  # master/main branch color
export SLICK_PROMPT_GIT_ACTION_COLOR=3         # Git action (merge, rebase) color
export SLICK_PROMPT_GIT_STATUS_COLOR=5         # Modified files color
export SLICK_PROMPT_GIT_STAGED_COLOR=7         # Staged files color
export SLICK_PROMPT_GIT_REMOTE_COLOR=6         # Remote status color
export SLICK_PROMPT_GIT_UNAME_COLOR=8          # Git username color
export SLICK_PROMPT_GIT_AUTH_COLOR=red         # Git auth failed color
```

If `SLICK_PROMPT_GIT_BRANCH_SYMBOL` is set, it is printed immediately before the branch name, for example ` main`. In `zsh`, you can set it safely with `export SLICK_PROMPT_GIT_BRANCH_SYMBOL=$'\ue0a0'`. It uses the same color as the branch text: `SLICK_PROMPT_GIT_MASTER_BRANCH_COLOR` for `main`/`master`, and `SLICK_PROMPT_GIT_BRANCH_COLOR` for other branches.

`PIPENV_ACTIVE_COLOR` is still honored as a legacy fallback, but `SLICK_PROMPT_PYTHON_ENV_COLOR` is the preferred setting for Python environments.

### Example Configurations

#### Minimal/Fast (no network calls)
```bash
export SLICK_PROMPT_GIT_FETCH=0           # No git fetch
export SLICK_PROMPT_NO_GIT_UNAME=1        # Hide username
export SLICK_PROMPT_SYMBOL=">"            # Simple symbol
```

#### Colorful
```bash
export SLICK_PROMPT_SYMBOL="➜"
export SLICK_PROMPT_SYMBOL_COLOR=cyan
export SLICK_PROMPT_PATH_COLOR=blue
export SLICK_PROMPT_GIT_BRANCH_COLOR=yellow
export SLICK_PROMPT_ERROR_COLOR=red
```

#### Nerd Fonts (Monoid, JetBrainsMono, etc.)
⚠️ **Requires a Nerd Font** - Works great with Monoid Nerd Font, JetBrainsMono Nerd Font, FiraCode Nerd Font

```bash
# Example with Monoid Nerd Font or similar Nerd Fonts
export SLICK_PROMPT_SYMBOL=""           # nf-oct-chevron_right
export SLICK_PROMPT_VICMD_SYMBOL=""       # nf-oct-chevron_left
export SLICK_PROMPT_ROOT_SYMBOL=""        # nf-fa-flash
export SLICK_PROMPT_GIT_AUTH_SYMBOL=""    # nf-fa-lock
export SLICK_PROMPT_GIT_REMOTE_AHEAD=""   # nf-md-arrow_up
export SLICK_PROMPT_GIT_REMOTE_BEHIND=""  # nf-md-arrow_down
```

**Download Nerd Fonts:** https://www.nerdfonts.com/

These symbols work with any Nerd Font (Monoid, JetBrainsMono, FiraCode, Hack, etc.)

See more examples in [envrc](envrc).

## Prompt Preview

Use the preview helper to render the prompt with simulated Toolbx, DevPod, and Python contexts while keeping your current `SLICK_PROMPT_*` settings:

```bash
just preview
just preview-watch
```

Optional overrides let you tune the sample names and branch data:

```bash
SLICK_PREVIEW_BRANCH=main \
SLICK_PREVIEW_STATUS="M 2" \
SLICK_PREVIEW_TOOLBOX_NAME=toolbox \
SLICK_PREVIEW_DEVPOD_NAME=workspace \
SLICK_PREVIEW_PYTHON_ENV=.venv \
just preview
```

The helper lives at `scripts/preview_prompt.zsh` and uses `print -P` so the prompt colors render as they would in `zsh`.

## Transient Prompt

By default, slick rewrites the prompt you just used into a compact single-line form when you press Enter. The live prompt stays rich while you type; only the scrollback version is compacted. The transient form includes an RFC 3339 timestamp plus the key context markers, path, branch, and prompt symbol while omitting noisier git status details.

Disable it if you prefer the old behavior:

```bash
export SLICK_PROMPT_TRANSIENT=0
```

## Toolbx Detection

Slick detects when it is running inside Fedora Toolbx and shows the toolbox name before the path.

**Example:**
```bash
(▣ codex) ~/projects/slick main
❯
```

Configure the Toolbx marker:
```bash
export SLICK_PROMPT_TOOLBOX_SYMBOL="▣"   # Default
export SLICK_PROMPT_TOOLBOX_COLOR=3       # Default
```

## DevPod Detection

Slick detects when `DEVPOD` is set and shows `DEVPOD_WORKSPACE_ID` before the path.

**Example:**
```bash
( hfile) ~/projects/slick main
❯
```

Configure the DevPod marker:
```bash
export SLICK_PROMPT_DEVPOD_SYMBOL=""      # Default
export SLICK_PROMPT_DEVPOD_COLOR=7        # Default
```

## 🔒 SSH Authentication Detection

Slick automatically detects when SSH remotes require authentication and displays a lock symbol (🔒).

**How it works (Streaming Async Prompts):**
- **Instant display**: Prompt shows immediately with cached auth status (from previous runs)
- **Async update**: If git fetch is enabled, auth check runs in background with 500ms grace period
- **Smart cache**: First run takes ~500ms to write cache, subsequent runs are instant with cached status
- **Cache-based**: Auth status is cached for 5 minutes in `~/.cache/slick/`
- **Non-blocking**: Uses tokio for true async I/O - no delays, no hanging
- **Smart timeout**: Git fetch has 5-second timeout to prevent indefinite waits

**Example:**
```bash
# cd into repo with SSH remote requiring auth
cd my-private-repo
# First time: ~500ms to check and write cache (shows auth_failed status)
# Subsequent prompts: instant 🔒 from cache (valid for 5 minutes)
# After cache expires (5 min): another ~500ms check to refresh cache
```

**Two-phase rendering:**
1. **Phase 1 (0ms)**: Display prompt immediately with all git info + cached auth status
2. **Phase 2 (async)**: Git status updates (~10-50ms), then git fetch checks auth (~500ms)
3. **Cache write**: Auth status written to cache for next prompt

Configure the lock symbol:
```bash
export SLICK_PROMPT_GIT_AUTH_SYMBOL="🔒"  # Default
export SLICK_PROMPT_GIT_AUTH_COLOR=red     # Default
```

See [TEST_README.md](TEST_README.md#test-auth-lock-detection) for testing instructions.

___
Inspired by:

* [pure](https://github.com/sindresorhus/pure)
* [purs](https://github.com/xcambar/purs)
* [zsh-efgit-prompt](https://github.com/ericfreese/zsh-efgit-prompt)
