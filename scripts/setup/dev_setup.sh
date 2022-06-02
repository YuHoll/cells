#!/bin/bash
# Copyright 2022 Cells Project Authors. Licensed under Apache-2.0.

SCRIPT_PATH="$( cd  "$( dirname "$0" )" >/dev/null 2>&1 && pwd)"
cd "$SCRIPT_PATH/../.." || exit

# install rustup function
function install_rustup {

    echo "Installing Rust ..."
    if rustup --version &>/dev/null; then
        echo "Rust is already installed"
    else
        curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
        PATH="${HOME}/.cargo/bin:${PATH}"
        source $HOME/.cargo/env
    fi
}

# install rust toolchain
function install_toolchain {
    version=$1
    echo "Installing ${version} of rust toolchain"
    rustup install "$version"
    rustup set profile minimal
    rustup component add llvm-tools-preview
    rustup component add clippy
    rustup default "$version"
}


# install dev env
function install_dev_env {
    if [ -z "$(which cargo-make)" ]; then
        echo "Installing cargo-make..."
        cargo install cargo-make --version "^0.35"
    fi
}

install_rustup
install_toolchain "$(cat ./rust-toolchain)"
install_dev_env

exit 0
