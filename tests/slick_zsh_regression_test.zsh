#!/usr/bin/env zsh

set -eo pipefail

ROOT_DIR=${${(%):-%N}:A:h:h}
cd "$ROOT_DIR"

if [[ -n "$CARGO_TARGET_DIR" ]]; then
    SLICK_BINARY="$CARGO_TARGET_DIR/release/slick"
else
    SLICK_BINARY="./target/release/slick"
fi

if [[ ! -x "$SLICK_BINARY" ]]; then
    print -u2 -- "error: expected $SLICK_BINARY to exist"
    exit 1
fi

typeset -ga ZLE_CALLS=()
typeset -gi ACCEPT_LINE_CALLED=0
typeset -gi ORIGINAL_ACCEPT_LINE_CALLED=0
typeset -gi ORIGINAL_LINE_INIT_CALLED=0
typeset -gi ORIGINAL_KEYMAP_CALLED=0

export SLICK_PATH="$SLICK_BINARY"

typeset -g TEST_OUTPUT_FILE="tests/.slick_zsh_regression_test.$$.$RANDOM.out"
trap 'rm -f -- "$TEST_OUTPUT_FILE"' EXIT

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

psvar=()
psvar[12]='existing-user-value'
source ./slick.zsh >"$TEST_OUTPUT_FILE"
SOURCE_OUTPUT=$(<"$TEST_OUTPUT_FILE")
[[ -z "$SOURCE_OUTPUT" ]] || die "slick.zsh should be quiet when sourced"
[[ "${psvar[12]}" == "existing-user-value" ]] || die "slick.zsh should preserve existing psvar values"

[[ "${widgets[accept-line]-}" == "user:slick_prompt_accept_line" ]] || die "slick.zsh should install the accept-line wrapper"
[[ "${widgets[slick_prompt_original_accept_line]-}" == "user:custom_accept_line" ]] || die "slick.zsh should preserve the previous accept-line widget"
[[ "${widgets[zle-line-init]-}" == "user:slick_prompt_zle_line_init" ]] || die "slick.zsh should install the zle-line-init wrapper"
[[ "${widgets[slick_prompt_original_zle_line_init]-}" == "user:custom_zle_line_init" ]] || die "slick.zsh should preserve the previous zle-line-init widget"
[[ "${widgets[zle-keymap-select]-}" == "user:slick_prompt_zle_keymap_select" ]] || die "slick.zsh should install the zle-keymap-select wrapper"
[[ "${widgets[slick_prompt_original_zle_keymap_select]-}" == "user:custom_zle_keymap_select" ]] || die "slick.zsh should preserve the previous zle-keymap-select widget"

autoload -Uz add-zsh-hook
add-zsh-hook -D precmd slick_prompt_precmd
add-zsh-hook -D preexec slick_prompt_preexec

# Canonical prompts must remain literal with PROMPT_SUBST off, on, or toggled after storage.
typeset -g PROMPT_SUBST_WAS=${options[promptsubst]}
typeset -g PROMPT_LITERAL='branch-$(id)-%n-`id`-\literal'
typeset -g slick_prompt_data='{"branch":"branch-$(id)-%n-`id`-\\literal"}'
export SLICK_PROMPT_CURSOR_SHAPE=''
export SLICK_PROMPT_GIT_BRANCH_SYMBOL=''

# Dynamic cursor rendering must follow the active ZLE keymap before widgets redraw the prompt.
export SLICK_PROMPT_CURSOR_SHAPE=dynamic
KEYMAP=main
DYNAMIC_INSERT_PROMPT=$(slick_prompt_render 0)
[[ "$DYNAMIC_INSERT_PROMPT" == *$'%{\e[6 q%}'* ]] || die "main keymap should render a steady bar cursor"
KEYMAP=vicmd
DYNAMIC_COMMAND_PROMPT=$(slick_prompt_render 0)
[[ "$DYNAMIC_COMMAND_PROMPT" == *$'%{\e[2 q%}'* ]] || die "vicmd keymap should render a steady block cursor"
unset KEYMAP
export SLICK_PROMPT_CURSOR_SHAPE=''

unsetopt promptsubst
slick_prompt_set_prompt 0
[[ "${options[promptsubst]}" == off ]] || die "rendering should preserve PROMPT_SUBST-off state"
PROMPT_SUBST_OFF_OUTPUT=$(print -r -P -- "$PROMPT")
[[ "$PROMPT_SUBST_OFF_OUTPUT" == *"$PROMPT_LITERAL"* ]] || die "PROMPT_SUBST-off rendering should not expose escape backslashes"
[[ "$PROMPT_SUBST_OFF_OUTPUT" != *uid=* ]] || die "PROMPT_SUBST-off rendering unexpectedly executed branch text"

setopt promptsubst
PROMPT_SUBST_TRANSITION_OUTPUT=$(print -r -P -- "$PROMPT")
[[ "$PROMPT_SUBST_TRANSITION_OUTPUT" == *"$PROMPT_LITERAL"* ]] || die "stored prompt should survive enabling PROMPT_SUBST"
[[ "$PROMPT_SUBST_TRANSITION_OUTPUT" != *uid=* ]] || die "enabling PROMPT_SUBST executed stored branch text"

slick_prompt_set_prompt 0
[[ "${options[promptsubst]}" == on ]] || die "rendering should preserve PROMPT_SUBST-on state"
PROMPT_SUBST_ON_OUTPUT=$(print -r -P -- "$PROMPT")
[[ "$PROMPT_SUBST_ON_OUTPUT" == *"$PROMPT_LITERAL"* ]] || die "PROMPT_SUBST-on rendering should preserve exact branch text"
[[ "$PROMPT_SUBST_ON_OUTPUT" != *uid=* ]] || die "PROMPT_SUBST-on rendering executed branch command substitution"

psvar=('other-hook-value')
slick_prompt_set_prompt 0
[[ "${psvar[1]}" == "other-hook-value" ]] || die "rendering should preserve replacement psvar values"
[[ "${psvar[$slick_prompt_dollar_psvar_index]}" == '$' ]] || die "rendering should restore the parent dollar psvar"
[[ "${psvar[$slick_prompt_backtick_psvar_index]}" == '`' ]] || die "rendering should restore the parent backtick psvar"
[[ "${psvar[$slick_prompt_backslash_psvar_index]}" == '\' ]] || die "rendering should restore the parent backslash psvar"
PSVAR_RESTORED_OUTPUT=$(print -r -P -- "$PROMPT")
[[ "$PSVAR_RESTORED_OUTPUT" == *"$PROMPT_LITERAL"* ]] || die "rendering should survive replacement of psvar"
[[ "$PSVAR_RESTORED_OUTPUT" != *uid=* ]] || die "restoring replaced psvar executed branch text"

KSH_ARRAYS_OUTPUT=$(
    (
        setopt KSH_ARRAYS
        unsetopt PROMPT_SUBST
        psvar=()
        slick_prompt_dollar_psvar_index=0
        slick_prompt_backtick_psvar_index=0
        slick_prompt_backslash_psvar_index=0
        slick_prompt_data='{"branch":"cash-$"}'
        slick_prompt_set_prompt 0
        (( slick_prompt_dollar_psvar_index >= 1 &&
           slick_prompt_backtick_psvar_index > slick_prompt_dollar_psvar_index &&
           slick_prompt_backslash_psvar_index > slick_prompt_backtick_psvar_index )) ||
            die "KSH_ARRAYS should produce positive, distinct literal psvar indexes"
        [[ "$PROMPT" == *"%${slick_prompt_dollar_psvar_index}v"* ]] ||
            die "KSH_ARRAYS should use psvar encoding rather than fallback escaping"
        print -r -P -- "$PROMPT"
    )
)
[[ "$KSH_ARRAYS_OUTPUT" == *'cash-$'* ]] || die "literal psvars should render correctly with KSH_ARRAYS"

if [[ "$PROMPT_SUBST_WAS" == on ]]; then
    setopt promptsubst
else
    unsetopt promptsubst
fi
slick_prompt_data=""

zle() {
    ZLE_CALLS+=("$*")
    case "${1:-}" in
        .accept-line)
            ACCEPT_LINE_CALLED=1
            ;;
        slick_prompt_original_accept_line)
            ORIGINAL_ACCEPT_LINE_CALLED=1
            [[ "${SLICK_TEST_WIDGET_REPLACES_PSVAR:-0}" == 1 ]] && psvar=('other-hook-value')
            ;;
        slick_prompt_original_zle_line_init)
            ORIGINAL_LINE_INIT_CALLED=1
            [[ "${SLICK_TEST_WIDGET_REPLACES_PSVAR:-0}" == 1 ]] && psvar=('other-hook-value')
            ;;
        slick_prompt_original_zle_keymap_select)
            ORIGINAL_KEYMAP_CALLED=1
            [[ "${SLICK_TEST_WIDGET_REPLACES_PSVAR:-0}" == 1 ]] && psvar=('other-hook-value')
            ;;
    esac
    return 0
}

function slick_prompt_render {
    if [[ "${SLICK_TEST_RENDER_PSVAR_LITERAL:-0}" == 1 ]]; then
        print -r -- "literal:%${slick_prompt_dollar_psvar_index}v"
    else
        print -r -- "render:$*"
    fi
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

[[ "${slick_prompt_exit_status:-}" == "0" ]] || die "saved exit status should default to zero"

return_status() {
    return "$1"
}

# The command status must survive prompt setup and both asynchronous data phases.
return_status 23 || slick_prompt_precmd
[[ "$slick_prompt_exit_status" -eq 23 ]] || die "precmd should save the previous command status"
ASYNC_PROMPT_FD=$slick_prompt_fd

:
slick_prompt_refresh "$ASYNC_PROMPT_FD"
[[ "$PROMPT" == "render:23" ]] || die "first async prompt phase should use the saved exit status"

:
slick_prompt_refresh "$ASYNC_PROMPT_FD"
[[ "$PROMPT" == "render:23" ]] || die "second async prompt phase should use the saved exit status"

:
slick_prompt_refresh "$ASYNC_PROMPT_FD"
[[ -z ${slick_prompt_fd-} ]] || die "async prompt fd should be unset after refresh"
fd_is_open "$ASYNC_PROMPT_FD" && die "refresh should close the completed async prompt fd"

SLICK_PROMPT_SHORT_TIMESTAMP=1
TIMESTAMP_OUTPUT=$(slick_prompt_rfc3339_timestamp)
[[ "$TIMESTAMP_OUTPUT" == <->:<->:<-> ]] || die "short timestamp should use HH:MM:SS"
[[ "$TIMESTAMP_OUTPUT" != *T* ]] || die "short timestamp should not use RFC3339"

# Regression guard for the macOS flicker/output-clearing bug:
# preexec must tear down the async prompt FD before command output starts.
exec {test_fd}< <(sleep 5)
typeset -g slick_prompt_fd=$test_fd
slick_prompt_preexec >"$TEST_OUTPUT_FILE"
PREEXEC_OUTPUT=$(<"$TEST_OUTPUT_FILE")
[[ -z ${slick_prompt_fd-} ]] || die "slick_prompt_fd should be unset after preexec"
[[ -z "$PREEXEC_OUTPUT" ]] || die "slick_prompt_preexec should not emit cursor-shape output"
fd_is_open $test_fd && die "preexec should close the async prompt fd"
assert_contains_call "-F $test_fd"

# Transient mode must still delegate to the preserved accept-line widget so command execution/output proceeds.
ZLE_CALLS=()
ACCEPT_LINE_CALLED=0
ORIGINAL_ACCEPT_LINE_CALLED=0
PROMPT="full"
SLICK_PROMPT_TRANSIENT=1
SLICK_PROMPT_SHORT_TIMESTAMP=1
slick_prompt_accept_line
[[ $ACCEPT_LINE_CALLED -eq 0 ]] || die "accept-line wrapper should not bypass the preserved widget"
[[ $ORIGINAL_ACCEPT_LINE_CALLED -eq 1 ]] || die "accept-line wrapper must call the preserved widget"
[[ "$PROMPT" == render:* ]] || die "transient accept-line should replace PROMPT"
[[ "$PROMPT" == render:23\ 1\ [0-9][0-9]:[0-9][0-9]:[0-9][0-9] ]] || die "accept-line should preserve status and pass the short transient timestamp"
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
slick_prompt_zle_line_init >"$TEST_OUTPUT_FILE"
LINE_INIT_OUTPUT=$(<"$TEST_OUTPUT_FILE")
[[ $ORIGINAL_LINE_INIT_CALLED -eq 1 ]] || die "zle-line-init wrapper should call the preserved widget"
[[ "$PROMPT" == "render:23" ]] || die "zle-line-init should use the saved exit status"
assert_contains_call "reset-prompt"
assert_contains_call "slick_prompt_original_zle_line_init"

ZLE_CALLS=()
ORIGINAL_KEYMAP_CALLED=0
slick_prompt_zle_keymap_select >"$TEST_OUTPUT_FILE"
KEYMAP_OUTPUT=$(<"$TEST_OUTPUT_FILE")
[[ $ORIGINAL_KEYMAP_CALLED -eq 1 ]] || die "zle-keymap-select wrapper should call the preserved widget"
[[ "$PROMPT" == "render:23" ]] || die "zle-keymap-select should use the saved exit status"
assert_contains_call "reset-prompt"
assert_contains_call "slick_prompt_original_zle_keymap_select"

# Preserved widgets may replace psvar; Slick must repair its slots and stored prompt afterward.
SLICK_TEST_WIDGET_REPLACES_PSVAR=1
SLICK_TEST_RENDER_PSVAR_LITERAL=1

slick_prompt_zle_line_init >"$TEST_OUTPUT_FILE"
[[ "${psvar[1]}" == "other-hook-value" ]] || die "line-init should preserve replacement psvar values"
[[ "$(print -r -P -- "$PROMPT")" == 'literal:$' ]] || die "line-init should repair prompt literals after a chained widget"

slick_prompt_zle_keymap_select >"$TEST_OUTPUT_FILE"
[[ "${psvar[1]}" == "other-hook-value" ]] || die "keymap-select should preserve replacement psvar values"
[[ "$(print -r -P -- "$PROMPT")" == 'literal:$' ]] || die "keymap-select should repair prompt literals after a chained widget"

SLICK_PROMPT_TRANSIENT=1
slick_prompt_accept_line >"$TEST_OUTPUT_FILE"
[[ "${psvar[1]}" == "other-hook-value" ]] || die "accept-line should preserve replacement psvar values"
[[ "$(print -r -P -- "$PROMPT")" == 'literal:$' ]] || die "accept-line should repair prompt literals after a chained widget"

print -r -- "slick.zsh regression tests passed"
