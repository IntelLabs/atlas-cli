# Atlas CLI User Guide

This guide provides detailed information on installing, configuring, and using the C2PA ML CLI tool.

## Table of Contents

- [Installation](#installation)
- [Command Line Reference](#command-line-reference)
- [Configuration Options](#configuration-options)
- [Storage Backends](#storage-backends)
- [TEE Attestation](#tee-attestation)
- [Troubleshooting](#troubleshooting)

## Installation

### Prerequisites

- Rust toolchain (1.58 or later)
- OpenSSL development libraries

### Standard Installation

```bash
# Clone repositories
git clone https://github.com/IntelLabs/atlas-cli

# Build CLI
cd atlas-cli && make

# Install (optional)
make install
```

### Installation with TDX Attestation Support

The Atlas CLI currently supports Intel TDX 1.5
or later for Ubuntu systems on select Google
Cloud Engine instances.

```bash
apt install protobuf-compiler
cargo build --features with-tdx
make install
```

## Command Line Reference

The C2PA ML CLI provides the following main commands:

### Model Commands

```
atlas-cli model [SUBCOMMAND]
```

Subcommands:
- `create` - Create a new model manifest
- `list` - List all model manifests
- `verify` - Verify a model manifest
- `link-dataset` - Link a dataset to a model

### Dataset Commands

```
atlas-cli dataset [SUBCOMMAND]
```

Subcommands:
- `create` - Create a new dataset manifest
- `list` - List all dataset manifests
- `verify` - Verify a dataset manifest

### Manifest Commands

```
atlas-cli manifest [SUBCOMMAND]
```

Subcommands:
- `link` - Link manifests together
- `show` - Show manifest details
- `validate` - Validate manifest cross-references
- `verify-link` - Verify a specific link between two manifests
- `export` - Export provenance graph information

### Evaluation Commands

```
atlas-cli evaluation [SUBCOMMAND]
```

Subcommands:
- `create` - Create a new evaluation result manifest
- `list` - List all evaluation results
- `verify` - Verify an evaluation result manifest

### Software Commands

```
atlas-cli software [SUBCOMMAND]
```

Subcommands:
- `create` - Create a new software component manifest
- `list` - List all software component manifests
- `verify` - Verify a software component manifest
- `link-model` - Link software to a model
- `link-dataset` - Link software to a dataset

## Configuration Options

### Keys for Signing

Generate signing keys:

```bash
make generate-keys
```

This creates:
- `private.pem` - Private key for signing
- `public.pem` - Public key for verification

### Output Formats

The CLI supports two output formats:
- `json` - Human-readable JSON (default)
- `cbor` - Compact binary format

Specify the format using the `--format` flag:

```bash
atlas-cli model create --format=json ...
atlas-cli model create --format=cbor ...
```

### Common Flags

Most commands support the following flags:

- `--print` - Display the manifest without storing it
- `--key=<path>` - Path to private key for signing
- `--storage-type=<type>` - Storage backend type (database, filesystem)
- `--storage-url=<url>` - URL or path for the storage backend

## Storage Backends

### Database Storage

Uses a custom HTTP API with MongoDB backend:

```bash
atlas-cli model create \
    --storage-type=database \
    --storage-url=http://localhost:8080 \
    ...
```

### Filesystem Storage

Stores manifests in the local filesystem:

```bash
atlas-cli model create \
    --storage-type=filesystem \
    --storage-url=./storage \
    ...
```

### Rekor Storage

Stores manifests in a Rekor transparency log:

```bash
export REKOR_URL=https://rekor.example.com
atlas-cli model create \
    --storage-type=rekor \
    ...
```

## TDX Attestation

When built with the `with-tdx` feature, you can both create attested manifests and verify
the guest integrity:

```bash
atlas-cli model create \
    --with-tdx \
    ...
```

## Security Configuration

### Key Generation and Management

**Important**: Cryptographic key generation and management are the responsibility of the end user. The security of your C2PA manifests depends entirely on the strength and proper management of your keys.

Generate signing keys using the provided Makefile:

```bash
make generate-keys
```

This creates:

private.pem - Private key for signing
public.pem - Public key for verification

For custom key generation with specific requirements:

```bash
# Generate a strong 4096-bit RSA private key (recommended)
openssl genpkey -algorithm RSA -out private_key.pem -pkeyopt rsa_keygen_bits:4096

# Extract the public key
openssl rsa -pubout -in private_key.pem -out public_key.pem
```

Key Requirements:

- RSA keys: minimum 2048 bits (4096 bits recommended)
- EC keys: use approved curves (P-256, P-384, or P-521)
- Follow [key management best practices](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-57pt1r5.pdf), and use a well-known key management service (KMS) when possible.
- Never commit private keys to version control
- Rotate keys regularly according to your security policy

### Hash Algorithms

The Atlas CLI supports multiple hash algorithms for signing manifests:
```bash

Uses SHA-384 by default (recommended)

atlas-cli model create --key=private.pem ...

# Use SHA-512 for maximum security
atlas-cli model create --key=private.pem --hash-algo=sha512 ...

```
Available algorithms:

- sha384 - Default, recommended for security/performance balance
- sha256 - For backward compatibility with existing systems
- sha512 - Maximum security for sensitive applications

The --hash-alg flag is supported by all creation commands:

```
model create
dataset create
software create
evaluation create
```

## Supported Formats

### Models
- ONNX (.onnx)
- TensorFlow (.pb)
- PyTorch (.pt, .pth)
- Keras (.h5)

### Datasets
- Any directory structure
- Common formats: CSV, JSON, NPY, etc.

## Troubleshooting

### Common Issues

#### Storage Connection Errors

If you encounter errors connecting to a storage backend:

1. Verify the storage URL is correct
2. Check if the storage service is running
3. Verify network connectivity
4. For database storage, check MongoDB is running

#### Signing Errors

If you encounter signing-related errors:

1. Verify the private key path is correct
2. Ensure the key is in PEM format
3. Check file permissions on the key file

#### File Not Found Errors

If you encounter "file not found" errors when creating manifests:

1. Verify the paths provided exist
2. Use absolute paths to avoid working directory issues
3. Check file permissions

### Getting Help

For more detailed help on any command, you can use the `--help` flag:

```bash
atlas-cli --help
atlas-cli model --help
atlas-cli model create --help
```

### Logging

To enable debug logging, set the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug atlas-cli ...
```
