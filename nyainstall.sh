#!/bin/sh

if ! command -v cargo >/dev/null 2>&1; then
    echo "Error: Cargo not found"
    echo "Use this command to install the Rust setup:"
    echo "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh && source \"\$HOME/.cargo/env\""
    rm -- "$0"
    exit 1
fi

# /usr/bin is protected by SIP on macOS; prefer /usr/local/bin, fall back to ~/.local/bin
if [ -w /usr/local/bin ]; then
    INSTALL_DIR="/usr/local/bin"
elif sudo -n true 2>/dev/null; then
    INSTALL_DIR="/usr/local/bin"
    USE_SUDO=1
elif which doas 2>/dev/null; then
    INSTALL_DIR="/usr/local/bin"
    USE_DOAS=1
else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

INSTALL_PATH="$INSTALL_DIR/nyano"

git clone https://github.com/shareui/nyano nyano-src
cd nyano-src
cargo build --release

echo "Installing to $INSTALL_PATH"
if [ -n "$USE_SUDO" ]; then
    sudo install -m 755 target/release/nyano "$INSTALL_PATH"
elif [ -n "$USE_DOAS" ]; then
    doas install -m 755 target/release/nyano "$INSTALL_PATH"
else
    install -m 755 target/release/nyano "$INSTALL_PATH"
fi

cd ..
rm -rf nyano-src
# When run through `curl | sh`, $0 is "sh" rather than a script path.
[ -f "$0" ] && rm -- "$0"

printf '\033[32mSuccessfully installed to %s!\033[0m\n' "$INSTALL_PATH"

# Remind the user if the install dir is not in PATH
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) printf '\033[33mNote: add %s to your PATH:\n  export PATH="%s:$PATH"\033[0m\n' "$INSTALL_DIR" "$INSTALL_DIR" ;;
esac
