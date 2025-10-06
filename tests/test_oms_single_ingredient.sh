#!/bin/bash
# This script does an offline collection of a C2PA manifest in OMS format for
# a trained model with a single ingredient

# Configuration
source common/config.sh
source common/keys.sh

echo -e "Generate Provenance Signing/Verification Key Pair"
generate_signing_keys

touch classifier.onnx
echo -e "Generate C2PA manifest in OMS format"
../target/debug/atlas-cli model create \
			  --paths=classifier.onnx \
			  --ingredient-names="MNIST CNN Model" \
			  --name="Trained MNIST Classifier" \
			  --author-org="Your Organization" \
			  --author-name="Your Name" \
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
