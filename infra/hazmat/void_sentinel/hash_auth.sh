#!/bin/bash

set -euo pipefail

mkdir -p tmp

# Prompt the user to enter a password without echoing it to the terminal
echo -n "Enter your password: "
read -sr password
echo

# Calculate the SHA-256 hash of the password
HASH=$(echo -n "$password" | sha256sum | awk '{print $1}')

echo "${HASH}"> tmp/master_pass.txt
