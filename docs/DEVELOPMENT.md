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

## Releases

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). To publish a new release of atlas-cli, follow these steps:

1. Update the Cargo.toml `version` field and run `cargo update` in the root directory
2. Run `cargo update` in the `examples` directory
3. Bump any references to the version number in source files (search via `grep -r "<CURRENT_MAJOR>\.<CURRENT_MINOR>\.<CURRENT_PATCH>" ./src`)
4. Add a new entry to CHANGELOG.md
5. Open a PR against the main branch with these changes
7. Create a new release on GitHub with the tag `v<NEW_MAJOR.NEW_MINOR.NEW_PATCH>`
6. Once the PR is merged, pull the latest head of main and new git tag, and follow the [cargo publish steps](https://doc.rust-lang.org/cargo/reference/publishing.html)
