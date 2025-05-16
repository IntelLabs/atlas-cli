# Atlas CLI Development Guide

This guide contains information for developers who want to contribute to, build, or modify the Atlas ML CLI tool.

## Development Environment Setup

### Prerequisites

- Rust (latest stable) - [Install Rust](https://rustup.rs/)
- OpenSSL development libraries
- Protobuf Compiler (for TDX attestation features)

### Getting the Source Code

```bash
git clone https://github.com/IntelLabs/atlas-cli
cd atlas-cli
```

### Development Tools

Install development dependencies:

```bash
make dev-deps
```

This installs:
- `cargo-watch` for continuous compilation during development
- `cargo-edit` for easier dependency management

## Build Instructions

### Standard Build

```bash
cargo build
```

### Build with GCP TDX Attestation & Verification

```bash
apt install protobuf-compiler
cargo build --features with-tdx
```

## Testing

Run the full test suite:

```bash
cargo test
```

Run tests with output:

```bash
make test-verbose
```

Watch for changes and run tests automatically:

```bash
make watch-test
```

## Contribution Guidelines

1. Fork the repository
2. Create a feature branch for your changes
3. Make your changes with appropriate tests
4. Run `make check` to ensure formatting, linting, and tests pass
5. Submit a pull request with a clear description of the changes

### Code Style

The project uses `rustfmt` and `clippy` for code formatting and linting:

```bash
make fmt    # Format code
make lint   # Run the linter
```

### Documentation

When adding new features, please update the relevant documentation files and include inline documentation for your code.
