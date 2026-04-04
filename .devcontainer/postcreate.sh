#!/usr/bin/env bash
set -euo pipefail

# Fix all volume ownership upfront
sudo chown -R "$(id -u):$(id -g)" \
    /home/vscode/.cargo \
    /home/vscode/.rustup

sudo apt-get update
sudo apt-get install -y \
    pkg-config libssl-dev \
    ripgrep fd-find delta just \
    curl wget ca-certificates \
    direnv fzf

sudo chsh -s /usr/bin/zsh vscode

rustup update stable
rustup default stable
rustup component add rustfmt clippy rust-analyzer

# slick
SLICK_URL=$(curl -sf https://api.github.com/repos/nbari/slick/releases/latest |
    grep "browser_download_url.*amd64.deb" |
    cut -d: -f2,3 |
    tr -d '"' |
    xargs)

if [[ -n "$SLICK_URL" ]]; then
    wget -qO /tmp/slick.deb "$SLICK_URL"
    sudo dpkg -i /tmp/slick.deb
    rm /tmp/slick.deb
else
    echo "WARNING: could not resolve slick release URL, skipping"
fi

bash .devcontainer/setup-zsh.sh
