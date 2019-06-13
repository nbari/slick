# slick

async ZSH prompt in Rust inspired by:

* [pure](https://github.com/sindresorhus/pure)
* [purs](https://github.com/xcambar/purs)
* [zsh-efgit-prompt](https://github.com/ericfreese/zsh-efgit-prompt)

![sreenshot](./prompt.png)

## How to use

Install:

    cargo install slick

> check your PATH `$HOME/.cargo/bin/slick`

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

function slick_prompt_refresh {
    if ! read -r slick_prompt_data <&$1; then
        slick_prompt_data=" "
    fi
    PROMPT=$(slick prompt -k "$KEYMAP" -r $? -d ${slick_prompt_data:-" "} -t ${slick_prompt_timestamp:-$EPOCHSECONDS})

    zle reset-prompt

    # Remove the handler and close the fd
    zle -F $1
    exec {1}<&-
}

function zle-line-init zle-keymap-select {
    PROMPT=$(slick prompt -k "$KEYMAP" -r $? -d ${slick_prompt_data:-" "})
    zle && zle reset-prompt
}

function slick_prompt_precmd() {
    exec {FD}< <(slick precmd)
    zle -F $FD slick_prompt_refresh
}

function slick_prompt_preexec() {
    typeset -g slick_prompt_timestamp=$EPOCHSECONDS
}
```
