# MNIST Training Provenance Collection Example

## Introduction

This example demonstrates how to collect comprehensive provenance data from a complete machine learning workflow using the Atlas CLI tool and Atlas Test Framework. Using the classic MNIST handwritten digit classification task, we track the entire pipeline from dataset download through model training to evaluation, creating a complete audit trail of all artifacts and their relationships.

Provenance tracking in machine learning is crucial for:
- **Reproducibility**: Knowing exactly which data, code, and configurations produced a model
- **Accountability**: Tracking who created what and when
- **Compliance**: Meeting regulatory requirements for model transparency
- **Model Governance**: Understanding the lineage of production models
- **Debugging**: Tracing issues back to their source in the pipeline

This example creates C2PA-compliant manifests for:
- **Datasets**: Raw MNIST data, training configurations, evaluation configurations
- **Software Components**: Training and evaluation scripts with proper versioning
- **Models**: Trained PyTorch CNN model with complete lineage
- **Evaluation Results**: Accuracy metrics linked to model and test data

All components are linked to their direct parents during creation to form a complete provenance graph that can be exported and audited. The Atlas Test Framework automates this entire process, providing reproducibility, error handling, and comprehensive logging.

## Prerequisites

### System Requirements
- Python 3.8 or higher
- Poetry package manager
- Atlas CLI tool compiled and available
- Atlas Test Framework installed
- Database backend running at http://localhost:8080

### Setting up Atlas Test Framework

The example uses the Atlas Test Framework for orchestration and reproducibility:

```bash
# Navigate to the project root
cd /path/to/atlas-test-framework

# Install dependencies with Poetry
poetry install

# Verify installation
poetry run atlas-test --help
```

The framework provides significant advantages over manual script execution:
- Automatic manifest ID tracking between steps
- Built-in error handling and recovery mechanisms
- Comprehensive logging and audit trails
- Reproducible execution scripts generation
- Variable resolution and dependency management

### Installing Python Dependencies

This project uses Poetry for dependency management. Install Poetry if you haven't already:

```bash
# Install Poetry (if not already installed)
curl -sSL https://install.python-poetry.org | python3 -
```

The training scripts require additional dependencies that should be installed in the MNIST directory:

```bash
# Navigate to the MNIST example directory
cd examples/workflows/mnist

# Install ML dependencies (if pyproject.toml exists in this directory)
poetry install

# Or install manually if needed
pip install torch torchvision tqdm matplotlib colorlog
```

### Setting up Atlas CLI

Ensure Atlas CLI is built and available in your PATH:

```bash
# Build Atlas CLI (from the root directory)
cargo build --release

# Add to PATH or use full path
export PATH=$PATH:./target/release

# Verify installation
atlas-cli --version
```

### Setting up the Atlas Test Framework
See the parent [README.md](../../README.md) for instructions on building the test framework.

### Database Backend

Start the database backend for manifest storage:

```bash
# Start the database service
cd storage_service && docker-compose build && docker-compose up -d && cd ..

# Verify database is running
curl http://localhost:8080/health
```

## Project Structure

```
examples/mnist_pipeline/
├── mnist_complete_pipeline.yaml    # Atlas Test Framework configuration
├── README.md                       # This documentation
├── download.py                     # Script to download MNIST dataset
├── train.py                        # Training script for CNN model
├── eval.py                         # Evaluation script
├── model.py                        # CNN model definition (if separate)
├── pyproject.toml                  # Python dependencies (optional)
└── output/                         # Generated during execution
    ├── data/                       # Downloaded MNIST data
    │   └── MNIST/raw/             # IDX format files
    ├── train/                      # Training artifacts
    │   ├── model.pkl              # Trained model weights
    │   └── training_conf.json     # Training configuration
    ├── eval/                       # Evaluation artifacts
    │   ├── eval_results.json      # Evaluation metrics
    │   └── eval_conf.json         # Evaluation configuration
    ├── mnist_provenance.json      # Complete provenance graph
    ├── commands.log                # Detailed execution log
    └── reproduce.sh                # Reproducible execution script
```

## Running the Workflow

You can run the complete workflow using the Atlas Test Framework with the provided YAML configuration.

### Automated Execution (Recommended)

Run the complete MNIST pipeline using the framework:

```bash
# Run the complete MNIST training pipeline
./target/release/atlas-test mnist_complete_pipeline.yaml

# Run with verbose output for debugging
./target/release/atlas-test mnist_complete_pipeline.yaml --verbose

# Run without interactive pauses
./target/release/atlas-test mnist_complete_pipeline.yaml --no-interactive
```

### Preview Mode

To see what the framework will do without executing:

```bash
# Dry run to preview all operations
./target/release/atlas-test mnist_complete_pipeline.yaml --dry-run
```

### Interactive Mode

For step-by-step execution with pauses:

```bash
# Run with interactive pauses between steps
./target/release/atlas-test mnist_complete_pipeline.yaml --interactive
```

## Workflow Steps

The framework orchestrates the following sequence automatically:

### Step 1: Dataset Download and Registration
```yaml
- name: "Download MNIST Dataset"
  action: shell:command
  parameters:
    command: "poetry run python download.py --path_to_output ./output/data"
```

Downloads MNIST data and creates a manifest with multiple ingredients:
- Training images (train-images-idx3-ubyte.gz)
- Training labels (train-labels-idx1-ubyte.gz)
- Test images (t10k-images-idx3-ubyte.gz)
- Test labels (t10k-labels-idx1-ubyte.gz)

### Step 2: Model Training
```yaml
- name: "Train MNIST Model"
  action: shell:command
  parameters:
    command: "poetry run python train.py --path_to_data ./output/data --path_to_output ./output/train --batch_size 128 --lr 0.5 --epochs 1 --use_cuda false"
```

Executes training and creates manifests for:
- Training script with dataset linkage
- Training configuration file
- Trained model with dataset linkage

### Step 3: Model Evaluation
```yaml
- name: "Evaluate MNIST Model"
  action: shell:command
  parameters:
    command: "poetry run python eval.py --path_to_data ./output/data --path_to_model ./output/train/model.pkl --path_to_output ./output/eval --batch_size 128 --use_cuda false"
```

Runs evaluation and creates manifests for:
- Evaluation script with model linkage
- Evaluation configuration
- Evaluation results with model and dataset linkage

### Step 4: Provenance Export and Validation
```yaml
- name: "Export Provenance Graph"
  action: manifest:export
  parameters:
    manifest_id: "${EVAL_RESULTS_ID}"
    output_file: ./output/mnist_provenance.json
```

Exports complete provenance graph and validates all cross-references.

## Understanding the Output

### Generated Files

The framework creates several important output files:

1. **mnist_provenance.json**: Complete provenance graph showing all relationships
2. **commands.log**: Detailed log of every command executed
3. **reproduce.sh**: Executable script to reproduce the exact same run
4. **Training artifacts**: Model files, configurations, and metrics

### Provenance Graph Structure

The generated provenance graph contains direct relationships:

```
Dataset ─┬─→ Training Script
         │
         ├─→ Model ──→ Evaluation Results
         │              ↑
         └─→ Evaluation Script
```

Example manifest structure:
```json
{
  "manifest": {
    "title": "MNIST Model Evaluation Results",
    "ingredients": [
      {
        "title": "Evaluation Results",
        "hash": "sha384:...",
        "mediaType": "application/json"
      }
    ],
    "linked_manifests": [
      "urn:c2pa:model-id",
      "urn:c2pa:dataset-id"
    ]
  }
}
```

### Framework Benefits

The Atlas Test Framework provides:
- **Automatic ID Tracking**: No manual ID extraction needed
- **Error Recovery**: Graceful handling of failures
- **Reproducibility**: Generated scripts for exact reproduction
- **Comprehensive Logging**: Detailed audit trails
- **Variable Resolution**: Automatic substitution of manifest IDs

## Verification and Validation

The framework automatically performs validation:

```yaml
- name: "Validate Cross-References"
  action: manifest:validate
  parameters:
    manifest_id: "${EVAL_RESULTS_ID}"
```

Manual verification is also possible:
```bash
# Validate specific manifest
atlas-cli manifest validate --id=<MANIFEST_ID> --storage-type=database --storage-url=http://localhost:8080

# Show complete manifest with relationships
atlas-cli manifest show --id=<MANIFEST_ID> --storage-type=database --storage-url=http://localhost:8080

# Reproduce the entire workflow
bash ./output/reproduce.sh
```

## Customization

### Modifying Training Parameters

Edit the YAML configuration to change training parameters:

```yaml
- name: "Train MNIST Model"
  action: shell:command
  parameters:
    command: "poetry run python train.py --path_to_data ./output/data --path_to_output ./output/train --batch_size 256 --lr 0.1 --epochs 5 --use_cuda true"
```

### Adding New Steps

Extend the pipeline with additional operations:

```yaml
- name: "Data Preprocessing"
  action: software:create
  parameters:
    paths:
      - preprocess.py
    name: "Data Preprocessing Script"
    linked_manifests:
      - "${DATASET_ID}"
  store_as: PREPROCESS_ID
```

### Environment Configuration

Modify environment settings in the YAML:

```yaml
environment:
  storage_type: database
  storage_url: http://localhost:8080
  author_org: "Your Organization"
  author_name: "Your Name"
  output_dir: ./custom_output
```

## Troubleshooting

### Common Issues

1. **Framework Installation**:
   ```bash
   # Reinstall if needed
   poetry install --no-cache
   
   # Check installation
   atlas-test --help
   ```

2. **Python Dependencies**:
   ```bash
   # Install ML dependencies
   pip install torch torchvision tqdm matplotlib
   
   # Or use Poetry if pyproject.toml exists
   poetry install
   ```

3. **Atlas CLI Issues**:
   ```bash
   # Verify Atlas CLI
   atlas-cli --version
   
   # Rebuild if needed
   cargo build --release
   ```

4. **Database Connection**:
   ```bash
   # Check database status
   curl http://localhost:8080/health
   
   # Restart database if needed
   cd storage_service && docker-compose restart
   ```
 

### Debug Mode

For detailed troubleshooting:

```bash
# Run with maximum verbosity
./target/release/atlas-test mnist_complete_pipeline.yaml --verbose

# Check generated logs
cat ./output/commands.log

# Examine specific manifest
atlas-cli manifest show --id=<ID> --storage-type=database --storage-url=http://localhost:8080
```

## Conclusion

This example demonstrates how the Atlas Test Framework can automate comprehensive provenance collection for machine learning workflows. By tracking every artifact and its relationships through a declarative YAML configuration, you create an auditable trail that enhances reproducibility, accountability, and compliance in ML development.

The framework approach provides significant advantages over manual scripting, including automatic error handling, logging, and guaranteed command reproducibility through generated scripts.