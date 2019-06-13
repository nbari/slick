# slick

async ZSH prompt in Rust inspired by:

* [pure](https://github.com/sindresorhus/pure)
* [purs](https://github.com/xcambar/purs)
* [zsh-efgit-prompt](https://github.com/ericfreese/zsh-efgit-prompt)

## How to use

Install:

    cargo install slick

> check your PATH `$HOME/.cargo/bin/slick`

Then add this to your `.zshrc`:

typeset -g slick_prompt_data

```sh
function slick_prompt_refresh {
    if ! read -r slick_prompt_data <&$1; then
        slick_prompt_data=" "
    fi
    PROMPT=$($HOME/projects/rust/slick/target/debug/slick prompt -k "$KEYMAP" -r $? -d $slick_prompt_data)

    zle reset-prompt

    # Remove the handler and close the fd
    zle -F $1
    exec {1}<&-
}

function zle-line-init zle-keymap-select {
    PROMPT=$($HOME/projects/rust/slick/target/debug/slick prompt -k "$KEYMAP" -r $? -d $slick_prompt_data)
    zle && zle reset-prompt
}

function slick_prompt_precmd() {
    exec {FD}< <(
        $HOME/projects/rust/slick/target/debug/slick precmd
    )
    zle -F $FD slick_prompt_refresh
}

function slick_prompt_preexec() {
    typeset -g prompt_slick_cmd_timestamp=$EPOCHSECONDS
}

zle -N zle-line-init
zle -N zle-keymap-select
autoload -Uz add-zsh-hook
add-zsh-hook precmd slick_prompt_precmd
add-zsh-hook preexec slick_prompt_preexec
```
