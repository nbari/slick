#!/usr/bin/env zsh

set -eo pipefail

ROOT_DIR=${${(%):-%N}:A:h:h}
cd "$ROOT_DIR"

if [[ ! -x ./target/release/slick ]]; then
    print -u2 -- "error: expected ./target/release/slick to exist"
    exit 1
fi

die() {
    print -u2 -- "error: $1"
    exit 1
}

zsh -n ./slick.zsh || die "slick.zsh should parse"
zsh -n ./slick.plugin.zsh || die "slick.plugin.zsh should parse"
zsh -n ./load.zsh || die "load.zsh should parse"

NOOP_OUTPUT=$(PATH=/usr/bin:/bin HOME=/tmp/slick-loader-noop zsh -dfi -c 'source ./slick.zsh; print -r -- ${+functions[slick_prompt_preexec]}:${SLICK_PATH-unset}' 2>/dev/null)
[[ "$NOOP_OUTPUT" == "0:unset" ]] || die "slick.zsh should no-op quietly when slick is unavailable"

zle() {
    return 0
}

export SLICK_PATH="$ROOT_DIR/target/release/slick"
PLUGIN_OUTPUT_FILE=$(mktemp)
source ./slick.plugin.zsh >"$PLUGIN_OUTPUT_FILE"
PLUGIN_OUTPUT=$(<"$PLUGIN_OUTPUT_FILE")
rm -f "$PLUGIN_OUTPUT_FILE"
[[ -z "$PLUGIN_OUTPUT" ]] || die "slick.plugin.zsh should be quiet when sourced"
[[ "$SLICK_PATH" == "$ROOT_DIR/target/release/slick" ]] || die "slick.plugin.zsh should preserve an explicit SLICK_PATH"
[[ ${+functions[slick_prompt_preexec]} -eq 1 ]] || die "slick.plugin.zsh should source slick.zsh and define prompt hooks"
[[ ${+functions[slick_prompt_accept_line]} -eq 1 ]] || die "slick.plugin.zsh should expose the transient accept-line hook"

autoload -Uz add-zsh-hook
add-zsh-hook -D precmd slick_prompt_precmd
add-zsh-hook -D preexec slick_prompt_preexec

export SLICK_PATH="/tmp/should-be-overridden"
LOAD_OUTPUT_FILE=$(mktemp)
source ./load.zsh >"$LOAD_OUTPUT_FILE"
LOAD_OUTPUT=$(<"$LOAD_OUTPUT_FILE")
rm -f "$LOAD_OUTPUT_FILE"
[[ -z "$LOAD_OUTPUT" ]] || die "load.zsh should be quiet when sourced"
[[ "$SLICK_PATH" == "$ROOT_DIR/target/release/slick" ]] || die "load.zsh should prefer the local release binary"
[[ ${+functions[slick_prompt_preexec]} -eq 1 ]] || die "load.zsh should source slick.zsh and define prompt hooks"
[[ ${+functions[slick_prompt_accept_line]} -eq 1 ]] || die "load.zsh should expose the transient accept-line hook"

print -r -- "wrapper loader tests passed"
