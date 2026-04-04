#!/usr/bin/env zsh

set -eo pipefail

ROOT_DIR=${${(%):-%N}:A:h:h}
cd "$ROOT_DIR"

if [[ ! -x ./target/release/slick ]]; then
    print -u2 -- "error: expected ./target/release/slick to exist"
    exit 1
fi

typeset -ga ZLE_CALLS=()
typeset -gi ACCEPT_LINE_CALLED=0
typeset -gi ORIGINAL_ACCEPT_LINE_CALLED=0
typeset -gi ORIGINAL_LINE_INIT_CALLED=0
typeset -gi ORIGINAL_KEYMAP_CALLED=0

export SLICK_PATH="$ROOT_DIR/target/release/slick"

die() {
    print -u2 -- "error: $1"
    exit 1
}

function custom_accept_line {
    ORIGINAL_ACCEPT_LINE_CALLED=1
}

function custom_zle_line_init {
    ORIGINAL_LINE_INIT_CALLED=1
}

function custom_zle_keymap_select {
    ORIGINAL_KEYMAP_CALLED=1
}

zle -N accept-line custom_accept_line
zle -N zle-line-init custom_zle_line_init
zle -N zle-keymap-select custom_zle_keymap_select

SOURCE_OUTPUT_FILE=$(mktemp)
source ./slick.zsh >"$SOURCE_OUTPUT_FILE"
SOURCE_OUTPUT=$(<"$SOURCE_OUTPUT_FILE")
rm -f "$SOURCE_OUTPUT_FILE"
[[ -z "$SOURCE_OUTPUT" ]] || die "slick.zsh should be quiet when sourced"

[[ "${widgets[accept-line]-}" == "user:slick_prompt_accept_line" ]] || die "slick.zsh should install the accept-line wrapper"
[[ "${widgets[slick_prompt_original_accept_line]-}" == "user:custom_accept_line" ]] || die "slick.zsh should preserve the previous accept-line widget"
[[ "${widgets[zle-line-init]-}" == "user:slick_prompt_zle_line_init" ]] || die "slick.zsh should install the zle-line-init wrapper"
[[ "${widgets[slick_prompt_original_zle_line_init]-}" == "user:custom_zle_line_init" ]] || die "slick.zsh should preserve the previous zle-line-init widget"
[[ "${widgets[zle-keymap-select]-}" == "user:slick_prompt_zle_keymap_select" ]] || die "slick.zsh should install the zle-keymap-select wrapper"
[[ "${widgets[slick_prompt_original_zle_keymap_select]-}" == "user:custom_zle_keymap_select" ]] || die "slick.zsh should preserve the previous zle-keymap-select widget"

autoload -Uz add-zsh-hook
add-zsh-hook -D precmd slick_prompt_precmd
add-zsh-hook -D preexec slick_prompt_preexec

zle() {
    ZLE_CALLS+=("$*")
    case "${1:-}" in
        .accept-line)
            ACCEPT_LINE_CALLED=1
            ;;
        slick_prompt_original_accept_line)
            ORIGINAL_ACCEPT_LINE_CALLED=1
            ;;
        slick_prompt_original_zle_line_init)
            ORIGINAL_LINE_INIT_CALLED=1
            ;;
        slick_prompt_original_zle_keymap_select)
            ORIGINAL_KEYMAP_CALLED=1
            ;;
    esac
    return 0
}

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

# Transient mode must still delegate to the preserved accept-line widget so command execution/output proceeds.
ZLE_CALLS=()
ACCEPT_LINE_CALLED=0
ORIGINAL_ACCEPT_LINE_CALLED=0
PROMPT="full"
SLICK_PROMPT_TRANSIENT=1
slick_prompt_accept_line
[[ $ACCEPT_LINE_CALLED -eq 0 ]] || die "accept-line wrapper should not bypass the preserved widget"
[[ $ORIGINAL_ACCEPT_LINE_CALLED -eq 1 ]] || die "accept-line wrapper must call the preserved widget"
[[ "$PROMPT" == render:* ]] || die "transient accept-line should replace PROMPT"
assert_contains_call "reset-prompt"
assert_contains_call "slick_prompt_original_accept_line"
assert_no_call ".accept-line"

# When disabled, accept-line should skip the transient rewrite but still execute the preserved widget.
ZLE_CALLS=()
ACCEPT_LINE_CALLED=0
ORIGINAL_ACCEPT_LINE_CALLED=0
PROMPT="full"
SLICK_PROMPT_TRANSIENT=0
slick_prompt_accept_line
[[ $ACCEPT_LINE_CALLED -eq 0 ]] || die "accept-line wrapper should not bypass the preserved widget when transient is disabled"
[[ $ORIGINAL_ACCEPT_LINE_CALLED -eq 1 ]] || die "accept-line wrapper must still call the preserved widget when transient is disabled"
[[ "$PROMPT" == "full" ]] || die "PROMPT should remain unchanged when transient mode is disabled"
assert_no_call "reset-prompt"
assert_contains_call "slick_prompt_original_accept_line"
assert_no_call ".accept-line"

# Loader wrappers should preserve existing line-init and keymap-select widgets.
ZLE_CALLS=()
ORIGINAL_LINE_INIT_CALLED=0
slick_prompt_zle_line_init
[[ $ORIGINAL_LINE_INIT_CALLED -eq 1 ]] || die "zle-line-init wrapper should call the preserved widget"
assert_contains_call "reset-prompt"
assert_contains_call "slick_prompt_original_zle_line_init"

ZLE_CALLS=()
ORIGINAL_KEYMAP_CALLED=0
slick_prompt_zle_keymap_select
[[ $ORIGINAL_KEYMAP_CALLED -eq 1 ]] || die "zle-keymap-select wrapper should call the preserved widget"
assert_contains_call "reset-prompt"
assert_contains_call "slick_prompt_original_zle_keymap_select"

print -r -- "slick.zsh regression tests passed"
