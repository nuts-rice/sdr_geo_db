#!/bin/bash
set -e

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "Building from directory: $SCRIPT_DIR"

# Check if rustup is installed, if not install Rust
if ! command -v rustup &> /dev/null; then
    echo "Rust not found, installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
    # Source the newly installed cargo environment
    source "$HOME/.cargo/env"
else
    echo "Rust already installed"
    # Source cargo environment if it exists
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
fi

# Verify cargo is now available
if ! command -v cargo &> /dev/null; then
    echo "ERROR: cargo not found after installation"
    exit 1
fi

echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"

# Ensure wasm32 target is installed
echo "Adding wasm32-unknown-unknown target..."
rustup target add wasm32-unknown-unknown

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "Installing trunk..."
    cargo install --locked trunk
else
    echo "Trunk already installed: $(trunk --version)"
fi

# Build the project
echo "Building project with trunk..."
cd ./sdr_db_website/
trunk build --release

echo "Build complete! Output in $SCRIPT_DIR/dist"
