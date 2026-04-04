#!/usr/bin/env zsh

set -eo pipefail

ROOT_DIR=${0:A:h:h}
cd "$ROOT_DIR"

if [[ ! -x ./target/release/slick ]]; then
    print -u2 -- "error: expected ./target/release/slick to exist"
    exit 1
fi

typeset -ga ZLE_CALLS=()
typeset -gi ACCEPT_LINE_CALLED=0

die() {
    print -u2 -- "error: $1"
    exit 1
}

zle() {
    ZLE_CALLS+=("$*")
    if [[ "${1:-}" == ".accept-line" ]]; then
        ACCEPT_LINE_CALLED=1
    fi
    return 0
}

source ./load.zsh >/dev/null
autoload -Uz add-zsh-hook
add-zsh-hook -D precmd slick_prompt_precmd
add-zsh-hook -D preexec slick_prompt_preexec

function slick_prompt_render {
    print -r -- "render:$*"
}

fd_is_open() {
    local fd=$1
    (: <&$fd) 2>/dev/null
}

assert_contains_call() {
    local expected=$1
    local call
    for call in "${ZLE_CALLS[@]}"; do
        if [[ "$call" == "$expected" ]]; then
            return 0
        fi
    done
    die "missing zle call: $expected"
}

assert_no_call() {
    local forbidden=$1
    local call
    for call in "${ZLE_CALLS[@]}"; do
        if [[ "$call" == "$forbidden" ]]; then
            die "unexpected zle call: $forbidden"
        fi
    done
}

# Regression guard for the macOS flicker/output-clearing bug:
# preexec must tear down the async prompt FD before command output starts.
exec {test_fd}< <(sleep 5)
typeset -g slick_prompt_fd=$test_fd
slick_prompt_preexec
[[ -z ${slick_prompt_fd-} ]] || die "slick_prompt_fd should be unset after preexec"
fd_is_open $test_fd && die "preexec should close the async prompt fd"
assert_contains_call "-F $test_fd"

# Transient mode must still delegate to .accept-line so command execution/output proceeds.
ZLE_CALLS=()
ACCEPT_LINE_CALLED=0
PROMPT="full"
SLICK_PROMPT_TRANSIENT=1
slick_prompt_accept_line
[[ $ACCEPT_LINE_CALLED -eq 1 ]] || die "accept-line wrapper must call .accept-line"
[[ "$PROMPT" == render:* ]] || die "transient accept-line should replace PROMPT"
assert_contains_call "reset-prompt"
assert_contains_call ".accept-line"

# When disabled, accept-line should skip the transient rewrite but still execute the command.
ZLE_CALLS=()
ACCEPT_LINE_CALLED=0
PROMPT="full"
SLICK_PROMPT_TRANSIENT=0
slick_prompt_accept_line
[[ $ACCEPT_LINE_CALLED -eq 1 ]] || die "accept-line wrapper must still call .accept-line when transient is disabled"
[[ "$PROMPT" == "full" ]] || die "PROMPT should remain unchanged when transient mode is disabled"
assert_no_call "reset-prompt"
assert_contains_call ".accept-line"

print -r -- "load.zsh regression tests passed"
