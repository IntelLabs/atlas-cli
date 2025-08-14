#!/bin/bash
# This script contains common manifest utilities for Atlas demos and examples

# Helper function to extract ID from output
extract_c2pa_id() {
    grep -o "ID: [^ ]*" "$1" | cut -d' ' -f2
}

# Check if a set of files exists. If it does not, take an optional action
if_file_not_exists_do() {
    if [ ! -e "$1" ]; then
	echo "Warning: $1 does not exist"
	"$2"
    fi
} 

# Prompt and wait for user key press
wait_for_user() {
    read -s -r -p "$1"
}
