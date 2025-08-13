#!/bin/bash
# MNIST Provenance Collection Script
# This script runs the complete MNIST workflow and collects provenance data

# Configuration
source ../common/config.sh
source ../common/keys.sh
source ../common/manifest_utils.sh
source ../common/manifest_create.sh
source ../common/manifest_verify.sh

echo "=== STEP 1: Download MNIST Dataset ==="
poetry run python download.py --path_to_output ./output/data

FILES=(
  ./output/data/MNIST/raw/t10k-images-idx3-ubyte.gz
  ./output/data/MNIST/raw/t10k-labels-idx1-ubyte.gz
  ./output/data/MNIST/raw/train-images-idx3-ubyte.gz
  ./output/data/MNIST/raw/train-labels-idx1-ubyte.gz
)

for f in "${FILES[@]}"; do
  if [ ! -e "$f" ]; then
    echo "Warning: $f does not exist"
  fi
done

DATAPATHS=$(printf "%s\n" "${FILES[@]}" | xargs realpath 2>/dev/null | paste -sd, -)

echo "Creating dataset manifest..."
create_dataset_manifest \
    "$DATAPATHS" \
    "MNIST Dataset" \
    "MNIST Training and Test Data" \
    "Your Organization" \
    "Your Name" \
    dataset_output.txt
DATASET_ID=$(extract_id dataset_output.txt)
echo "Dataset ID: $DATASET_ID"

echo -e "\n=== STEP 2: Train the Model ==="
poetry run python train.py \
    --path_to_data ./output/data \
    --path_to_output ./output/train \
    --batch_size 128 \
    --lr 0.5 \
    --epochs 1 \
    --use_cuda false

echo "Creating training script manifest linked to dataset..."
EXTRA_CLI_FLAGS="--linked-manifests=$DATASET_ID"
create_software_manifest \
    "train.py" \
    "MNIST Training Script" \
    "MNIST CNN Training Implementation" \
    "script" \
    "1.0.0" \
    "Your Organization" \
    "Your Name" \
    "PyTorch training script for MNIST CNN model" \
    training_script_output.txt
TRAINING_SCRIPT_ID=$(extract_id training_script_output.txt)
echo "Training Script ID: $TRAINING_SCRIPT_ID"

EXTRA_CLI_FLAGS=
echo "Creating training configuration manifest..."
create_dataset_manifest \
    "./output/train/training_conf.json" \
    "Training Configuration" \
    "MNIST Training Configuration" \
    "Your Organization" \
    "Your Name" \
    training_config_output.txt
TRAINING_CONFIG_ID=$(extract_id training_config_output.txt)
echo "Training Config ID: $TRAINING_CONFIG_ID"

EXTRA_CLI_FLAGS="--linked-manifests=$DATASET_ID"
echo "Creating model manifest linked to dataset..."
create_model_manifest \
    "./output/train/model.pkl" \
    "MNIST CNN Model" \
    "Trained MNIST Classifier" \
    "Your Organization" \
    "Your Name" \
    model_output.txt
MODEL_ID=$(extract_id model_output.txt)
echo "Model ID: $MODEL_ID"

echo -e "\n=== STEP 3: Evaluate the Model ==="
poetry run python eval.py \
    --path_to_data ./output/data \
    --path_to_model ./output/train/model.pkl \
    --path_to_output ./output/eval \
    --batch_size 128 \
    --use_cuda false

EXTRA_CLI_FLAGS="--linked-manifests=$MODEL_ID"
echo "Creating evaluation script manifest linked to model..."
create_software_manifest \
    "eval.py" \
    "MNIST Evaluation Script" \
    "MNIST Model Evaluation Implementation" \
    "script" \
    "1.0.0" \
    "Your Organization" \
    "Your Name" \
    "PyTorch evaluation script for MNIST CNN model" \
    eval_script_output.txt
EVAL_SCRIPT_ID=$(extract_id eval_script_output.txt)
echo "Evaluation Script ID: $EVAL_SCRIPT_ID"

EXTRA_CLI_FLAGS=
echo "Creating evaluation configuration manifest..."
create_dataset_manifest \
    "./output/eval/eval_conf.json" \
    "Evaluation Configuration" \
    "MNIST Evaluation Configuration" \
    "Your Organization" \
    "Your Name" \
    eval_config_output.txt
EVAL_CONFIG_ID=$(extract_id eval_config_output.txt)
echo "Evaluation Config ID: $EVAL_CONFIG_ID"

echo "Creating evaluation results manifest linked to model..."
create_evaluation_manifest \
    "./output/eval/eval_results.json" \
    "MNIST Model Evaluation Results" \
    $MODEL_ID \
    $DATASET_ID \
    "Your Organization" \
    --author-name="Your Name" \
    eval_results_output.txt
EVAL_RESULTS_ID=$(extract_id eval_results_output.txt)
echo "Evaluation Results ID: $EVAL_RESULTS_ID"

echo -e "\n=== STEP 4: Export Provenance Graph ==="
export_manifest_json \
    $EVAL_RESULTS_ID \
    mnist_provenance.json

echo -e "\n=== STEP 5: Validate and Show Provenance ==="
validate_manifest $EVAL_RESULTS_ID

echo -e "\nShowing complete manifest with cross-references..."
display_manifest_json $EVAL_RESULTS_ID

echo -e "\n=== Cleanup ==="
rm -f *_output.txt

echo -e "\n=== Complete! ==="
echo "Provenance graph exported to: mnist_provenance.json"
echo "Final Evaluation Results ID: $EVAL_RESULTS_ID"
