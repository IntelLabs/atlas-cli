#!/bin/bash
# This script contains common manifest utilities for Atlas demos and examples

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
