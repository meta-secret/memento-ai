#!/bin/bash

set -e

# Check if Nix is installed
if ! command -v nix-env &> /dev/null; then
    echo "Nix is not installed. Installing..."
    # Install Nix
    sh <(curl -L https://nixos.org/nix/install) --no-daemon
    . "${HOME}"/.nix-profile/etc/profile.d/nix.sh
fi
