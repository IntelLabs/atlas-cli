#!/bin/bash
# This script does an offline collection of a C2PA manifest in OMS format for
# a trained model with many ingredients

# Configuration
source common/config.sh
source common/keys.sh

echo -e "Generate Provenance Signing/Verification Key Pair"
generate_signing_keys

echo -e "Download multi-file model"
git clone https://huggingface.co/ByteDance/Dolphin

echo -e "Generate C2PA manifest in OMS format"
../target/debug/atlas-cli model create \
			  --paths=Dolphin/config.json,Dolphin/generation_config.json,Dolphin/preprocessor_config.json,Dolphin/special_tokens_map.json,Dolphin/tokenizer.json,Dolphin/tokenizer_config.json \
			  --ingredient-names="config.json","generation_config.json","preprocessor_config.json","special_tokens_map.json","tokenizer.json","tokenizer_config.json" \
			  --name="https://huggingface.co/ByteDance/Dolphin/tree/main" \
			  --author-name="ByteDance" \
			  --key=$SIGNING_KEY \
			  --encoding=json \
			  --format=oms \
			  --hash-alg=sha256 \
			  --print > test.dsse

echo -e "Output the generated in-toto Statement"
go install github.com/adityasaky/essd@latest
essd cat -p -d test.dsse | jq '.'

echo "Cleanup"
remove_signing_keys
rm -f classifier.onnx test.dsse
rm -rf Dolphin
