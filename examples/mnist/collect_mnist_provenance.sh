#!/bin/bash
# MNIST Provenance Collection Script
# This script runs the complete MNIST workflow and collects provenance data

# Configuration
STORAGE_URL="http://localhost:8080"

# Helper function to extract ID from output
extract_id() {
    grep -o "ID: [^ ]*" "$1" | cut -d' ' -f2
}

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
atlas-cli dataset create \
    --paths="$DATAPATHS" \
    --ingredient-names="MNIST Dataset" \
    --name="MNIST Training and Test Data" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --storage-type=database \
    --storage-url=$STORAGE_URL \
    --key=private.pem \
    > dataset_output.txt
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
atlas-cli software create \
    --paths=train.py \
    --ingredient-names="MNIST Training Script" \
    --name="MNIST CNN Training Implementation" \
    --software-type="script" \
    --version="1.0.0" \
    --linked-manifests=$DATASET_ID \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --description="PyTorch training script for MNIST CNN model" \
    --key=private.pem \
    --storage-type=database \
    --storage-url=$STORAGE_URL \
    > training_script_output.txt
TRAINING_SCRIPT_ID=$(extract_id training_script_output.txt)
echo "Training Script ID: $TRAINING_SCRIPT_ID"

echo "Creating training configuration manifest..."
atlas-cli dataset create \
    --paths=./output/train/training_conf.json \
    --ingredient-names="Training Configuration" \
    --name="MNIST Training Configuration" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --key=private.pem \
    --storage-type=database \
    --storage-url=$STORAGE_URL \
    > training_config_output.txt
TRAINING_CONFIG_ID=$(extract_id training_config_output.txt)
echo "Training Config ID: $TRAINING_CONFIG_ID"

echo "Creating model manifest linked to dataset..."
atlas-cli model create \
    --paths=./output/train/model.pkl \
    --ingredient-names="MNIST CNN Model" \
    --name="Trained MNIST Classifier" \
    --linked-manifests=$DATASET_ID \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --key=private.pem \
    --storage-type=database \
    --storage-url=$STORAGE_URL \
    > model_output.txt
MODEL_ID=$(extract_id model_output.txt)
echo "Model ID: $MODEL_ID"

echo -e "\n=== STEP 3: Evaluate the Model ==="
poetry run python eval.py \
    --path_to_data ./output/data \
    --path_to_model ./output/train/model.pkl \
    --path_to_output ./output/eval \
    --batch_size 128 \
    --use_cuda false

echo "Creating evaluation script manifest linked to model..."
atlas-cli software create \
    --paths=eval.py \
    --ingredient-names="MNIST Evaluation Script" \
    --name="MNIST Model Evaluation Implementation" \
    --software-type="script" \
    --version="1.0.0" \
    --linked-manifests=$MODEL_ID \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --description="PyTorch evaluation script for MNIST CNN model" \
    --key=private.pem \
    --storage-type=database \
    --storage-url=$STORAGE_URL \
    > eval_script_output.txt
EVAL_SCRIPT_ID=$(extract_id eval_script_output.txt)
echo "Evaluation Script ID: $EVAL_SCRIPT_ID"

echo "Creating evaluation configuration manifest..."
atlas-cli dataset create \
    --paths=./output/eval/eval_conf.json \
    --ingredient-names="Evaluation Configuration" \
    --name="MNIST Evaluation Configuration" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --key=private.pem \
    --storage-type=database \
    --storage-url=$STORAGE_URL \
    > eval_config_output.txt
EVAL_CONFIG_ID=$(extract_id eval_config_output.txt)
echo "Evaluation Config ID: $EVAL_CONFIG_ID"

echo "Creating evaluation results manifest linked to model..."
atlas-cli evaluation create \
    --path=./output/eval/eval_results.json \
    --name="MNIST Model Evaluation Results" \
    --model-id=$MODEL_ID \
    --dataset-id=$DATASET_ID \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --key=private.pem \
    --storage-type=database \
    --storage-url=$STORAGE_URL \
    > eval_results_output.txt
EVAL_RESULTS_ID=$(extract_id eval_results_output.txt)
echo "Evaluation Results ID: $EVAL_RESULTS_ID"

echo -e "\n=== STEP 4: Export Provenance Graph ==="
atlas-cli manifest export \
    --id=$EVAL_RESULTS_ID \
    --storage-type=database \
    --storage-url=$STORAGE_URL \
    --format=json \
    --max-depth=10 \
    --output=mnist_provenance.json

echo -e "\n=== STEP 5: Validate and Show Provenance ==="
atlas-cli manifest validate \
    --id=$EVAL_RESULTS_ID \
    --storage-type=database \
    --storage-url=$STORAGE_URL

echo -e "\nShowing complete manifest with cross-references..."
atlas-cli manifest show \
    --id=$EVAL_RESULTS_ID \
    --storage-type=database \
    --storage-url=$STORAGE_URL

echo -e "\n=== Cleanup ==="
rm -f *_output.txt

echo -e "\n=== Complete! ==="
echo "Provenance graph exported to: mnist_provenance.json"
echo "Final Evaluation Results ID: $EVAL_RESULTS_ID"