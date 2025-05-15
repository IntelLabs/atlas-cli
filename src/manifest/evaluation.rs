use crate::error::{Error, Result};
use crate::manifest::common::{self, list_manifests, verify_manifest, AssetKind};
use crate::manifest::config::ManifestCreationConfig;
use crate::manifest::{determine_manifest_type, manifest_type_to_str};
use crate::storage::traits::StorageBackend;
use atlas_c2pa_lib::assertion::Assertion;
use std::collections::HashMap;

/// Create a new evaluation result manifest using the standard configuration
pub fn create_manifest(
    mut config: ManifestCreationConfig,
    model_id: String,
    dataset_id: String,
    metrics: Vec<String>,
) -> Result<()> {
    // Parse metrics into a map
    let mut metrics_map = HashMap::new();
    for metric in metrics {
        let parts: Vec<&str> = metric.split('=').collect();
        if parts.len() == 2 {
            metrics_map.insert(parts[0].to_string(), parts[1].to_string());
        } else {
            return Err(Error::Validation(format!(
                "Invalid metric format: {}. Expected format: key=value",
                metric
            )));
        }
    }

    // Add evaluation-specific custom_fields to the config
    let eval_params = serde_json::json!({
        "model_id": model_id,
        "dataset_id": dataset_id,
        "metrics": metrics_map,
    });

    // Update the description to include evaluation info
    let enhanced_description = match &config.description {
        Some(desc) => Some(format!(
            "{} (Model: {}, Dataset: {})",
            desc, model_id, dataset_id
        )),
        None => Some(format!(
            "Evaluation of Model: {} on Dataset: {}",
            model_id, dataset_id
        )),
    };
    config.description = enhanced_description;

    // Ensure linked_manifests includes model and dataset IDs
    let mut linked_manifests = config.linked_manifests.unwrap_or_default();
    if !linked_manifests.contains(&model_id) {
        linked_manifests.push(model_id.clone());
    }
    if !linked_manifests.contains(&dataset_id) {
        linked_manifests.push(dataset_id.clone());
    }
    config.linked_manifests = Some(linked_manifests);

    // Store evaluation parameters in the config's custom_fields
    config.custom_fields = Some(serde_json::json!({
        "evaluation": eval_params
    }));

    // Call the common implementation with AssetKind::Evaluation
    common::create_manifest(config, AssetKind::Evaluation)
}

/// List evaluation manifests from storage
pub fn list_evaluation_manifests(storage: &dyn StorageBackend) -> Result<()> {
    list_manifests(storage, Some(AssetKind::Evaluation))
}

/// Verify an evaluation manifest
pub fn verify_evaluation_manifest(id: &str, storage: &dyn StorageBackend) -> Result<()> {
    // Use the common verification function first
    verify_manifest(id, storage)?;

    // Additional verification specific to evaluation manifests
    let manifest = storage.retrieve_manifest(id)?;

    // Verify that it's an evaluation manifest
    if !is_evaluation_manifest(&manifest) {
        return Err(Error::Validation("Not an evaluation manifest".to_string()));
    }

    // Verify cross-references to model and dataset
    let mut found_model = false;
    let mut found_dataset = false;

    for cross_ref in &manifest.cross_references {
        match storage.retrieve_manifest(&cross_ref.manifest_url) {
            Ok(ref_manifest) => {
                if manifest_type_to_str(&determine_manifest_type(&ref_manifest)) == "Model" {
                    found_model = true;
                }
                if manifest_type_to_str(&determine_manifest_type(&ref_manifest)) == "Dataset" {
                    found_dataset = true;
                }
            }
            Err(e) => {
                return Err(Error::Validation(format!(
                    "Failed to retrieve referenced manifest {}: {}",
                    cross_ref.manifest_url, e
                )));
            }
        }
    }

    if !found_model {
        return Err(Error::Validation(
            "Evaluation manifest must reference a model".to_string(),
        ));
    }

    if !found_dataset {
        return Err(Error::Validation(
            "Evaluation manifest must reference a dataset".to_string(),
        ));
    }

    println!("✓ Evaluation manifest verification successful");
    Ok(())
}

/// Check if a manifest is an evaluation result manifest
fn is_evaluation_manifest(manifest: &atlas_c2pa_lib::manifest::Manifest) -> bool {
    if let Some(claim) = &manifest.claim_v2 {
        claim.created_assertions.iter().any(|assertion| {
            matches!(assertion, Assertion::CreativeWork(creative_work) if creative_work.creative_type == "EvaluationResult")
        })
    } else {
        false
    }
}
