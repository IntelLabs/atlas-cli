use crate::error::{Error, Result};
use crate::storage::traits::ManifestType;
use atlas_c2pa_lib::assertion::Assertion;
use atlas_c2pa_lib::asset_type::AssetType;
use atlas_c2pa_lib::manifest::Manifest;
use std::path::Path;

pub fn determine_model_type(path: &Path) -> Result<AssetType> {
    match path.extension().and_then(|ext| ext.to_str()) {
        // TensorFlow models
        Some("pb") | Some("savedmodel") | Some("tf") => Ok(AssetType::ModelTensorFlow),

        // PyTorch models
        Some("pt") | Some("pth") | Some("pytorch") => Ok(AssetType::ModelPytorch),

        // ONNX models
        Some("onnx") => Ok(AssetType::ModelOnnx),

        // OpenVINO models
        Some("bin") | Some("xml") => Ok(AssetType::ModelOpenVino),

        // Keras models
        Some("h5") | Some("keras") | Some("hdf5") => Ok(AssetType::ModelKeras),

        // JAX models
        Some("jax") => Ok(AssetType::ModelJax),

        // ML.NET models
        Some("mlnet") | Some("zip") => Ok(AssetType::ModelMlNet),

        // MXNet models
        Some("params") | Some("json") | Some("mxnet") => Ok(AssetType::ModelMxNet),

        // Format types
        Some("npy") | Some("npz") => Ok(AssetType::FormatNumpy),
        Some("protobuf") | Some("proto") => Ok(AssetType::FormatProtobuf),
        Some("pkl") | Some("pickle") => Ok(AssetType::FormatPickle),

        // Default generic model when extension doesn't match
        Some(_) => Ok(AssetType::Model),

        // No extension
        None => Err(Error::Validation(
            "Unsupported model format: file has no extension".to_string(),
        )),
    }
}

pub fn determine_format(path: &Path) -> Result<String> {
    match path.extension().and_then(|ext| ext.to_str()) {
        // TensorFlow models
        Some("pb") => Ok("application/x-protobuf".to_string()),
        Some("savedmodel") | Some("tf") => Ok("application/x-tensorflow".to_string()),

        // PyTorch models
        Some("pt") | Some("pth") | Some("pytorch") => Ok("application/x-pytorch".to_string()),

        // ONNX models
        Some("onnx") => Ok("application/onnx".to_string()),

        // OpenVINO models
        Some("bin") | Some("xml") => Ok("application/x-openvino".to_string()),

        // Keras models
        Some("h5") | Some("keras") | Some("hdf5") => Ok("application/x-hdf5".to_string()),

        // JAX models
        Some("jax") => Ok("application/x-jax".to_string()),

        // ML.NET models
        Some("mlnet") => Ok("application/x-mlnet".to_string()),
        Some("zip") => Ok("application/zip".to_string()),

        // MXNet models
        Some("params") | Some("mxnet") => Ok("application/x-mxnet".to_string()),
        Some("json") => Ok("application/json".to_string()),

        // Format types
        Some("npy") | Some("npz") => Ok("application/x-numpy".to_string()),
        Some("protobuf") | Some("proto") => Ok("application/x-protobuf".to_string()),
        Some("pkl") | Some("pickle") => Ok("application/x-pickle".to_string()),

        // Default
        _ => Ok("application/octet-stream".to_string()),
    }
}

pub fn determine_software_type(path: &Path) -> Result<AssetType> {
    // Use AssetType::Generator for all software files related to ML
    match path.extension().and_then(|ext| ext.to_str()) {
        // Data science and ML languages
        Some("py") => Ok(AssetType::Generator), // Python - primary ML language
        Some("ipynb") => Ok(AssetType::Generator), // Jupyter notebooks - common for ML experimentation
        Some("r") => Ok(AssetType::Generator),     // R - statistical computing
        Some("jl") => Ok(AssetType::Generator),    // Julia - scientific computing

        // ML build and environment files
        Some("dockerfile") | Some("Dockerfile") => Ok(AssetType::Generator), // Docker containers for ML
        Some("yaml") | Some("yml") => Ok(AssetType::Generator), // Config files (env, kubernetes, etc)
        Some("toml") => Ok(AssetType::Generator), // Config files (often used with Python)
        Some("json") => Ok(AssetType::Generator), // Config files and model metadata

        // ML framework specific
        Some("pbtxt") => Ok(AssetType::Generator), // Tensorflow protobuf text
        Some("prototxt") => Ok(AssetType::Generator), // Caffe model definition

        // Lower-level ML implementation languages
        Some("rs") => Ok(AssetType::Generator), // Rust (used in some ML frameworks)
        Some("cpp") | Some("cc") | Some("cxx") => Ok(AssetType::Generator), // C++ (backend of many ML frameworks)
        Some("cu") | Some("cuh") => Ok(AssetType::Generator), // CUDA files for GPU acceleration

        // Shell scripts and utilities
        Some("sh") => Ok(AssetType::Generator), // Shell scripts for automation
        Some("bash") => Ok(AssetType::Generator), // Bash scripts

        // Dataset processing scripts
        Some("sql") => Ok(AssetType::Generator), // SQL for data extraction

        // VM configuration
        Some("vmx") | Some("ovf") | Some("ova") => Ok(AssetType::Generator), // VM configuration files

        // Default for unknown formats
        _ => Ok(AssetType::Generator),
    }
}

pub fn determine_dataset_type(path: &Path) -> Result<AssetType> {
    match path.extension().and_then(|ext| ext.to_str()) {
        // Common dataset formats
        Some("csv") | Some("tsv") | Some("txt") => Ok(AssetType::Dataset),

        // JSON-based datasets
        Some("json") | Some("jsonl") => Ok(AssetType::Dataset),

        // Parquet and other columnar formats
        Some("parquet") | Some("orc") | Some("avro") => Ok(AssetType::Dataset),

        // TensorFlow specific datasets
        Some("tfrecord") | Some("tfrec") => Ok(AssetType::DatasetTensorFlow),
        Some("pb") | Some("proto") | Some("tf") => Ok(AssetType::DatasetTensorFlow),

        // PyTorch specific datasets
        Some("pt") | Some("pth") | Some("pytorch") => Ok(AssetType::DatasetPytorch),

        // ONNX specific datasets
        Some("onnx") => Ok(AssetType::DatasetOnnx),

        // OpenVINO specific datasets
        Some("bin") | Some("xml") => Ok(AssetType::DatasetOpenVino),

        // Keras specific datasets
        Some("h5") | Some("hdf5") | Some("keras") => Ok(AssetType::DatasetKeras),

        // JAX specific datasets
        Some("jax") => Ok(AssetType::DatasetJax),

        // ML.NET specific datasets
        Some("mlnet") | Some("zip") => Ok(AssetType::DatasetMlNet),

        // MXNet specific datasets
        Some("rec") | Some("idx") | Some("params") | Some("lst") | Some("mxnet") => {
            Ok(AssetType::DatasetMxNet)
        }

        // NumPy formats (could be any framework)
        Some("npy") | Some("npz") => Ok(AssetType::Dataset),

        // Pickle formats (could be any framework)
        Some("pkl") | Some("pickle") => Ok(AssetType::Dataset),

        // Images and other media that might be datasets
        Some("jpg") | Some("jpeg") | Some("png") | Some("bmp") | Some("tiff") => {
            Ok(AssetType::Dataset)
        }

        // Default to generic dataset for any other extension
        Some(_) => Ok(AssetType::Dataset),

        // No extension
        None => Err(Error::Validation(
            "Unsupported dataset format: file has no extension".to_string(),
        )),
    }
}
/// This function examines the ingredients and assertions in the manifest
/// to determine whether it's a Dataset, Model, Software, or other type.
pub fn determine_manifest_type(manifest: &Manifest) -> ManifestType {
    // Check for Dataset assertion
    let has_dataset_assertion = (if let Some(claim) = &manifest.claim_v2 {
        claim.created_assertions.iter().any(|assertion| {
            if let Assertion::CreativeWork(creative_work) = assertion {
                creative_work.creative_type == "Dataset"
            } else {
                false
            }
        })
    } else {
        false
    }) || manifest.claim.created_assertions.iter().any(|assertion| {
        if let Assertion::CreativeWork(creative_work) = assertion {
            creative_work.creative_type == "Dataset"
        } else {
            false
        }
    });

    // Check for Dataset ingredients
    let has_dataset_ingredients = manifest.ingredients.iter().any(|ingredient| {
        ingredient.data.data_types.iter().any(|t| {
            matches!(
                t,
                AssetType::Dataset
                    | AssetType::DatasetOnnx
                    | AssetType::DatasetTensorFlow
                    | AssetType::DatasetPytorch
            )
        })
    });

    // If we have a Dataset assertion OR Dataset ingredients, return Dataset
    if has_dataset_assertion || has_dataset_ingredients {
        return ManifestType::Dataset;
    }

    // Check for Software assertion
    let has_software_assertion = (if let Some(claim) = &manifest.claim_v2 {
        claim.created_assertions.iter().any(|assertion| {
            if let Assertion::CreativeWork(creative_work) = assertion {
                creative_work.creative_type == "Software"
            } else {
                false
            }
        })
    } else {
        false
    }) || manifest.claim.created_assertions.iter().any(|assertion| {
        if let Assertion::CreativeWork(creative_work) = assertion {
            creative_work.creative_type == "Software"
        } else {
            false
        }
    });

    // Check for Software parameters in Action assertions
    let has_software_parameters = (if let Some(claim) = &manifest.claim_v2 {
        claim.created_assertions.iter().any(|assertion| {
            if let Assertion::Action(action_assertion) = assertion {
                action_assertion.actions.iter().any(|action| {
                    if let Some(params) = &action.parameters {
                        params.get("software_type").is_some()
                    } else {
                        false
                    }
                })
            } else {
                false
            }
        })
    } else {
        false
    }) || manifest.claim.created_assertions.iter().any(|assertion| {
        if let Assertion::Action(action_assertion) = assertion {
            action_assertion.actions.iter().any(|action| {
                if let Some(params) = &action.parameters {
                    params.get("software_type").is_some()
                } else {
                    false
                }
            })
        } else {
            false
        }
    });

    // Check for Software ingredients
    let has_software_ingredients = manifest.ingredients.iter().any(|ingredient| {
        ingredient
            .data
            .data_types
            .iter()
            .any(|t| matches!(t, AssetType::Generator))
    });

    // If we have a Software assertion OR Software parameters OR Software ingredients, return Software
    if has_software_assertion || has_software_parameters || has_software_ingredients {
        return ManifestType::Software;
    }

    // Check for Model assertion
    let has_model_assertion = (if let Some(claim) = &manifest.claim_v2 {
        claim.created_assertions.iter().any(|assertion| {
            if let Assertion::CreativeWork(creative_work) = assertion {
                creative_work.creative_type == "Model"
            } else {
                false
            }
        })
    } else {
        false
    }) || manifest.claim.created_assertions.iter().any(|assertion| {
        if let Assertion::CreativeWork(creative_work) = assertion {
            creative_work.creative_type == "Model"
        } else {
            false
        }
    });

    // Check for Model ingredients
    let has_model_ingredients = manifest.ingredients.iter().any(|ingredient| {
        ingredient.data.data_types.iter().any(|t| {
            matches!(
                t,
                AssetType::Model
                    | AssetType::ModelOnnx
                    | AssetType::ModelTensorFlow
                    | AssetType::ModelPytorch
                    | AssetType::ModelOpenVino
            )
        })
    });

    // If we have a Model assertion OR Model ingredients, return Model
    if has_model_assertion || has_model_ingredients {
        return ManifestType::Model;
    }

    // Default case if nothing else matches
    ManifestType::Unknown
}
/// Get a string representation of the manifest type
pub fn manifest_type_to_string(manifest_type: &ManifestType) -> String {
    manifest_type.to_string()
}

/// Get the manifest type as a static string
pub fn manifest_type_to_str(manifest_type: &ManifestType) -> &'static str {
    match manifest_type {
        ManifestType::Dataset => "Dataset",
        ManifestType::Model => "Model",
        ManifestType::Software => "Software",
        ManifestType::Unknown => "Unknown",
    }
}

/// Convert a manifest type string to ManifestType enum
pub fn parse_manifest_type(type_str: &str) -> ManifestType {
    match type_str.to_lowercase().as_str() {
        "dataset" => ManifestType::Dataset,
        "software" => ManifestType::Software,
        _ => ManifestType::Model, // Default to Model for unknown types
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;
    use atlas_c2pa_lib::assertion::{Assertion, Author, CreativeWorkAssertion};
    use atlas_c2pa_lib::asset_type::AssetType;
    use atlas_c2pa_lib::manifest::Manifest;
    use std::path::PathBuf;

    #[test]
    fn test_determine_model_type() -> Result<()> {
        // Test TensorFlow model types
        assert_eq!(
            determine_model_type(&PathBuf::from("model.pb"))?,
            AssetType::ModelTensorFlow
        );
        assert_eq!(
            determine_model_type(&PathBuf::from("model.savedmodel"))?,
            AssetType::ModelTensorFlow
        );
        assert_eq!(
            determine_model_type(&PathBuf::from("model.tf"))?,
            AssetType::ModelTensorFlow
        );

        // Test PyTorch model types
        assert_eq!(
            determine_model_type(&PathBuf::from("model.pt"))?,
            AssetType::ModelPytorch
        );
        assert_eq!(
            determine_model_type(&PathBuf::from("model.pth"))?,
            AssetType::ModelPytorch
        );
        assert_eq!(
            determine_model_type(&PathBuf::from("model.pytorch"))?,
            AssetType::ModelPytorch
        );

        // Test ONNX model types
        assert_eq!(
            determine_model_type(&PathBuf::from("model.onnx"))?,
            AssetType::ModelOnnx
        );

        // Test OpenVINO model types
        assert_eq!(
            determine_model_type(&PathBuf::from("model.bin"))?,
            AssetType::ModelOpenVino
        );
        assert_eq!(
            determine_model_type(&PathBuf::from("model.xml"))?,
            AssetType::ModelOpenVino
        );

        // Test Keras model types
        assert_eq!(
            determine_model_type(&PathBuf::from("model.h5"))?,
            AssetType::ModelKeras
        );
        assert_eq!(
            determine_model_type(&PathBuf::from("model.keras"))?,
            AssetType::ModelKeras
        );
        assert_eq!(
            determine_model_type(&PathBuf::from("model.hdf5"))?,
            AssetType::ModelKeras
        );

        // Test JAX model types
        assert_eq!(
            determine_model_type(&PathBuf::from("model.jax"))?,
            AssetType::ModelJax
        );

        // Test ML.NET model types
        assert_eq!(
            determine_model_type(&PathBuf::from("model.mlnet"))?,
            AssetType::ModelMlNet
        );
        assert_eq!(
            determine_model_type(&PathBuf::from("model.zip"))?,
            AssetType::ModelMlNet
        );

        // Test MXNet model types
        assert_eq!(
            determine_model_type(&PathBuf::from("model.params"))?,
            AssetType::ModelMxNet
        );
        assert_eq!(
            determine_model_type(&PathBuf::from("model.json"))?,
            AssetType::ModelMxNet
        );
        assert_eq!(
            determine_model_type(&PathBuf::from("model.mxnet"))?,
            AssetType::ModelMxNet
        );

        // Test format types
        assert_eq!(
            determine_model_type(&PathBuf::from("model.npy"))?,
            AssetType::FormatNumpy
        );
        assert_eq!(
            determine_model_type(&PathBuf::from("model.npz"))?,
            AssetType::FormatNumpy
        );
        assert_eq!(
            determine_model_type(&PathBuf::from("model.protobuf"))?,
            AssetType::FormatProtobuf
        );
        assert_eq!(
            determine_model_type(&PathBuf::from("model.proto"))?,
            AssetType::FormatProtobuf
        );
        assert_eq!(
            determine_model_type(&PathBuf::from("model.pkl"))?,
            AssetType::FormatPickle
        );
        assert_eq!(
            determine_model_type(&PathBuf::from("model.pickle"))?,
            AssetType::FormatPickle
        );

        // Test generic model type for unknown extension
        assert_eq!(
            determine_model_type(&PathBuf::from("model.unknown"))?,
            AssetType::Model
        );

        // Test error for no extension
        let result = determine_model_type(&PathBuf::from("model"));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_determine_format() -> Result<()> {
        // Test TensorFlow formats
        assert_eq!(
            determine_format(&PathBuf::from("model.pb"))?,
            "application/x-protobuf"
        );
        assert_eq!(
            determine_format(&PathBuf::from("model.savedmodel"))?,
            "application/x-tensorflow"
        );
        assert_eq!(
            determine_format(&PathBuf::from("model.tf"))?,
            "application/x-tensorflow"
        );

        // Test PyTorch formats
        assert_eq!(
            determine_format(&PathBuf::from("model.pt"))?,
            "application/x-pytorch"
        );
        assert_eq!(
            determine_format(&PathBuf::from("model.pth"))?,
            "application/x-pytorch"
        );
        assert_eq!(
            determine_format(&PathBuf::from("model.pytorch"))?,
            "application/x-pytorch"
        );

        // Test ONNX formats
        assert_eq!(
            determine_format(&PathBuf::from("model.onnx"))?,
            "application/onnx"
        );

        // Test OpenVINO formats
        assert_eq!(
            determine_format(&PathBuf::from("model.bin"))?,
            "application/x-openvino"
        );
        assert_eq!(
            determine_format(&PathBuf::from("model.xml"))?,
            "application/x-openvino"
        );

        // Test Keras formats
        assert_eq!(
            determine_format(&PathBuf::from("model.h5"))?,
            "application/x-hdf5"
        );
        assert_eq!(
            determine_format(&PathBuf::from("model.keras"))?,
            "application/x-hdf5"
        );
        assert_eq!(
            determine_format(&PathBuf::from("model.hdf5"))?,
            "application/x-hdf5"
        );

        // Test JAX formats
        assert_eq!(
            determine_format(&PathBuf::from("model.jax"))?,
            "application/x-jax"
        );

        // Test ML.NET formats
        assert_eq!(
            determine_format(&PathBuf::from("model.mlnet"))?,
            "application/x-mlnet"
        );
        assert_eq!(
            determine_format(&PathBuf::from("model.zip"))?,
            "application/zip"
        );

        // Test MXNet formats
        assert_eq!(
            determine_format(&PathBuf::from("model.params"))?,
            "application/x-mxnet"
        );
        assert_eq!(
            determine_format(&PathBuf::from("model.json"))?,
            "application/json"
        );
        assert_eq!(
            determine_format(&PathBuf::from("model.mxnet"))?,
            "application/x-mxnet"
        );

        // Test format types
        assert_eq!(
            determine_format(&PathBuf::from("model.npy"))?,
            "application/x-numpy"
        );
        assert_eq!(
            determine_format(&PathBuf::from("model.npz"))?,
            "application/x-numpy"
        );
        assert_eq!(
            determine_format(&PathBuf::from("model.protobuf"))?,
            "application/x-protobuf"
        );
        assert_eq!(
            determine_format(&PathBuf::from("model.proto"))?,
            "application/x-protobuf"
        );
        assert_eq!(
            determine_format(&PathBuf::from("model.pkl"))?,
            "application/x-pickle"
        );
        assert_eq!(
            determine_format(&PathBuf::from("model.pickle"))?,
            "application/x-pickle"
        );

        // Test default format
        assert_eq!(
            determine_format(&PathBuf::from("model.unknown"))?,
            "application/octet-stream"
        );
        assert_eq!(
            determine_format(&PathBuf::from("model"))?,
            "application/octet-stream"
        );

        Ok(())
    }

    #[test]
    fn test_determine_software_type() -> Result<()> {
        // Test data science and ML languages
        assert_eq!(
            determine_software_type(&PathBuf::from("script.py"))?,
            AssetType::Generator
        );
        assert_eq!(
            determine_software_type(&PathBuf::from("notebook.ipynb"))?,
            AssetType::Generator
        );
        assert_eq!(
            determine_software_type(&PathBuf::from("analysis.r"))?,
            AssetType::Generator
        );
        assert_eq!(
            determine_software_type(&PathBuf::from("compute.jl"))?,
            AssetType::Generator
        );

        // Test ML build and environment files
        assert_eq!(
            determine_software_type(&PathBuf::from("Dockerfile"))?,
            AssetType::Generator
        );
        assert_eq!(
            determine_software_type(&PathBuf::from("config.yaml"))?,
            AssetType::Generator
        );
        assert_eq!(
            determine_software_type(&PathBuf::from("config.yml"))?,
            AssetType::Generator
        );
        assert_eq!(
            determine_software_type(&PathBuf::from("pyproject.toml"))?,
            AssetType::Generator
        );
        assert_eq!(
            determine_software_type(&PathBuf::from("config.json"))?,
            AssetType::Generator
        );

        // Test ML framework specific
        assert_eq!(
            determine_software_type(&PathBuf::from("model.pbtxt"))?,
            AssetType::Generator
        );
        assert_eq!(
            determine_software_type(&PathBuf::from("model.prototxt"))?,
            AssetType::Generator
        );

        // Test lower-level implementation languages
        assert_eq!(
            determine_software_type(&PathBuf::from("wrapper.rs"))?,
            AssetType::Generator
        );
        assert_eq!(
            determine_software_type(&PathBuf::from("kernels.cpp"))?,
            AssetType::Generator
        );
        assert_eq!(
            determine_software_type(&PathBuf::from("kernels.cc"))?,
            AssetType::Generator
        );
        assert_eq!(
            determine_software_type(&PathBuf::from("algo.cu"))?,
            AssetType::Generator
        );

        // Test scripts
        assert_eq!(
            determine_software_type(&PathBuf::from("build.sh"))?,
            AssetType::Generator
        );
        assert_eq!(
            determine_software_type(&PathBuf::from("setup.bash"))?,
            AssetType::Generator
        );
        assert_eq!(
            determine_software_type(&PathBuf::from("query.sql"))?,
            AssetType::Generator
        );

        // Test VM configuration
        assert_eq!(
            determine_software_type(&PathBuf::from("config.vmx"))?,
            AssetType::Generator
        );
        assert_eq!(
            determine_software_type(&PathBuf::from("template.ovf"))?,
            AssetType::Generator
        );
        assert_eq!(
            determine_software_type(&PathBuf::from("image.ova"))?,
            AssetType::Generator
        );

        // Test default for unknown extension
        assert_eq!(
            determine_software_type(&PathBuf::from("unknown.ext"))?,
            AssetType::Generator
        );
        assert_eq!(
            determine_software_type(&PathBuf::from("no_extension"))?,
            AssetType::Generator
        );

        Ok(())
    }

    #[test]
    fn test_determine_dataset_type() -> Result<()> {
        // Test common dataset formats
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.csv"))?,
            AssetType::Dataset
        );
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.tsv"))?,
            AssetType::Dataset
        );
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.txt"))?,
            AssetType::Dataset
        );

        // Test JSON-based datasets
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.json"))?,
            AssetType::Dataset
        );
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.jsonl"))?,
            AssetType::Dataset
        );

        // Test columnar formats
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.parquet"))?,
            AssetType::Dataset
        );
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.orc"))?,
            AssetType::Dataset
        );
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.avro"))?,
            AssetType::Dataset
        );

        // Test framework-specific datasets
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.tfrecord"))?,
            AssetType::DatasetTensorFlow
        );
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.tfrec"))?,
            AssetType::DatasetTensorFlow
        );
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.pb"))?,
            AssetType::DatasetTensorFlow
        );

        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.pt"))?,
            AssetType::DatasetPytorch
        );
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.pth"))?,
            AssetType::DatasetPytorch
        );

        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.onnx"))?,
            AssetType::DatasetOnnx
        );

        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.bin"))?,
            AssetType::DatasetOpenVino
        );
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.xml"))?,
            AssetType::DatasetOpenVino
        );

        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.h5"))?,
            AssetType::DatasetKeras
        );
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.hdf5"))?,
            AssetType::DatasetKeras
        );

        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.jax"))?,
            AssetType::DatasetJax
        );

        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.mlnet"))?,
            AssetType::DatasetMlNet
        );

        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.rec"))?,
            AssetType::DatasetMxNet
        );
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.idx"))?,
            AssetType::DatasetMxNet
        );

        // Test NumPy and Pickle formats
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.npy"))?,
            AssetType::Dataset
        );
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.npz"))?,
            AssetType::Dataset
        );
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.pkl"))?,
            AssetType::Dataset
        );
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.pickle"))?,
            AssetType::Dataset
        );

        // Test image formats as datasets
        assert_eq!(
            determine_dataset_type(&PathBuf::from("images.jpg"))?,
            AssetType::Dataset
        );
        assert_eq!(
            determine_dataset_type(&PathBuf::from("images.jpeg"))?,
            AssetType::Dataset
        );
        assert_eq!(
            determine_dataset_type(&PathBuf::from("images.png"))?,
            AssetType::Dataset
        );

        // Test generic dataset for unknown extension
        assert_eq!(
            determine_dataset_type(&PathBuf::from("data.unknown"))?,
            AssetType::Dataset
        );

        // Test error for no extension
        let result = determine_dataset_type(&PathBuf::from("dataset"));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_determine_manifest_type() {
        // Create a basic manifest first
        let mut manifest = create_test_manifest();

        // Set ingredient to Dataset type
        manifest.ingredients[0].data.data_types = vec![AssetType::Dataset];
        println!("DEBUG: Set ingredient data_type to Dataset");

        // Add Model assertions
        let model_assertion = Assertion::CreativeWork(CreativeWorkAssertion {
            context: "http://schema.org/".to_string(),
            creative_type: "Model".to_string(),
            author: vec![Author {
                author_type: "Organization".to_string(),
                name: "Test Org".to_string(),
            }],
        });

        // Clear existing assertions first
        if let Some(claim) = &mut manifest.claim_v2 {
            claim.created_assertions.clear();
            // Add Model assertion
            claim.created_assertions.push(model_assertion.clone());
            println!("DEBUG: Added Model assertion to claim_v2");
        }

        manifest.claim.created_assertions.clear();
        manifest.claim.created_assertions.push(model_assertion);
        println!("DEBUG: Added Model assertion to legacy claim");

        let manifest_type = determine_manifest_type(&manifest);
        assert_eq!(
            manifest_type_to_str(&manifest_type),
            "Dataset",
            "Should detect Dataset from ingredients when there's no matching Dataset assertion"
        );
    }

    #[test]
    fn test_determine_manifest_type_with_action_parameters() {
        // Create a basic manifest
        let mut manifest = create_test_manifest();

        // Set ingredient type to a neutral type
        manifest.ingredients[0].data.data_types = vec![AssetType::Model];

        // Create an Action assertion with software_type parameter
        if let Some(claim) = &mut manifest.claim_v2 {
            claim.created_assertions = vec![Assertion::Action(
                atlas_c2pa_lib::assertion::ActionAssertion {
                    actions: vec![atlas_c2pa_lib::assertion::Action {
                        action: "c2pa.created".to_string(),
                        software_agent: Some("test_agent".to_string()),
                        parameters: Some(serde_json::json!({
                            "software_type": "container_image",
                            "version": "1.0.0"
                        })),
                        digital_source_type: Some(
                            "http://cv.iptc.org/newscodes/digitalsourcetype/software".to_string(),
                        ),
                        instance_id: None,
                    }],
                },
            )];
        }

        // Also update the legacy claim
        manifest.claim.created_assertions = vec![Assertion::Action(
            atlas_c2pa_lib::assertion::ActionAssertion {
                actions: vec![atlas_c2pa_lib::assertion::Action {
                    action: "c2pa.created".to_string(),
                    software_agent: Some("test_agent".to_string()),
                    parameters: Some(serde_json::json!({
                        "software_type": "container_image",
                        "version": "1.0.0"
                    })),
                    digital_source_type: Some(
                        "http://cv.iptc.org/newscodes/digitalsourcetype/software".to_string(),
                    ),
                    instance_id: None,
                }],
            },
        )];

        // Even with Model ingredients, the st parameter in Action should mark it as Software
        let manifest_type = determine_manifest_type(&manifest);
        assert_eq!(manifest_type_to_str(&manifest_type), "Software");
    }

    #[test]
    fn test_manifest_type_conversion() {
        use crate::storage::traits::ManifestType;

        // Test conversion to string
        assert_eq!(manifest_type_to_string(&ManifestType::Dataset), "Dataset");
        assert_eq!(manifest_type_to_string(&ManifestType::Model), "Model");
        assert_eq!(manifest_type_to_string(&ManifestType::Software), "Software");

        // Test conversion to str
        assert_eq!(manifest_type_to_str(&ManifestType::Dataset), "Dataset");
        assert_eq!(manifest_type_to_str(&ManifestType::Model), "Model");
        assert_eq!(manifest_type_to_str(&ManifestType::Software), "Software");

        // Test parsing from string
        assert_eq!(parse_manifest_type("dataset"), ManifestType::Dataset);
        assert_eq!(parse_manifest_type("Dataset"), ManifestType::Dataset);
        assert_eq!(parse_manifest_type("DATASET"), ManifestType::Dataset);

        assert_eq!(parse_manifest_type("software"), ManifestType::Software);
        assert_eq!(parse_manifest_type("Software"), ManifestType::Software);
        assert_eq!(parse_manifest_type("SOFTWARE"), ManifestType::Software);

        // Test default to Model for unknown types
        assert_eq!(parse_manifest_type("model"), ManifestType::Model);
        assert_eq!(parse_manifest_type("unknown"), ManifestType::Model);
        assert_eq!(parse_manifest_type(""), ManifestType::Model);
    }

    // Helper function to create a test manifest
    fn create_test_manifest() -> Manifest {
        use atlas_c2pa_lib::claim::ClaimV2;
        use atlas_c2pa_lib::datetime_wrapper::OffsetDateTimeWrapper;
        use atlas_c2pa_lib::ingredient::{Ingredient, IngredientData};
        use time::OffsetDateTime;
        use uuid::Uuid;

        let ingredient = Ingredient {
            title: "Test Ingredient".to_string(),
            format: "application/octet-stream".to_string(),
            relationship: "componentOf".to_string(),
            document_id: format!("uuid:{}", Uuid::new_v4()),
            instance_id: format!("uuid:{}", Uuid::new_v4()),
            data: IngredientData {
                url: "file:///test/path.bin".to_string(),
                alg: "sha256".to_string(),
                hash: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                    .to_string(),
                data_types: vec![AssetType::Model],
                linked_ingredient_url: None,
                linked_ingredient_hash: None,
            },
            linked_ingredient: None,
            public_key: None,
        };

        let claim = ClaimV2 {
            instance_id: format!("urn:uuid:{}", Uuid::new_v4()),
            claim_generator_info: "test".to_string(),
            created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
            ingredients: vec![ingredient.clone()],
            created_assertions: vec![Assertion::CreativeWork(CreativeWorkAssertion {
                context: "http://schema.org/".to_string(),
                creative_type: "Model".to_string(),
                author: vec![Author {
                    author_type: "Organization".to_string(),
                    name: "Test Organization".to_string(),
                }],
            })],
            signature: None,
        };

        Manifest {
            claim_generator: "test".to_string(),
            title: "Test Manifest".to_string(),
            instance_id: format!("urn:uuid:{}", Uuid::new_v4()),
            ingredients: vec![ingredient],
            claim: claim.clone(),
            created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
            cross_references: vec![],
            claim_v2: Some(claim),
            is_active: true,
        }
    }
}
