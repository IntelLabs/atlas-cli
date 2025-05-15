# Atlas CLI: Machine Learning (ML) Lifecycle & Transparency Manager

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
git clone https://github.com/IntelLabs/tdx-workload-attestation.git tdx-workload-attestation
git clone https://github.com/intel-sandbox/frameworks.orchestration.c2pa_ml.rust.git c2pa_ml
git clone https://github.com/intel-sandbox/frameworks.orchestration.c2pa_ml_cli.rust.git c2pa_ml_cli

# Build CLI
cd c2pa_ml_cli && cargo build
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
