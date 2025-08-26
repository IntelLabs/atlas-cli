# Atlas Test Framework

A testing framework for Atlas CLI that enables automated testing of C2PA ML provenance workflows, integrity verification, and manifest management.

The Atlas Test Framework provides:
- Automated execution of Atlas CLI commands
- End-to-end testing of ML provenance workflows
- Integrity verification testing
- Command recording and reproduction
- Support for multiple storage backends (local-fs, database, rekor)
- Modular test examples with shared resources

## 📋 Prerequisites

- Python 3.8 or higher
- Poetry (for dependency management)
- Atlas CLI installed and accessible in PATH
- OpenSSL (for key generation)

## 🚀 Quick Start

### 1. Install Poetry

```bash
curl -sSL https://install.python-poetry.org | python3 -
```

### 2. Clone and Setup

```bash
cd examples
poetry install
```

### 3. Verify Atlas CLI

```bash
atlas-cli --version
```

### 4. Run Your First Test

```bash
# Run the simple demo
poetry run atlas-test examples/simple_demo/config.yaml

# Or activate the virtual environment
poetry shell
atlas-test examples/simple_demo/config.yaml
```

## 📁 Project Structure

```
atlas-cli/examples/
├── pyproject.toml              # Project dependencies
├── README.md                   # This file
├── .env.example               # Environment variables template
├── .gitignore                 # Git ignore rules
│
├── atlas_test/                # Framework code
│   ├── __init__.py
│   ├── framework.py          # Main framework implementation
│   ├── runner.py            # CLI runner
│   ├── recorder.py          # Command recording
│   └── utils.py             # Utility functions
│
├── shared/                   # Shared resources
│   ├── data/                # Common datasets
│   ├── scripts/             # Reusable scripts
│   ├── models/              # Pre-trained models
│   └── keys/                # Shared signing keys
│
├── examples/                 # Test examples
│   ├── simple_demo/         # Basic functionality test
│   ├── integrity_test/      # Tampering detection
│   ├── mnist_pipeline/      # Complete ML pipeline
│   └── oss025demo/         # OSS 2025 demo
└── tests/                    # Framework tests
    ├── test_framework.py
    └── test_utils.py
```

## 🔧 Configuration

### Basic Configuration Structure

```yaml
name: "Test Name"
description: "Test description"

environment:
  storage_type: local-fs        # Storage backend: local-fs, database, rekor
  storage_url: ./test_storage   # Storage location
  signing_key: ./keys/test.pem  # Private key for signing
  verifying_key: ./keys/test_pub.pem  # Public key
  generate_keys: true           # Auto-generate keys if missing
  output_dir: ./test_output     # Output directory
  hash_alg: sha384             # Hash algorithm

steps:
  - name: "Step Name"
    action: dataset:create      # Action to perform
    parameters:                 # Action-specific parameters
      paths:
        - ./data/file.csv
      name: "Dataset Name"
    store_as: DATASET_ID       # Store result for later use
```

### Environment Variables

Create a `.env` file in your project root:

```bash
# Storage Configuration
STORAGE_TYPE=local-fs
STORAGE_URL=./storage

# Author Information
AUTHOR_ORG=My Organization
AUTHOR_NAME=My Name
AUTHOR_EMAIL=my.email@example.com

# Signing Keys
SIGNING_KEY=./keys/private.pem
VERIFYING_KEY=./keys/public.pem
```

## 📚 Available Actions

### Dataset Operations

- `dataset:create` - Create a dataset manifest
- `dataset:verify` - Verify dataset integrity
- `dataset:list` - List all datasets

### Model Operations

- `model:create` - Create a model manifest
- `model:verify` - Verify model integrity
- `model:list` - List all models

### Software Operations

- `software:create` - Create a software manifest
- `software:verify` - Verify software integrity

### Evaluation Operations

- `evaluation:create` - Create an evaluation manifest
- `evaluation:verify` - Verify evaluation integrity

### Manifest Operations

- `manifest:validate` - Validate cross-references between manifests
- `manifest:link` - Link two manifests
- `manifest:show` - Display manifest details
- `manifest:export` - Export provenance graph

### Utility Operations

- `shell:command` - Execute custom shell command
- `file:tamper` - Tamper with a file (for testing)

## 💡 Examples

### Simple Dataset Creation

```yaml
steps:
  - name: "Create Dataset"
    action: dataset:create
    parameters:
      paths:
        - ./data/training.csv
        - ./data/validation.csv
      name: "Training Dataset"
      description: "MNIST training data"
      author_org: "AI Lab"
      author_name: "John Doe"
    store_as: DATASET_ID
```

### Model with Linked Dataset

```yaml
steps:
  - name: "Create Model"
    action: model:create
    parameters:
      paths:
        - ./models/model.pkl
      name: "Classification Model"
      linked_manifests:
        - "${DATASET_ID}"  # Reference previous step
    store_as: MODEL_ID
```

### Integrity Verification Test

```yaml
steps:
  - name: "Create Dataset"
    action: dataset:create
    parameters:
      paths: ["./data/original.csv"]
      name: "Original Data"
    store_as: DATASET_ID

  - name: "Verify Original"
    action: dataset:verify
    parameters:
      manifest_id: "${DATASET_ID}"
    expect: success

  - name: "Tamper File"
    action: file:tamper
    parameters:
      file: ./data/original.csv

  - name: "Verify Tampered"
    action: dataset:verify
    parameters:
      manifest_id: "${DATASET_ID}"
    expect: failure  # Should fail after tampering
```

## 🔍 Path Resolution

The framework supports special path prefixes:

- `@shared/` - Reference shared resources
- `@example/` - Reference current example directory
- `./` - Relative to config file
- `/` - Absolute path

Example:
```yaml
parameters:
  paths:
    - ./data/local.csv          # Example-specific
    - "@shared/data/common.csv" # Shared resource
```

## 🏃 Running Tests

### Basic Usage

```bash
# Run a test configuration
atlas-test <config-file>

# Run with options
atlas-test examples/simple_demo/config.yaml \
    --dry-run \              # Preview commands without execution
    --verbose \              # Show detailed output
    --interactive \          # Pause between steps
    --continue-on-error \    # Don't stop on errors
    --output-dir ./output    # Custom output directory
```

### Command Recording

All executed commands are recorded in:
- `commands.log` - Detailed execution log
- `reproduce.sh` - Executable script to reproduce the test

### Dry Run Mode

Test your configuration without executing commands:

```bash
atlas-test examples/simple_demo/config.yaml --dry-run
```

## 📝 Creating New Examples

### 1. Create Directory Structure

```bash
mkdir -p examples/my_example/{data,scripts,models,output,keys}
```

### 2. Create Configuration

```yaml
# examples/my_example/config.yaml
name: "My Example"
description: "Description of my example"

environment:
  storage_type: local-fs
  storage_url: ./output/storage
  generate_keys: true
  output_dir: ./output

steps:
  - name: "First Step"
    action: dataset:create
    parameters:
      paths: ["./data/mydata.csv"]
      name: "My Dataset"
```

### 3. Add Test Data

```bash
# Add your test files
cp your_data.csv examples/my_example/data/
cp your_model.pkl examples/my_example/models/
```

### 4. Create README

```markdown
# My Example

## Purpose
Describe what this example demonstrates

## Requirements
- List specific requirements

## Usage
\`\`\`bash
poetry run atlas-test examples/my_example/config.yaml
\`\`\`

## Expected Results
Describe expected outcomes
```

## 🧪 Testing

### Run Framework Tests

```bash
# Run all tests
poetry run pytest

# Run with coverage
poetry run pytest --cov=atlas_test

# Run specific test
poetry run pytest tests/test_framework.py::test_command_builder
```

### Test All Examples

```bash
# Test all examples in dry-run mode
for config in examples/*/config.yaml; do
    echo "Testing: $config"
    poetry run atlas-test "$config" --dry-run
done
```

## 🐛 Troubleshooting

### Atlas CLI Not Found

```bash
# Check if atlas-cli is in PATH
which atlas-cli

# Add to PATH if needed
export PATH=$PATH:/path/to/atlas-cli
```

### Storage Access Issues

For local-fs storage:
```bash
# Ensure storage directory is writable
chmod 755 ./test_storage
```

For database storage:
```bash
# Verify database is running
curl http://localhost:8080/health
```

### Key Generation Fails

```bash
# Generate keys manually
openssl genpkey -algorithm RSA -out private.pem -pkeyopt rsa_keygen_bits:4096
openssl rsa -pubout -in private.pem -out public.pem
```

## 📊 Output Files

After running tests, check the output directory for:

- `commands.log` - Complete command execution log
- `reproduce.sh` - Script to reproduce all commands
- `*.json` - Exported manifests
- `provenance_*.json` - Provenance graphs

## 🚦 Status Codes

- ✅ Success - Command executed successfully
- ❌ Failure - Command failed
- ⚠️ Warning - Non-critical issue
- 📋 Info - Informational message
- 🔑 Security - Key/signing related
- 📦 Manifest - Manifest created/verified
- 🏁 Complete - Test finished

## 📖 Additional Resources

- [Atlas CLI Documentation](https://github.com/IntelLabs/atlas-cli)
- [C2PA Specification](https://c2pa.org)
- [Poetry Documentation](https://python-poetry.org/docs/)