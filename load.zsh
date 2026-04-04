#!/bin/zsh
# Development/testing loader for slick prompt

# Detect slick binary location (use absolute paths)
if [[ -f "./target/release/slick" ]]; then
    SLICK_PATH="${PWD}/target/release/slick"
elif [[ -f "$HOME/.cargo/bin/slick" ]]; then
    SLICK_PATH="$HOME/.cargo/bin/slick"
else
    echo "Error: slick binary not found" >&2
    return 1
fi

# Clean up previous instances (safe for re-sourcing)
autoload -Uz add-zsh-hook
add-zsh-hook -D precmd slick_prompt_precmd
add-zsh-hook -D preexec slick_prompt_preexec

# Load required modules
zmodload zsh/datetime

# Register hooks
add-zsh-hook precmd slick_prompt_precmd
add-zsh-hook preexec slick_prompt_preexec

# Register zle widgets
zle -N zle-keymap-select
zle -N zle-line-init
zle -N accept-line slick_prompt_accept_line

# Global variables
typeset -g slick_prompt_data
typeset -g slick_prompt_fd
typeset -g slick_prompt_timestamp
typeset -g slick_prompt_elapsed

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

    # Read ONE line per callback (non-blocking!)
    # ZSH will call this function again if there's more data
    if read -r -u $1 line; then
        slick_prompt_data="$line"
        PROMPT=$(slick_prompt_render "$exit_status")
        zle && zle reset-prompt
        return  # RETURN immediately - don't block! Handler will be called again for next line
    fi

    # No more data - close fd and remove handler
    # Clean up timestamp and elapsed now that all phases are complete
    unset slick_prompt_timestamp
    unset slick_prompt_elapsed
    zle -F $1
    exec {1}<&-

    # Reset global fd if it matches
    if [[ "$1" == "$slick_prompt_fd" ]]; then
        unset slick_prompt_fd
    fi
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

function slick_prompt_precmd() {
    slick_prompt_data=""

    # Clean up any lingering fd from previous prompt
    if [[ -n "$slick_prompt_fd" ]]; then
        zle -F $slick_prompt_fd
        exec {slick_prompt_fd}<&-
        unset slick_prompt_fd
    fi

    # Calculate elapsed time ONCE here (avoids flickering across multiple render phases)
    # If timestamp is set (command was run), calculate elapsed seconds
    # Otherwise, leave it unset (no command was run, e.g., just pressed enter)
    if [[ -n "$slick_prompt_timestamp" ]]; then
        slick_prompt_elapsed=$(( EPOCHSECONDS - slick_prompt_timestamp ))
        # Ensure elapsed time is never negative (can happen with clock adjustments)
        [[ $slick_prompt_elapsed -lt 0 ]] && slick_prompt_elapsed=0
    else
        unset slick_prompt_elapsed
    fi

    exec {slick_prompt_fd}< <($SLICK_PATH precmd)
    zle -F $slick_prompt_fd slick_prompt_refresh
}

function slick_prompt_preexec() {
    # Kill any running async prompt immediately so it doesn't mess up command output
    if [[ -n "$slick_prompt_fd" ]]; then
        zle -F $slick_prompt_fd
        exec {slick_prompt_fd}<&-
        unset slick_prompt_fd
    fi

    slick_prompt_timestamp=$EPOCHSECONDS
    echo -ne "\e[4 q"
}

echo "slick prompt loaded (2-phase async prompt + transient scrollback by default; set SLICK_PROMPT_TRANSIENT=0 to disable)"
