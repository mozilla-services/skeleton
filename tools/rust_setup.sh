#! /bin/bash

if ! [ -e $HOME/.cargo/bin/rustup ]
then
    echo "rustup not found. Please either run 'rustup_install.sh' or 'curl https://sh.rustup.rs -sSf | sh]'"
    return
fi
## Fetch rustup if not installed.
# curl https://sh.rustup.rs -sSf | sh
## CHANNEL is the default channel for this project ('stable', 'nightly', etc.)
## You only need the following if you're planning on using a non-stable channel
#$CHANNEL = stable
#rustup install $CHANNEL
#rustup update $CHANNEL
## use Nightly with this project
#rustup override set $CHANNEL
## if you want clippy, the rust linter:
# cargo component add clippy-preview
## if you want to use rust-wasm, uncomment
#cargo install -f cargo-web
