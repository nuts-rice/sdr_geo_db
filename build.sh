#!/bin/bash
set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "Building from directory: $SCRIPT_DIR"

if ! command -v rustup &> /dev/null; then
    echo "Rust not found, installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
    source "$HOME/.cargo/env"
else
    echo "Rust already installed"
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
fi

if ! command -v cargo &> /dev/null; then
    echo "ERROR: cargo not found after installation"
    exit 1
fi

echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"

echo "Adding wasm32-unknown-unknown target..."
rustup target add wasm32-unknown-unknown

if ! command -v trunk &> /dev/null; then
    echo "Installing trunk..."
    cargo install --locked trunk
else
    echo "Trunk already installed: $(trunk --version)"
fi

echo "Building project with trunk..."
cd ./sdr_db_website/
trunk build --release

echo "Build complete! Output in $SCRIPT_DIR/dist"

ARGO_OS="darwin"
if [[ "$(uname -s)" != "Darwin" ]]; then
  ARGO_OS="linux"
fi

# Download the binary
curl -sLO "https://github.com/argoproj/argo-workflows/releases/download/v4.0.0-rc2/argo-$ARGO_OS-amd64.gz"

# Unzip
gunzip "argo-$ARGO_OS-amd64.gz"

# Make binary executable
chmod +x "argo-$ARGO_OS-amd64"

# Move binary to path
mv "./argo-$ARGO_OS-amd64" /usr/local/bin/argo

# Test installation
argo version
