use crate::error::{Error, Result};
use crate::storage::traits::ManifestType;
use c2pa_ml::assertion::Assertion;
use c2pa_ml::asset_type::AssetType;
use c2pa_ml::manifest::Manifest;
use std::path::Path;

pub fn determine_model_type(path: &Path) -> Result<AssetType> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("onnx") => Ok(AssetType::ModelOnnx),
        Some("pb") => Ok(AssetType::ModelTensorFlow),
        Some("pt") | Some("pth") => Ok(AssetType::ModelPytorch),
        Some("h5") => Ok(AssetType::ModelKeras),
        _ => Err(Error::Validation("Unsupported model format".to_string())),
    }
}

pub fn determine_format(path: &Path) -> Result<String> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("onnx") => Ok("application/onnx".to_string()),
        Some("pb") => Ok("application/x-protobuf".to_string()),
        Some("pt") | Some("pth") => Ok("application/x-pytorch".to_string()),
        Some("h5") => Ok("application/x-hdf5".to_string()),
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

pub fn determine_dataset_type(_path: &Path) -> Result<AssetType> {
    // Always return Dataset type for now
    Ok(AssetType::Dataset)
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
