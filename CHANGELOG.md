# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-10-15

### Added
- Implement secure key zeroization for cryptographic operations
- Support for OpenSSF Model Signing (OMS) for C2PA and in-toto formats
- Support for Atlas CLI-specific SLSA Provenance generation
- Atlas CLI test framework

## [0.1.0] - 2025-06-05

### Added
- Initial release of Atlas CLI
- Support for creating C2PA manifests for ML models and datasets
- Cryptographic signing capabilities
- Multiple storage backends (MongoDB, filesystem, Rekor)
- Model format support (ONNX, TensorFlow, PyTorch, Keras)
- TEE attestation support (Intel TDX)
- Cross-reference linking between manifests
- Manifest verification and validation
