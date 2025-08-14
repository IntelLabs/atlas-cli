#!/bin/bash
# This script contains common manifest verification functionality for Atlas demos and examples

validate_manifest() {
    atlas-cli manifest validate \
	      --id="$1" \
	      --storage-type=$STORAGE_BACKEND \
	      --storage-url=$STORAGE_URL
}

verify_linked_manifests() {
    atlas-cli manifest verify-link \
	      --source="$1" \
	      --target="$2" \
	      --storage-type=$STORAGE_BACKEND \
	      --storage-url=$STORAGE_URL    
}
