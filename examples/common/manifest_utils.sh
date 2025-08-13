#!/bin/bash
# This script contains common manifest utilities for Atlas demos and examples

# Helper function to extract ID from output
extract_c2pa_id() {
    grep -o "ID: [^ ]*" "$1" | cut -d' ' -f2
}

display_manifest_json() {
    atlas-cli manifest export \
	      --id="$1" \
	      --format=json \
	| jq '.'
}

export_manifest_json() {
    atlas-cli manifest export \
	      --id="$1" \
	      --storage-type=$STORAGE_BACKEND \
	      --storage-url=$STORAGE_URL \
	      --format=json \
	      --max-depth=10 \
	      --output="$2"   
}
