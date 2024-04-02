#!/bin/bash

set -e

# Check if Nix is installed
if ! command -v nix-env &> /dev/null; then
    echo "Nix is not installed. Installing..."
    # Install Nix
    sh <(curl -L https://nixos.org/nix/install) --no-daemon
fi

# Enter the project-specific environment
if [ -e "shell.nix" ]; then
    echo "Entering project-specific environment..."
    nix-shell
else
    echo "No shell.nix found in the current directory."
    echo "Please create a shell.nix file to specify the project environment."
fi