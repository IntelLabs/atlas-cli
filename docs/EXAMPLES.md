# Atlas CLI Examples

This document provides examples and workflow patterns for using the Atlas CLI tool.

## End-to-End Examples

We provide a number of end-to-end Atlas workflow examples in the
[/examples](../examples) directory,

## Basic Usage Examples

### Creating a Single Model Manifest

```bash
atlas-cli model create \
    --paths=model.onnx \
    --ingredient-names="Main Model" \
    --name="Model Name" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --format=json \
    --print
```

### Creating a Dataset Manifest

```bash
atlas-cli dataset create \
    --paths=data/ \
    --ingredient-names="Training Data" \
    --name="Dataset Name" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --format=json \
    --print
```

### Creating a Signed Manifest

To sign manifests, you'll need a private key.

**Important**: Cryptographic key generation and management are the responsibility of the end user. The security of your C2PA manifests depends entirely on the strength and proper management of your signing/verification keys.

```bash
# First, generate keys if you don't have them
make generate-keys

# Create a signed model manifest
atlas-cli model create \
    --paths=model.onnx \
    --ingredient-names="Main Model" \
    --name="Signed Model" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --key=private.pem \
    --format=json \
    --print
```

## Multiple Components Examples

### Multi-Component Model

```bash
atlas-cli model create \
    --paths=feature_extractor.onnx,classifier.onnx \
    --ingredient-names="Feature Extractor,Classifier" \
    --name="Complete Model" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --format=json \
    --print
```

### Multi-Component Dataset

```bash
atlas-cli dataset create \
    --paths=train/,test/,validation/ \
    --ingredient-names="Training Set,Test Set,Validation Set" \
    --name="Complete Dataset" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --format=json \
    --print
```

## Storage Examples

### Using Database Storage

```bash
# Store a model manifest in the database
atlas-cli model create \
    --paths=model.onnx \
    --ingredient-names="Main Model" \
    --name="Database Model" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --storage-type=database \
    --storage-url=http://localhost:8080 \
    --format=json

# List manifests in database storage
atlas-cli model list \
    --storage-type=database \
    --storage-url=http://localhost:8080
```

### Using Filesystem Storage

```bash
# Store a model manifest in the filesystem
atlas-cli model create \
    --paths=model.onnx \
    --ingredient-names="Main Model" \
    --name="Filesystem Model" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --storage-type=filesystem \
    --storage-url=./storage \
    --format=json

# List manifests in filesystem storage
atlas-cli model list \
    --storage-type=filesystem \
    --storage-url=./storage
```

## Workflow Examples

### Basic Model-Dataset Linking

```bash
# Step 1: Create a dataset manifest
atlas-cli dataset create \
    --paths=training_data.csv \
    --ingredient-names="Training Dataset" \
    --name="MNIST Training Dataset" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --storage-type=database \
    --storage-url=http://localhost:8080
# Output: Dataset ID (save for later use)

# Step 2: Create a model manifest
atlas-cli model create \
    --paths=model.onnx \
    --ingredient-names="Model" \
    --name="MNIST Classifier" \
    --author-org="Your Organization" \
    --author-name="Your Name" \
    --storage-type=database \
    --storage-url=http://localhost:8080
# Output: Model ID (save for later use)

# Step 3: Link the dataset to the model
atlas-cli model link-dataset \
    --model-id=<MODEL_ID> \
    --dataset-id=<DATASET_ID> \
    --storage-type=database \
    --storage-url=http://localhost:8080

# Step 4: Verify the linking
atlas-cli model verify \
    --id=<MODEL_ID> \
    --storage-type=database \
    --storage-url=http://localhost:8080
```

### C2PA-Compliant Linking

```bash
# Step 1 & 2: Create manifests as above

# Step 3: Link manifests using C2PA cross-references
atlas-cli manifest link \
    --source=<MODEL_URN> \
    --target=<DATASET_URN> \
    --storage-type=database \
    --storage-url=http://localhost:8080

# Step 4: Show the linked manifest with cross-references
atlas-cli manifest show \
    --id=<MODEL_URN> \
    --storage-type=database \
    --storage-url=http://localhost:8080
```

### Advanced Linking and Validation

```bash
# Validate all cross-references in a manifest
atlas-cli manifest validate \
    --id=<MANIFEST_URN> \
    --storage-type=database \
    --storage-url=http://localhost:8080

# Verify a specific link between two manifests
atlas-cli manifest verify-link \
    --source=<SOURCE_URN> \
    --target=<TARGET_URN> \
    --storage-type=database \
    --storage-url=http://localhost:8080

# Create bi-directional links
atlas-cli manifest link \
    --source=<MODEL_URN> \
    --target=<DATASET_URN> \
    --storage-type=database \
    --storage-url=http://localhost:8080

atlas-cli manifest link \
    --source=<DATASET_URN> \
    --target=<MODEL_URN> \
    --storage-type=database \
    --storage-url=http://localhost:8080
```

## Complete ML Workflow Example

This example demonstrates tracking the entire ML workflow from raw data to final model:

```bash
# 1. Create manifest for raw training data
atlas-cli dataset create \
    --paths=raw_data/ \
    --ingredient-names="Raw Data" \
    --name="Raw Training Data" \
    --storage-type=database \
    --storage-url=http://localhost:8080
# Save ID as RAW_DATA_ID

# 2. Create manifest for preprocessed data
atlas-cli dataset create \
    --paths=preprocessed_data/ \
    --ingredient-names="Preprocessed Data" \
    --name="Preprocessed Training Data" \
    --storage-type=database \
    --storage-url=http://localhost:8080
# Save ID as PREPROCESSED_DATA_ID

# 3. Link preprocessed data to raw data
atlas-cli manifest link \
    --source=$PREPROCESSED_DATA_ID \
    --target=$RAW_DATA_ID \
    --storage-type=database \
    --storage-url=http://localhost:8080

# 4. Create manifest for the model
atlas-cli model create \
    --paths=model.onnx \
    --ingredient-names="ONNX Model" \
    --name="Trained Model" \
    --storage-type=database \
    --storage-url=http://localhost:8080
# Save ID as MODEL_ID

# 5. Link model to preprocessed data
atlas-cli manifest link \
    --source=$MODEL_ID \
    --target=$PREPROCESSED_DATA_ID \
    --storage-type=database \
    --storage-url=http://localhost:8080
```

## Software Component Examples

### Creating a Software Component Manifest

```bash
atlas-cli software create \
  --paths=train.py \
  --ingredient-names="Training Script" \
  --name="Image Classification Trainer" \
  --software-type="script" \
  --version="1.2.0" \
  --author-org="ML Research Team" \
  --author-name="Your Name" \
  --description="Python script for training image classification models"
# Save ID as SOFTWARE_ID
```

### Linking Software Components

```bash
# Link a preprocessing script to a dataset
atlas-cli software link-dataset \
  --software-id=$SOFTWARE_ID \
  --dataset-id=$DATASET_ID

# Link a training script to a model
atlas-cli software link-model \
  --software-id=$SOFTWARE_ID \
  --model-id=$MODEL_ID
```

## Output Format Examples

### JSON Output Format

```bash
atlas-cli model create \
    --paths=model.onnx \
    --ingredient-names="Model" \
    --name="JSON Example" \
    --format=json \
    --print
```

### CBOR Output Format

```bash
atlas-cli model create \
    --paths=model.onnx \
    --ingredient-names="Model" \
    --name="CBOR Example" \
    --format=cbor \
    --print
```

## Makefile Examples

The repository includes pre-configured examples that can be run using make:

```bash
# Setup examples directory structure
make setup-examples

# Run example for a single model
make example-model-single

# Run example for multiple model components
make example-model-multi

# Run example for dataset
make example-dataset-single

# Run example for a complete workflow
make example-full-workflow

# Run example for filesystem storage
make example-filesystem-storage
```
