# slick - async ZSH prompt

[![crates.io](https://img.shields.io/crates/v/slick.svg)](https://crates.io/crates/slick)
[![Test & Build](https://github.com/nbari/slick/actions/workflows/build.yml/badge.svg)](https://github.com/nbari/slick/actions/workflows/build.yml)

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

Add this to your `.zshrc`:

```sh
# Load required modules
zmodload zsh/datetime
autoload -Uz add-zsh-hook

# Register hooks
add-zsh-hook precmd slick_prompt_precmd
add-zsh-hook preexec slick_prompt_preexec

# Register zle widgets
zle -N zle-keymap-select
zle -N zle-line-init

# Global variables
typeset -g slick_prompt_data
typeset -g slick_prompt_timestamp
typeset -g slick_prompt_elapsed

SLICK_PATH=$HOME/.cargo/bin/slick

function slick_prompt_refresh {
    local exit_status=$?
    local line

    # Read ONE line per callback (non-blocking!)
    # ZSH will call this function again if there's more data
    if read -r -u $1 line; then
        slick_prompt_data="$line"

        # Always pass elapsed time if available (needed for ALL phases to show consistent elapsed time!)
        # Use the pre-calculated elapsed time from precmd to avoid flickering
        if [[ -n "$slick_prompt_elapsed" ]]; then
            PROMPT=$($SLICK_PATH prompt -k "$KEYMAP" -r $exit_status -d ${slick_prompt_data:-""} -e $slick_prompt_elapsed)
        else
            PROMPT=$($SLICK_PATH prompt -k "$KEYMAP" -r $exit_status -d ${slick_prompt_data:-""})
        fi

        zle reset-prompt
        return  # RETURN immediately - don't block! Handler will be called again for next line
    fi

    # No more data - close fd and remove handler
    # Clean up timestamp and elapsed now that all phases are complete
    unset slick_prompt_timestamp
    unset slick_prompt_elapsed
    zle -F $1
    exec {1}<&-
}

function zle-line-init zle-keymap-select {
    PROMPT=$($SLICK_PATH prompt -k "$KEYMAP" -d ${slick_prompt_data:-""})
    zle && zle reset-prompt
}

function slick_prompt_precmd() {
    slick_prompt_data=""

    # Calculate elapsed time ONCE here (avoids flickering across multiple render phases)
    # If timestamp is set (command was run), calculate elapsed seconds
    # Otherwise, leave it unset (no command was run, e.g., just pressed enter)
    if [[ -n "$slick_prompt_timestamp" ]]; then
        slick_prompt_elapsed=$(( $EPOCHSECONDS - $slick_prompt_timestamp ))
    else
        unset slick_prompt_elapsed
    fi

    local fd
    exec {fd}< <($SLICK_PATH precmd)
    zle -F $fd slick_prompt_refresh
}

function slick_prompt_preexec() {
    slick_prompt_timestamp=$EPOCHSECONDS

    # Set cursor style
    # 0  ⇒  blinking block.
    # 1  ⇒  blinking block (default).
    # 2  ⇒  steady block.
    # 3  ⇒  blinking underline.
    # 4  ⇒  steady underline.
    # 5  ⇒  blinking bar, xterm.
    # 6  ⇒  steady bar, xterm.

    echo -ne "\e[4 q"
}
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
```

### All Environment Variables

#### General Settings
```bash
export SLICK_PROMPT_CMD_MAX_EXEC_TIME=5        # Max command time to display (seconds)
export SLICK_PROMPT_GIT_FETCH=1                # Enable git fetch (1=yes, 0=no)
export SLICK_PROMPT_NO_GIT_UNAME=0             # Hide git username (1=hide, 0=show)
export SLICK_PROMPT_NON_BREAKING_SPACE=" "     # Non-breaking space character
```

#### Prompt Symbols
```bash
export SLICK_PROMPT_SYMBOL="$"                 # Main prompt symbol
export SLICK_PROMPT_VICMD_SYMBOL=">"           # Vi command mode symbol
export SLICK_PROMPT_ROOT_SYMBOL="#"            # Root user symbol
export SLICK_PROMPT_GIT_REMOTE_AHEAD="⇡"       # Git ahead symbol
export SLICK_PROMPT_GIT_REMOTE_BEHIND="⇣"      # Git behind symbol
export SLICK_PROMPT_GIT_AUTH_SYMBOL="🔒"       # Git auth failed symbol
```

#### Colors
```bash
# Colors can be named (red, blue, etc.) or numbers (0-255)
export SLICK_PROMPT_ERROR_COLOR=196            # Error message color
export SLICK_PROMPT_PATH_COLOR=74              # Directory path color
export SLICK_PROMPT_SYMBOL_COLOR=5             # Prompt symbol color
export SLICK_PROMPT_VICMD_COLOR=3              # Vi command mode color
export SLICK_PROMPT_ROOT_COLOR=1               # Root user color
export SLICK_PROMPT_SSH_COLOR=8                # SSH session color
export SLICK_PROMPT_TIME_ELAPSED_COLOR=3       # Command time color
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
