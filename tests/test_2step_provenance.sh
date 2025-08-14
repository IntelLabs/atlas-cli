#!/bin/bash
# This script runs a mock MNIST workflow (no data prep or training) and collects
# and verifies provenance data

# Configuration
source ../examples/common/config.sh
source ../examples/common/keys.sh
source ../examples/common/utils.sh
source helpers/manifest_utils.sh
source helpers/manifest_create.sh
source helpers/manifest_verify.sh

# launch storage service
cd ../storage_service && docker-compose up -d

TRAIN_DATASET="train-00000-of-00001.parquet"
TEST_DATASET="test-00000-of-00001.parquet"

if_file_not_exists_do "$TRAIN_DATASET" "wget -q https://huggingface.co/datasets/ylecun/mnist/resolve/main/mnist/$TRAIN_DATASET"

if_file_not_exists_do "$TEST_DATASET" "wget -q https://huggingface.co/datasets/ylecun/mnist/resolve/main/mnist/$TEST_DATASET"

echo -e "=== STEP 0: Setup Provenance Signing/Verification Key Pair ==="
generate_signing_keys

echo -e "\n=== STEP 1: Generate Provenance for MNIST Training Data ==="
create_dataset_manifest \
    $TRAIN_DATASET \
    "MNIST Training Dataset" \
    "MNIST Training Data" \
    "https://huggingface.co/datasets/ylecun/mnist/tree/main/mnist/blob/main/mnist/$TRAIN_DATASET" \
    "ylecun" \
    train_dataset_output.txt
TRAIN_DATASET_ID=$(extract_c2pa_id train_dataset_output.txt)
echo "Dataset ID: $TRAIN_DATASET_ID"

echo -e "\n=== STEP 2: Generate Provenance for Model Training Artifacts ==="

EXTRA_CLI_FLAGS=--with-tdx
create_software_manifest \
    "../examples/mnist/train.py" \
    "MNIST Training Script" \
    "MNIST CNN Training Implementation" \
    "script" \
    "1.0.0" \
    "Your Organization" \
    "Your Name" \
    "PyTorch training script for MNIST CNN model" \
    training_script_output.txt
TRAINING_SCRIPT_ID=$(extract_c2pa_id training_script_output.txt)
echo "Training Script ID: $TRAINING_SCRIPT_ID"

EXTRA_CLI_FLAGS=

touch classifier.onnx
create_model_manifest \
    classifier.onnx \
    "MNIST CNN Model" \
    "Trained MNIST Classifier" \
    "Your Organization" \
    "Your Name" \
    model_output.txt
MODEL_ID=$(extract_c2pa_id model_output.txt)
echo "Model ID: $MODEL_ID"

echo -e "\n=== STEP 3: Link Model Training Manifests ==="

echo "Link MNIST training dataset to model"
link_manifests \
    $MODEL_ID \
    $TRAIN_DATASET_ID \
    model_train_dataset_link_output.txt
MODEL_ID=$(extract_c2pa_id model_train_dataset_link_output.txt)
echo "Updated Model ID: $MODEL_ID"

echo "Link training script to model"
link_manifests \
    $MODEL_ID \
    $TRAINING_SCRIPT_ID \
    model_train_script_link_output.txt
MODEL_ID=$(extract_c2pa_id model_train_script_link_output.txt)
echo "Updated Model ID: $MODEL_ID"

echo -e "\n=== STEP 4: Generate & Link Provenance for Model Evaluation Artifacts ==="

echo "Generate test dataset manifest"
create_dataset_manifest \
    $TEST_DATASET \
    "MNIST Test Dataset" \
    "MNIST Test Data" \
    "https://huggingface.co/datasets/ylecun/mnist/tree/main/mnist/blob/main/mnist/$TEST_DATASET" \
    "ylecun" \
    test_dataset_output.txt
TEST_DATASET_ID=$(extract_c2pa_id test_dataset_output.txt)
echo "Test Dataset ID: $TEST_DATASET_ID"

echo "Generate evaluation script manifest"
EXTRA_CLI_FLAGS=--with-tdx
create_software_manifest \
    "../examples/mnist/eval.py" \
    "MNIST Evaluation Script" \
    "MNIST Model Evaluation Implementation" \
    "script" \
    "1.0.0" \
    "Your Organization" \
    "Your Name" \
    "PyTorch evaluation script for MNIST CNN model" \
    eval_script_output.txt
EVAL_SCRIPT_ID=$(extract_c2pa_id eval_script_output.txt)
echo "Evaluation Script ID: $EVAL_SCRIPT_ID"

EXTRA_CLI_FLAGS=
touch eval_results.json
echo "Creating evaluation results manifest linked to model"
create_evaluation_manifest \
    "eval_results.json" \
    "MNIST Model Evaluation Results" \
    "Your Organization" \
    "Your Name" \
    $MODEL_ID \
    $TEST_DATASET_ID \
    eval_results_output.txt
EVAL_RESULTS_ID=$(extract_c2pa_id eval_results_output.txt)
echo "Evaluation Results ID: $EVAL_RESULTS_ID"

link_manifests \
    $EVAL_RESULTS_ID \
    $EVAL_SCRIPT_ID \
    eval_script_link_output.txt
EVAL_RESULTS_ID=$(extract_c2pa_id eval_script_link_output.txt)
echo "Updated Eval Results ID: $EVAL_RESULTS_ID"

echo -e "\n=== STEP 4: Export Provenance Graph ==="
export_manifest_json \
    $EVAL_RESULTS_ID \
    mnist_provenance.json

echo -e "\n=== STEP 5: Validate Provenance ==="

echo "Validate model manifest..."
validate_manifest $MODEL_ID

echo "Validate evaluation results manifest..."
validate_manifest $EVAL_RESULTS_ID

echo "Display exported evaluation results provenance"
cat mnist_provenance.json | jq '.'

echo "Cleanup"
remove_signing_keys
rm -f *_output.txt classifier.onnx eval_results.json mnist_provenance.json
 cd ../storage_service && docker-compose down
