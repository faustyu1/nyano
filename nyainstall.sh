#!/bin/sh

if ! command -v cargo >/dev/null 2>&1; then
    echo "Error: Cargo not found"
    echo "Use this command to install the Rust setup:"
    echo "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh && source \"\$HOME/.cargo/env\""
    rm -- "$0"
    exit 1
fi

git clone https://github.com/shareui/nyano
cd nyano
cargo build --release

echo "Installing to /usr/bin/nyano"
sudo install -m 755 target/release/nyano /usr/bin/nyano

printf '\033[32mSuccessfully installed to %s!\033[0m\n' "/usr/bin/nyano"
rm -- "$0"