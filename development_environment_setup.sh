#!/bin/bash
apt update
apt install -y build-essential make xorriso mtools dosfstools git curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
rustup target add x86_64-unknown-uefi

# Build
make
