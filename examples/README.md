# Atlas Test Framework

A testing framework for Atlas CLI that enables automated testing of AI provenance workflows, integrity verification, and manifest management.

The Atlas Test Framework provides:
- Automated execution of Atlas CLI commands
- End-to-end testing of ML provenance workflows
- Integrity verification testing
- Command recording and reproduction
- Support for multiple storage backends (local-fs, database, rekor)
- Modular test examples with shared resources

## Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)
- Atlas CLI installed and accessible in PATH
- OpenSSL (for key generation)

## Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/IntelLabs/atlas-cli
cd atlas-cli/examples
```

### 2. Build the Framework

```bash
# Build release version
cargo build --release

# The binary will be at ./target/release/atlas-test
```

### 3. Verify Atlas CLI

```bash
atlas-cli --version
```

### 4. Run Your First Test

```bash
# From atlas-cli/examples directory
./target/release/atlas-test workflows/simple_demo/config.yaml

# Run with verbose output
./target/release/atlas-test workflows/simple_demo/config.yaml --verbose

# Run in dry-run mode
./target/release/atlas-test workflows/simple_demo/config.yaml --dry-run
```

## Project Structure

```
atlas-cli/examples/
├── Cargo.toml                 # Rust dependencies and project config
├── README.md                  # This file
├── .gitignore                # Git ignore rules
│
├── src/                      # Framework source code
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── framework.rs         # Main framework implementation
│   ├── command.rs           # Command builder
│   ├── recorder.rs          # Command recording
│   ├── config.rs            # Configuration parsing
│   ├── error.rs             # Error types
│   ├── utils.rs             # Utility functions
│   └── subcommands/             # Action handlers
│       ├── mod.rs
│       ├── dataset.rs       # Dataset operations
│       ├── model.rs         # Model operations
│       ├── software.rs      # Software operations
│       ├── evaluation.rs    # Evaluation operations
│       ├── manifest.rs      # Manifest operations
│       └── utility.rs       # Utility operations
│
├── workflows/               # Test workflow configurations
│   ├── simple_demo/         # Basic functionality test
│   │   ├── config.yaml
│   │   ├── data/
│   │   └── keys/
│   │
│   ├── mnist/               # Complete ML pipeline example
│   │   ├── mnist_complete_pipeline.yaml
│   │   ├── pyproject.toml   # Python dependencies for training
│   │   ├── train.py
│   │   ├── eval.py
│   │   └── download.py
│   │
│   └── oss25_demo/          # OSS 2025 demo
│       └── oss25_demo_pipeline.yaml
│
└── shared/                   # Shared resources
    ├── data/                # Common datasets
    ├── scripts/             # Reusable scripts
    ├── models/              # Pre-trained models
    └── keys/                # Shared signing keys
```
**Note:** The term "workflow" in this framework refers to a test scenario configuration that defines a sequence of Atlas CLI commands for testing ML provenance pipelines. This is not related to GitHub Actions or other CI/CD workflow systems.

## Configuration

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

## Available Actions

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
- `manifest:list` - List all manifests

### Utility Operations

- `shell:command` - Execute custom shell command
- `file:tamper` - Tamper with a file (for testing)
- `file:copy` - Copy a file
- `file:delete` - Delete a file
- `file:create` - Create a file with content

## Examples

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

## Running Tests

### Basic Usage

```bash
# From atlas-cli/examples directory
./target/release/atlas-test <config-file>

# Run examples
./target/release/atlas-test workflows/simple_demo/config.yaml
./target/release/atlas-test workflows/mnist/mnist_complete_pipeline.yaml
./target/release/atlas-test workflows/oss25_demo/oss25_demo_pipeline.yaml

# Run with options
./target/release/atlas-test workflows/simple_demo/config.yaml \
    --dry-run \              # Preview commands without execution
    --verbose \              # Show detailed output
    --interactive \          # Pause between steps
```

### From Within Workflow Directory

```bash
# Navigate to specific workflow
cd workflows/oss25_demo

# Run from within the directory
../../target/release/atlas-test oss25_demo_pipeline.yaml

# Or for MNIST
cd workflows/mnist
../../target/release/atlas-test mnist_complete_pipeline.yaml
```

### Command Line Options

```bash
# Show help
./target/release/atlas-test --help

# Version information
./target/release/atlas-test --version

# Dry run (preview without execution)
./target/release/atlas-test config.yaml --dry-run

# Interactive mode (pause between steps)
./target/release/atlas-test config.yaml --interactive

# Verbose logging
./target/release/atlas-test config.yaml --verbose
```

### Command Recording

All executed commands are recorded in:
- `commands.log` - Detailed execution log with timestamps
- `reproduce.sh` - Executable bash script to reproduce the test

## Building and Development

### Build Commands

```bash
# From atlas-cli/examples directory

# Debug build (faster compilation, slower execution)
cargo build

# Release build (optimized)
cargo build --release

# Run directly with cargo
cargo run -- workflows/simple_demo/config.yaml

# Run with options
cargo run --release -- workflows/simple_demo/config.yaml --verbose

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

### Running Tests

```bash
# Run all unit tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_command_builder

# Run tests with coverage (requires tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Creating New Workflows

### 1. Create Directory Structure

```bash
# From atlas-cli/examples directory
mkdir -p workflows/my_workflow/{data,scripts,models,output,keys}
```

### 2. Create Configuration

```yaml
# workflows/my_workflow/config.yaml
name: "My Workflow"
description: "Description of my workflow"

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
cp your_data.csv workflows/my_workflow/data/
cp your_model.pkl workflows/my_workflow/models/
```

### 4. Run Your Workflow

```bash
./target/release/atlas-test workflows/my_workflow/config.yaml
```

## Troubleshooting

### Atlas CLI Not Found

```bash
# Check if atlas-cli is in PATH
which atlas-cli

# If building from source, add to PATH
cd ../  # to atlas-cli root
cargo build --release
export PATH=$PATH:$(pwd)/target/release
```

### Build Errors

```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build --release

# Check for dependency issues
cargo update
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
# Ensure OpenSSL is installed
openssl version

# Generate keys manually
openssl genpkey -algorithm RSA -out private.pem -pkeyopt rsa_keygen_bits:4096
openssl rsa -pubout -in private.pem -out public.pem
```

## Output Files

After running tests, check the output directory for:

- `commands.log` - Complete command execution log
- `reproduce.sh` - Executable script to reproduce all commands
- `*.json` - Exported manifests
- `provenance_*.json` - Provenance graphs

## MNIST Pipeline Example

The MNIST pipeline example demonstrates a complete ML workflow with provenance tracking:

```bash
# Navigate to mnist directory
cd workflows/mnist

# Install Python dependencies (for training)
poetry install

# Run the complete pipeline
../../target/release/atlas-test mnist_complete_pipeline.yaml
```

This example:
1. Downloads MNIST dataset
2. Trains a CNN model
3. Evaluates the model
4. Creates provenance manifests for all artifacts
5. Links manifests to show complete lineage
6. Exports the full provenance graph

## Additional Resources

- [Atlas CLI Repository](https://github.com/IntelLabs/atlas-cli)
- [Atlas CLI Documentation](https://github.com/IntelLabs/atlas-cli/blob/main/README.md)
- [C2PA Specification](https://c2pa.org)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please see the main [Atlas CLI repository](https://github.com/IntelLabs/atlas-cli) for contribution guidelines.
