#!/bin/bash
set -e

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "Building from directory: $SCRIPT_DIR"

# Source cargo environment if it exists
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

# Ensure wasm32 target is installed
echo "Adding wasm32-unknown-unknown target..."
rustup target add wasm32-unknown-unknown || true

# Install or update trunk
echo "Installing trunk..."
cargo install --locked trunk || echo "Trunk already installed or installation skipped"

# Build the project
echo "Building project with trunk..."
trunk build --release

echo "Build complete! Output in $SCRIPT_DIR/dist"
