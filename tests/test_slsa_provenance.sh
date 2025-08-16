#!/bin/bash
# This script does an offline collection of SLSA Build Provenance for a training
# pipeline verifies provenance data

# Configuration
source ../examples/common/config.sh
source ../examples/common/keys.sh

echo -e "Generate Provenance Signing/Verification Key Pair"
generate_signing_keys

touch classifier.onnx
echo -e "Generate SLSA Build Provenance"
../target/debug/atlas-cli pipeline generate-provenance --pipeline ../examples/mnist/train.py --inputs ../examples/oss-na-25-demo/train-00000-of-00001.parquet --products classifier.onnx --storage-url outputs --print --key $SIGNING_KEY > test.dsse

echo -e "Output the generated SLSA Provenance"
go install github.com/adityasaky/essd@latestewrlwerlkjwlkj
essd cat -p -d test.dsse | jq '.'

echo "Cleanup"
remove_signing_keys
rm -f classifier.onnx test.dsse
rm -rf outputs
