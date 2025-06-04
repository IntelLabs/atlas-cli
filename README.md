![GitHub License](https://img.shields.io/github/license/IntelLabs/atlas-cli)
[![Crates.io](https://img.shields.io/crates/v/atlas-c2pa-lib.svg)](https://crates.io/crates/atlas-cli)
[![Documentation](https://docs.rs/atlas-c2pa-lib/badge.svg)](https://docs.rs/atlas-cli)
[![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/IntelLabs/atlas-cli/badge)](https://scorecard.dev/viewer/?uri=github.com/IntelLabs/atlas-cli)


# Atlas CLI: Machine Learning (ML) Lifecycle & Transparency Manager

⚠️ **Disclaimer**: This project is currently in active development. The code is **not stable** and **not intended for use in production environments**. Interfaces, features, and behaviors are subject to change without notice.

A command-line interface tool for creating, managing, and verifying Content Provenance and Authenticity (C2PA) manifests for machine learning models, datasets, and related artifacts.

## Key Features

- **Model & Dataset Manifests**: Create C2PA-compliant manifests for ML models and datasets
- **Cryptographic Signing**: Sign manifests with cryptographic keys for authenticity verification
- **Provenance Linking**: Create verifiable links between models, datasets, and ML assets
- **Multiple Storage Types**: Store manifests in MongoDB, Rekor log, or filesystem backends
- **Format Support**: Work with models in ONNX, TensorFlow, PyTorch, and Keras formats
- **TEE Attestation**: Optional support for Trusted Execution Environment (TDX) integration

## Quick Installation

```bash
# Clone repositories
git clone https://github.com/IntelLabs/atlas-cli

# Build CLI
cd atlas-cli && cargo build
```

## Documentation

For more detailed information, please refer to:

- [User Guide](docs/USER_GUIDE.md) - Installation, configuration, and command reference
- [Development Guide](docs/DEVELOPMENT.md) - Contributing, building, and architecture
- [Examples](docs/EXAMPLES.md) - Usage examples and workflow patterns

## License

This project is licensed under the Apache 2.0 License - see the LICENSE file for details.


## Citation

If you use Atlas CLI in your research or work, please cite our paper:

```bibtex
@misc{atlas2025github,
      title={Atlas: A Framework for ML Lifecycle Provenance & Transparency},
      author={Marcin Spoczynski and Marcela S. Melara and Sebastian Szyller},
      year={2025},
      eprint={2502.19567},
      archivePrefix={arXiv},
      primaryClass={cs.CR},
      url={https://arxiv.org/abs/2502.19567v1}
}
```

## Related Resources

- **Paper**: [Atlas: A Framework for ML Lifecycle Provenance & Transparency](https://arxiv.org/abs/2502.19567v1)
- **Blog Post**: [Building Trust in AI: An End-to-End Approach for the Machine Learning Lifecycle](https://community.intel.com/t5/Blogs/Tech-Innovation/Artificial-Intelligence-AI/Building-Trust-in-AI-An-End-to-End-Approach-for-the-Machine/post/1648746)
