# slick - async ZSH prompt

[![crates.io](https://img.shields.io/crates/v/slick.svg)](https://crates.io/crates/slick)
[![Build Status](https://github.com/nbari/slick/workflows/build/badge.svg)](https://github.com/nbari/slick/actions?query=workflow%3Abuild)


[![example](https://img.youtube.com/vi/ZFQ2bykpm6s/0.jpg)](https://www.youtube.com/watch?v=ZFQ2bykpm6s)

## How to use

Install:

    cargo install slick

To install cargo:

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

If in Linux you may need install this:

    apt install -y build-essential libssl-dev pkg-config

check your PATH `$HOME/.cargo/bin/slick`:w

Then add this to your `.zshrc`:

```sh
zle -N zle-keymap-select
zle -N zle-line-init
zmodload zsh/datetime
autoload -Uz add-zsh-hook
add-zsh-hook precmd slick_prompt_precmd
add-zsh-hook preexec slick_prompt_preexec

typeset -g slick_prompt_data
typeset -g slick_prompt_timestamp

SLICK_PATH=$HOME/.cargo/bin/slick

function slick_prompt_refresh {
    local exit_status=$?
    read -r -u $1 slick_prompt_data
    PROMPT=$($SLICK_PATH prompt -k "$KEYMAP" -r $exit_status -d ${slick_prompt_data:-""} -t ${slick_prompt_timestamp:-$EPOCHSECONDS})
    unset slick_prompt_timestamp

    zle reset-prompt

    # Remove the handler and close the fd
    zle -F $1
    exec {1}<&-
}

function zle-line-init zle-keymap-select {
    PROMPT=$($SLICK_PATH prompt -k "$KEYMAP" -d ${slick_prompt_data:-""})
    zle && zle reset-prompt
}

function slick_prompt_precmd() {
    slick_prompt_data=""
    local fd
    exec {fd}< <($SLICK_PATH precmd)
    zle -F $fd slick_prompt_refresh
}

function slick_prompt_preexec() {
    slick_prompt_timestamp=$EPOCHSECONDS
}
```

## customizations

Set this environment variables to change color/symbols, for example:

    export SLICK_PROMPT_CMD_MAX_EXEC_TIME=3
    export SLICK_PROMPT_ERROR_COLOR=88
    export SLICK_PROMPT_GIT_ACTION_COLOR=1
    export SLICK_PROMPT_GIT_BRANCH_COLOR=202
    export SLICK_PROMPT_GIT_FETCH=0
    export SLICK_PROMPT_GIT_MASTER_BRANCH_COLOR=white
    export SLICK_PROMPT_GIT_REMOTE_AHEAD=""
    export SLICK_PROMPT_GIT_REMOTE_BEHIND=""
    export SLICK_PROMPT_GIT_REMOTE_COLOR=40
    export SLICK_PROMPT_GIT_STAGED_COLOR=1
    export SLICK_PROMPT_GIT_STATUS_COLOR=cyan
    export SLICK_PROMPT_GIT_UNAME_COLOR=8
    export SLICK_PROMPT_PATH_COLOR=blue
    export SLICK_PROMPT_ROOT_COLOR="red"
    export SLICK_PROMPT_ROOT_SYMBOL="#"
    export SLICK_PROMPT_SSH_COLOR=2
    export SLICK_PROMPT_SYMBOL="❯"
    export SLICK_PROMPT_SYMBOL_COLOR=magenta
    export SLICK_PROMPT_TIME_ELAPSED_COLOR=1
    export SLICK_PROMPT_VICMD_COLOR="yellow"
    export SLICK_PROMPT_VICMD_SYMBOL="❮"
    export SLICK_PROMPT_NON_BREAKING_SPACE=" "


`SLICK_PROMPT_GIT_FETCH=0` prevents doing a `git fetch`

To prevent displaying the git user.name:

    export SLICK_PROMPT_NO_GIT_UNAME=1

___
Inspired by:

* [pure](https://github.com/sindresorhus/pure)
* [purs](https://github.com/xcambar/purs)
* [zsh-efgit-prompt](https://github.com/ericfreese/zsh-efgit-prompt)
