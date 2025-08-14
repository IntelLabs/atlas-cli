#!/bin/bash
# This script contains common manifest creation functionality for Atlas demos and examples

create_dataset_manifest() {
    atlas-cli dataset create \
	      --paths="$1" \
	      --ingredient-names="$2" \
	      --name="$3" \
	      --author-org="$4" \
	      --author-name="$5" \
	      --storage-type=$STORAGE_BACKEND \
	      --storage-url=$STORAGE_URL \
	      --key=$SIGNING_KEY \
	      $EXTRA_CLI_FLAGS \
	      > $6
}

create_model_manifest() {
    atlas-cli model create \
	      --paths="$1" \
	      --ingredient-names="$2" \
	      --name="$3" \
	      --author-org="$4" \
	      --author-name="$5" \
	      --storage-type=$STORAGE_BACKEND \
	      --storage-url=$STORAGE_URL \
	      --key=$SIGNING_KEY \
	      $EXTRA_CLI_FLAGS \
	      > $6
}

create_software_manifest() {
    atlas-cli software create \
	      --paths="$1" \
	      --ingredient-names="$2" \
	      --name="$3" \
	      --software-type="$4" \
	      --version="$5" \
	      --author-org="$6" \
	      --author-name="$7" \
	      --description="$8" \
	      --storage-type=$STORAGE_BACKEND \
	      --storage-url=$STORAGE_URL \
	      --key=$SIGNING_KEY \
	      $EXTRA_CLI_FLAGS \
	      > $9
}

create_evaluation_manifest() {
    atlas-cli evaluation create \
	      --path="$1" \
	      --name="$2" \
	      --author-org="$3" \
	      --author-name="$4" \
	      --model-id="$5" \
	      --dataset-id="$6" \
	      --hash-alg=sha384 \
	      --key=private.pem \
	      --storage-type=$STORAGE_BACKEND \
	      --storage-url=$STORAGE_URL \
	      > $7
}

link_manifests() {
    atlas-cli manifest link \
	      --source="$1" \
	      --target="$2" \
	      --storage-type=$STORAGE_BACKEND \
	      --storage-url=$STORAGE_URL \
	      > $3
}
