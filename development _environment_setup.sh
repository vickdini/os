#!/bin/bash
apt install -y nasm build-essential make rustc
apt install -y cargo llvm binutils grub-efi-amd64-bin grub-pc-bin xorriso git
apt install -y rustup
rustup install nightly
rustup default nightly
rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
rustup target add x86_64-unknown-none
git clone https://github.com/vickdini/os.git
cd os
make
