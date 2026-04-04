#!/bin/zsh
# Development/testing wrapper for slick prompt

0=${(%):-%N}
if [[ -x "${0:A:h}/target/release/slick" ]]; then
    export SLICK_PATH="${0:A:h}/target/release/slick"
fi

source "${0:A:h}/slick.zsh"
