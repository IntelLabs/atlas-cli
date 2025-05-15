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
    // Check if any ingredients have dataset type
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

    // Check for dataset assertion
    let has_dataset_assertion = if let Some(claim) = &manifest.claim_v2 {
        claim.created_assertions.iter().any(|assertion| {
            matches!(assertion, Assertion::CreativeWork(creative_work) if creative_work.creative_type == "Dataset")
        })
    } else {
        manifest.claim.created_assertions.iter().any(|assertion| {
            matches!(assertion, Assertion::CreativeWork(creative_work) if creative_work.creative_type == "Dataset")
        })
    };

    // Check if any ingredients have model type
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

    // Check for model assertion
    let has_model_assertion = if let Some(claim) = &manifest.claim_v2 {
        claim.created_assertions.iter().any(|assertion| {
            matches!(assertion, Assertion::CreativeWork(creative_work) if creative_work.creative_type == "Model")
        })
    } else {
        manifest.claim.created_assertions.iter().any(|assertion| {
            matches!(assertion, Assertion::CreativeWork(creative_work) if creative_work.creative_type == "Model")
        })
    };

    // Check for software assertion or parameters
    let has_software_assertion = if let Some(claim) = &manifest.claim_v2 {
        claim.created_assertions.iter().any(|assertion| {
            matches!(assertion, Assertion::CreativeWork(creative_work) if creative_work.creative_type == "Software")
        })
    } else {
        manifest.claim.created_assertions.iter().any(|assertion| {
            matches!(assertion, Assertion::CreativeWork(creative_work) if creative_work.creative_type == "Software")
        })
    };

    let has_software_parameters = if let Some(claim) = &manifest.claim_v2 {
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
    };

    // Check if any ingredients have software type
    let has_software_ingredients = manifest.ingredients.iter().any(|ingredient| {
        ingredient
            .data
            .data_types
            .iter()
            .any(|t| matches!(t, AssetType::Generator))
    });

    // Determine type based on checks
    if has_dataset_ingredients || has_dataset_assertion {
        ManifestType::Dataset
    } else if has_model_ingredients || has_model_assertion {
        ManifestType::Model
    } else if has_software_ingredients || has_software_assertion || has_software_parameters {
        ManifestType::Software
    } else {
        // Default to Model if we can't determine
        ManifestType::Model
    }
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
