#!/bin/bash
# MNIST Demo Provenance Collection Script
# This script runs a demo MNIST workflow (no data prep or training) and collects
# provenance data

source ../common/config.sh
source ../common/keys.sh
source ../common/utils.sh

TRAIN_DATASET="train-00000-of-00001.parquet"
TEST_DATASET="test-00000-of-00001.parquet"

if_file_not_exists_do "$TRAIN_DATASET" "wget -q https://huggingface.co/datasets/ylecun/mnist/resolve/main/mnist/$TRAIN_DATASET"

if_file_not_exists_do "$TEST_DATASET" "wget -q https://huggingface.co/datasets/ylecun/mnist/resolve/main/mnist/$TEST_DATASET"

echo -e "=== STEP 0: Setup Provenance Signing/Verification Key Pair ==="
generate_signing_keys

wait_for_user "Press any key to continue"

echo -e "\n=== STEP 1: Generate Provenance for MNIST Training Data ==="

wait_for_user "Create training dataset manifest..."
atlas-cli dataset create \
    --paths="$TRAIN_DATASET" \
    --ingredient-names="MNIST Training Dataset" \
    --name="MNIST Training Data" \
    --author-org="https://huggingface.co/datasets/ylecun/mnist/tree/main/mnist/blob/main/mnist/$TRAIN_DATASET" \
    --author-name="ylecun" \
    --storage-type=database \
    --storage-url=$STORAGE_URL \
    --key=private.pem \
    > train_dataset_output.txt
TRAIN_DATASET_ID=$(extract_c2pa_id train_dataset_output.txt)
echo "Dataset ID: $TRAIN_DATASET_ID"

wait_for_user "Display training data manifest"
atlas-cli manifest export \
	  --id=$TRAIN_DATASET_ID \
	  --format=json \
	  | jq '.'

wait_for_user "Press any key to continue"

echo -e "\n=== STEP 2: Generate Provenance for Model Training Artifacts ==="

wait_for_user "Create training script manifest..."
atlas-cli software create \
    --paths=../mnist/train.py \
    --ingredient-names="MNIST Training Script" \
    --name="MNIST CNN Training Implementation" \
    --software-type="script" \
    --version="1.0.0" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --description="PyTorch training script for MNIST CNN model" \
    --with-tdx \
    --key=private.pem \
    --storage-type=database \
    --storage-url=$STORAGE_URL \
    > training_script_output.txt
TRAINING_SCRIPT_ID=$(extract_c2pa_id training_script_output.txt)
echo "Training Script ID: $TRAINING_SCRIPT_ID"

touch classifier.onnx
wait_for_user "Create model manifest..."
atlas-cli model create \
    --paths=classifier.onnx \
    --ingredient-names="MNIST CNN Model" \
    --name="Trained MNIST Classifier" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --key=private.pem \
    --storage-type=database \
    --storage-url=$STORAGE_URL \
    > model_output.txt
MODEL_ID=$(extract_c2pa_id model_output.txt)
echo "Model ID: $MODEL_ID"

wait_for_user "Display model's manifest"
atlas-cli manifest export \
	  --id=$MODEL_ID \
	  --format=json \
	  | jq '.'

wait_for_user "Press any key to continue"

echo -e "\n=== STEP 3: Link Model Training Manifests ==="

wait_for_user "Link MNIST training dataset to model..."
atlas-cli manifest link \
	  --source=$MODEL_ID \
	  --target=$TRAIN_DATASET_ID \
	  --storage-type=database \
	  --storage-url=$STORAGE_URL \
	  > model_train_dataset_link_output.txt
MODEL_ID=$(extract_c2pa_id model_train_dataset_link_output.txt)
echo "Updated Model ID: $MODEL_ID"

wait_for_user "Link training script to model..."
atlas-cli manifest link \
	  --source=$MODEL_ID \
	  --target=$TRAINING_SCRIPT_ID \
	  --storage-type=database \
	  --storage-url=$STORAGE_URL \
	  > model_train_script_link_output.txt
MODEL_ID=$(extract_c2pa_id model_train_script_link_output.txt)
echo "Updated Model ID: $MODEL_ID"

wait_for_user "Display model's manifest"
atlas-cli manifest export \
	  --id=$MODEL_ID \
	  --format=json \
	  | jq '.'

wait_for_user "Press any key to continue"

echo -e "\n=== STEP 4: Generate & Link Provenance for Model Evaluation Artifacts ==="

atlas-cli dataset create \
    --paths="$TEST_DATASET" \
    --ingredient-names="MNIST Training Dataset" \
    --name="MNIST Training Data" \
    --author-org="https://huggingface.co/datasets/ylecun/mnist/tree/main/mnist/blob/main/mnist/$TEST_DATASET" \
    --author-name="ylecun" \
    --storage-type=database \
    --storage-url=$STORAGE_URL \
    --key=private.pem \
    > test_dataset_output.txt
TEST_DATASET_ID=$(extract_c2pa_id test_dataset_output.txt)
echo "Test Dataset ID: $TEST_DATASET_ID"

atlas-cli software create \
    --paths=../mnist/eval.py \
    --ingredient-names="MNIST Evaluation Script" \
    --name="MNIST Model Evaluation Implementation" \
    --software-type="script" \
    --version="1.0.0" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --description="PyTorch evaluation script for MNIST CNN model" \
    --with-tdx \
    --key=private.pem \
    --storage-type=database \
    --storage-url=$STORAGE_URL \
    > eval_script_output.txt
EVAL_SCRIPT_ID=$(extract_c2pa_id eval_script_output.txt)
echo "Evaluation Script ID: $EVAL_SCRIPT_ID"

touch eval_results.json
echo "Creating evaluation results manifest linked to model..."
atlas-cli evaluation create \
    --path=eval_results.json \
    --name="MNIST Model Evaluation Results" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --model-id=$MODEL_ID \
    --dataset-id=$TEST_DATASET_ID \
    --hash-alg=sha384 \
    --key=private.pem \
    --storage-type=database \
    --storage-url=$STORAGE_URL \
    > eval_results_output.txt
EVAL_RESULTS_ID=$(extract_c2pa_id eval_results_output.txt)
echo "Evaluation Results ID: $EVAL_RESULTS_ID"

atlas-cli manifest link \
	  --source=$EVAL_RESULTS_ID \
	  --target=$EVAL_SCRIPT_ID \
	  --storage-type=database \
	  --storage-url=$STORAGE_URL \
	  > eval_script_link_output.txt
EVAL_RESULTS_ID=$(extract_c2pa_id eval_script_link_output.txt)
echo "Updated Eval Results ID: $EVAL_RESULTS_ID"

wait_for_user "Press any key to continue"

echo -e "\n=== STEP 4: Export Provenance Graph ==="
atlas-cli manifest export \
    --id=$EVAL_RESULTS_ID \
    --storage-type=database \
    --storage-url=$STORAGE_URL \
    --format=json \
    --max-depth=10 \
    --output=mnist_provenance.json

wait_for_user "Press any key to continue"

echo -e "\n=== STEP 5: Validate Provenance ==="

wait_for_user "Validate model manifest..."
atlas-cli manifest validate \
	  --id=$MODEL_ID \
	  --storage-type=database \
	  --storage-url=$STORAGE_URL

wait_for_user "Validate evaluation results manifest..."
atlas-cli manifest validate \
	  --id=$EVAL_RESULTS_ID \
	  --storage-type=database \
	  --storage-url=$STORAGE_URL

INVALID_LINKED_MANIFEST_ID="urn:c2pa:123e4567-e89b-12d3-a456-426614174000"

wait_for_user "Validate bad manifest link (should fail)..."
atlas-cli manifest verify-link \
	  --source=$MODEL_ID \
	  --target=$INVALID_LINKED_MANIFEST_ID \
	  --storage-type=database \
	  --storage-url=$STORAGE_URL

wait_for_user "Display exported evaluation results provenance"
echo -e "\n"
cat mnist_provenance.json | jq '.'

wait_for_user "Finish demo"
echo -e "\n"
remove_signing_keys
rm -f *_output.txt classifier.onnx eval_results.json mnist_provenance.json *.parquet
