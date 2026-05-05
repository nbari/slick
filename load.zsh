#!/bin/zsh
# Development/testing wrapper for slick prompt

0=${(%):-%N}
if [[ -n "$CARGO_TARGET_DIR" ]]; then
    local slick_binary="$CARGO_TARGET_DIR/release/slick"
else
    local slick_binary="${0:A:h}/target/release/slick"
fi

if [[ -x "$slick_binary" ]]; then
    export SLICK_PATH="$slick_binary"
fi

source "${0:A:h}/slick.zsh"
