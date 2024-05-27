#!/bin/bash

set -euo pipefail

# Prompt the user to enter a password without echoing it to the terminal
echo -n "Enter your password: "
read -sr password
echo

# Calculate the SHA-256 hash of the password
echo -n "$password" | sha256sum | awk '{print $1}'

echo "Screen will be cleared in (here) 30 seconds."

for _ in {1..30}; do read -rs -n1 -t1 || printf ".";done;echo

clear
