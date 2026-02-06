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

# Global variables
typeset -g slick_prompt_data
typeset -g slick_prompt_timestamp
typeset -g slick_prompt_elapsed

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

        zle && zle reset-prompt
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
        # Ensure elapsed time is never negative (can happen with clock adjustments)
        [[ $slick_prompt_elapsed -lt 0 ]] && slick_prompt_elapsed=0
    else
        unset slick_prompt_elapsed
    fi

    local fd
    exec {fd}< <($SLICK_PATH precmd)
    zle -F $fd slick_prompt_refresh
}

function slick_prompt_preexec() {
    slick_prompt_timestamp=$EPOCHSECONDS
    echo -ne "\e[4 q"
}

echo "slick prompt loaded (2-phase: instant [user path branch] + async [git status])"
