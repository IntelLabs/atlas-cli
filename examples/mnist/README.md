# MNIST Training Provenance Collection Example

## Introduction

This example demonstrates how to collect comprehensive provenance data from a complete machine learning workflow using the Atlas CLI tool. Using the classic MNIST handwritten digit classification task, we track the entire pipeline from dataset download through model training to evaluation, creating a complete audit trail of all artifacts and their relationships.

Provenance tracking in machine learning is crucial for:
- Reproducibility: Knowing exactly which data, code, and configurations produced a model
- Accountability: Tracking who created what and when
- Compliance: Meeting regulatory requirements for model transparency
- Model Governance: Understanding the lineage of production models
- Debugging: Tracing issues back to their source in the pipeline

This example creates C2PA-compliant manifests for:
- Datasets (raw MNIST data, configurations)
- Software components (training and evaluation scripts)
- Models (trained PyTorch model)
- Evaluation results (accuracy metrics)

All components are linked to their direct parents during creation to form a complete provenance graph that can be exported and audited.

## Prerequisites

### System Requirements
- Python 3.10 or higher
- Poetry package manager
- Atlas CLI tool compiled and available
- Database backend running at http://localhost:8080

### Installing Python Dependencies

This project uses Poetry for dependency management. Install Poetry if you haven't already:

```bash
# Install Poetry (if not already installed)
curl -sSL https://install.python-poetry.org | python3 -
```

Install the project dependencies using the provided pyproject.toml:

```bash
# Navigate to the project directory
cd examples/mnist

# Install dependencies with Poetry
poetry install

# Activate the virtual environment
poetry shell
```

The pyproject.toml includes all necessary dependencies:
- PyTorch with CUDA support (falls back to CPU if CUDA unavailable)
- TorchVision for MNIST dataset
- Supporting libraries (colorlog, tqdm, matplotlib, etc.)

### Setting up Atlas CLI

Ensure Atlas CLI is built and available in your PATH:

```bash
# Build Atlas CLI (from the root directory)
cargo build --release
make generate-keys
cp private.pem public.pem examples/mnist/
# Add to PATH or use full path
export PATH=$PATH:./target/release
```

### Database Backend

Start the database backend (if not already running):

```bash
# Start the database service
cd storage_service && docker-compose build && docker-compose up -d && cd ..
```

## Project Structure

```
examples/mnist/
├── download.py          # Script to download MNIST dataset
├── train.py            # Training script for CNN model
├── eval.py             # Evaluation script
├── model.py            # CNN model definition
├── pyproject.toml      # Python dependencies
└── output/             # Output directory (created by scripts)
    ├── data/           # Downloaded MNIST data
    ├── train/          # Training artifacts
    │   ├── model.pkl   # Trained model weights
    │   └── training_conf.json
    └── eval/           # Evaluation artifacts
        ├── eval_results.json
        └── eval_conf.json
```

## Step-by-Step Workflow

### Step 1: Download MNIST Dataset

First, download the MNIST dataset and create a manifest for it:

```bash
# Download MNIST data
poetry run python download.py --path_to_output ./output/data

# Create dataset manifest with multiple ingredients
FILES=(
  ./output/data/MNIST/raw/t10k-images-idx3-ubyte.gz
  ./output/data/MNIST/raw/t10k-labels-idx1-ubyte.gz
  ./output/data/MNIST/raw/train-images-idx3-ubyte.gz
  ./output/data/MNIST/raw/train-labels-idx1-ubyte.gz
)

# Verify files exist
for f in "${FILES[@]}"; do
  if [ ! -e "$f" ]; then
    echo "Warning: $f does not exist"
  fi
done

# Create comma-separated list of file paths
DATAPATHS=$(printf "%s\n" "${FILES[@]}" | xargs realpath 2>/dev/null | paste -sd, -)

# Create dataset manifest
atlas-cli dataset create \
    --paths="$DATAPATHS" \
    --ingredient-names="MNIST Test Images,MNIST Test Labels,MNIST Training Images,MNIST Training Labels" \
    --name="MNIST Training and Test Data" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --key=private.pem \
    --storage-type=database \
    --storage-url=http://localhost:8080
```

Save the output ID as DATASET_ID for later use.

### Step 2: Train the Model

Train the CNN model on MNIST data:

```bash
# Train the model
poetry run python train.py \
    --path_to_data ./output/data \
    --path_to_output ./output/train \
    --batch_size 128 \
    --lr 0.5 \
    --epochs 1 \
    --use_cuda false
```

Create manifests for training artifacts:

```bash
# Create manifest for training script linked to dataset
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
    --storage-url=http://localhost:8080
# Save output ID as TRAINING_SCRIPT_ID

# Create manifest for training configuration
atlas-cli dataset create \
    --paths=./output/train/training_conf.json \
    --ingredient-names="Training Configuration" \
    --name="MNIST Training Configuration" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --key=private.pem \
    --storage-type=database \
    --storage-url=http://localhost:8080
# Save output ID as TRAINING_CONFIG_ID

# Create manifest for trained model linked to dataset
atlas-cli model create \
    --paths=./output/train/model.pkl \
    --ingredient-names="MNIST CNN Model" \
    --name="Trained MNIST Classifier" \
    --linked-manifests=$DATASET_ID \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --key=private.pem \
    --storage-type=database \
    --storage-url=http://localhost:8080
# Save output ID as MODEL_ID
```

### Step 3: Evaluate the Model

Evaluate the trained model:

```bash
# Run evaluation
poetry run python eval.py \
    --path_to_data ./output/data \
    --path_to_model ./output/train/model.pkl \
    --path_to_output ./output/eval \
    --batch_size 128 \
    --use_cuda false
```

Create manifests for evaluation artifacts:

```bash
# Create manifest for evaluation script linked to model
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
    --storage-url=http://localhost:8080
# Save output ID as EVAL_SCRIPT_ID

# Create manifest for evaluation configuration
atlas-cli dataset create \
    --paths=./output/eval/eval_conf.json \
    --ingredient-names="Evaluation Configuration" \
    --name="MNIST Evaluation Configuration" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --key=private.pem \
    --storage-type=database \
    --storage-url=http://localhost:8080
# Save output ID as EVAL_CONFIG_ID

# Create manifest for evaluation results linked to model
atlas-cli evaluation create \
    --path=./output/eval/eval_results.json \
    --name="MNIST Model Evaluation Results" \
    --model-id=$MODEL_ID \
    --dataset-id=$DATASET_ID \
    --linked-manifests=$MODEL_ID \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --key=private.pem \
    --storage-type=database \
    --storage-url=http://localhost:8080
# Save output ID as EVAL_RESULTS_ID
```

### Step 4: Export Provenance Graph

Export the complete provenance graph:

```bash
atlas-cli manifest export \
    --id=$EVAL_RESULTS_ID \
    --storage-type=database \
    --storage-url=http://localhost:8080 \
    --format=json \
    --max-depth=10 \
    --output=mnist_provenance.json
```

## Automated Script

For convenience, we provide an automated script that runs the entire workflow as collect_mnist_provenance.sh file.

```

Run the automated script:

```bash
chmod +x collect_mnist_provenance.sh
./collect_mnist_provenance.sh
```

## Understanding the Output

The script generates a provenance graph (mnist_provenance.json) that contains:

1. **Direct relationships only**:
   - Model → Dataset (model was trained on this dataset)
   - Training Script → Dataset (script processes this dataset)
   - Evaluation Script → Model (script evaluates this model)
   - Evaluation Results → Model (results from evaluating this model)

2. **Provenance chain**: When exported, the graph will show the complete chain:
   ```
   Evaluation Results → Model → Dataset
                     ↘ Evaluation Script
   ```

3. **Cross-References**: C2PA-compliant links showing only direct parent relationships

Example provenance graph structure:
```json
{
  "manifest": {
    "title": "MNIST Model Evaluation Results",
    "ingredients": [...],
    "xrefs": [
      {
        "url": "urn:c2pa:model-id",
        "relationship": "parentOf"
      }
    ]
  }
}
```

## Verification and Validation

Verify the provenance data:

```bash
# Validate all cross-references
atlas-cli manifest validate \
    --id=$EVAL_RESULTS_ID \
    --storage-type=database \
    --storage-url=http://localhost:8080

# Show complete manifest with relationships
atlas-cli manifest show \
    --id=$EVAL_RESULTS_ID \
    --storage-type=database \
    --storage-url=http://localhost:8080
```

## Troubleshooting

### Common Issues

1. Python Dependencies:
   ```bash
   # If Poetry fails, try updating pip
   poetry run pip install --upgrade pip
   
   # Force reinstall dependencies
   poetry install --no-cache
   ```

2. CUDA Errors:
   ```bash
   # Use CPU-only version if CUDA issues occur
   poetry remove torch torchvision
   poetry add torch torchvision --source pypi
   ```

3. Database Connection:
   ```bash
   # Check if database is running
   curl http://localhost:8080/health
   
   # Use alternative port if needed
   export STORAGE_URL=http://localhost:8081
   ```

4. Permissions:
   ```bash
   # Ensure output directory is writable
   mkdir -p ./output
   chmod -R 755 ./output
   ```

## Next Steps

After completing this example, you can:

1. Extend the Pipeline: Add data preprocessing steps, hyperparameter tuning, or model optimization
2. Track Experiments: Create manifests for different training runs with varying parameters
3. Build CI/CD Integration: Automatically collect provenance in your ML pipeline
4. Create Visualizations: Use the provenance graph to create visual representations of your ML workflow
5. Implement Governance: Use provenance data for model approval and deployment decisions

## Conclusion

This example demonstrates how Atlas CLI can capture complete provenance for machine learning workflows. By tracking every artifact and its direct parent relationships, you create an auditable trail that enhances reproducibility, accountability, and compliance in ML development.