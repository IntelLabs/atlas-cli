use crate::cc_attestation;
use crate::error::{Error, Result};
use crate::hash::utils::calculate_file_hash;
use crate::manifest::config::ManifestCreationConfig;
use crate::manifest::utils::{
    determine_dataset_type, determine_format, determine_model_type, determine_software_type,
};
use crate::signing;
use crate::storage::traits::{ArtifactLocation, StorageBackend};
use atlas_c2pa_lib::assertion::{
    Action, ActionAssertion, Assertion, Author, CreativeWorkAssertion, CustomAssertion,
};
use atlas_c2pa_lib::asset_type::AssetType;
use atlas_c2pa_lib::claim::ClaimV2;
use atlas_c2pa_lib::cross_reference::CrossReference;
use atlas_c2pa_lib::datetime_wrapper::OffsetDateTimeWrapper;
use atlas_c2pa_lib::ingredient::{Ingredient, IngredientData};
use atlas_c2pa_lib::manifest::Manifest;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde_json::to_string_pretty;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tdx_workload_attestation::get_platform_name;
use time::OffsetDateTime;
use uuid::Uuid;

/// Asset type enum to distinguish between models, datasets, software, and evaluations
pub enum AssetKind {
    Model,
    Dataset,
    Software,
    Evaluation,
}

/// Creates a manifest for a model, dataset, software, or evaluation
pub fn create_manifest(config: ManifestCreationConfig, asset_kind: AssetKind) -> Result<()> {
    // Create ingredients using the helper function
    let mut ingredients = Vec::new();
    for (path, ingredient_name) in config.paths.iter().zip(config.ingredient_names.iter()) {
        // Determine asset type and format based on asset kind
        let format = determine_format(path)?;
        let asset_type = match asset_kind {
            AssetKind::Model => determine_model_type(path)?,
            AssetKind::Dataset => determine_dataset_type(path)?,
            AssetKind::Software => determine_software_type(path)?,
            AssetKind::Evaluation => AssetType::Dataset, // Use Dataset type for evaluation results
        };

        // Use the helper function to create the ingredient
        let ingredient = create_ingredient_from_path(path, ingredient_name, asset_type, format)?;
        ingredients.push(ingredient);
    }

    // Determine asset-specific values
    let (creative_type, digital_source_type) = match asset_kind {
        AssetKind::Model => (
            "Model".to_string(),
            "http://cv.iptc.org/newscodes/digitalsourcetype/algorithmicMedia".to_string(),
        ),
        AssetKind::Dataset => (
            "Dataset".to_string(),
            "http://cv.iptc.org/newscodes/digitalsourcetype/dataset".to_string(),
        ),
        AssetKind::Software => (
            "Software".to_string(),
            "http://cv.iptc.org/newscodes/digitalsourcetype/software".to_string(),
        ),
        AssetKind::Evaluation => (
            "EvaluationResult".to_string(),
            "http://cv.iptc.org/newscodes/digitalsourcetype/evaluationResult".to_string(),
        ),
    };

    // Create assertions
    let mut assertions = vec![
        Assertion::CreativeWork(CreativeWorkAssertion {
            context: "http://schema.org/".to_string(),
            creative_type,
            author: vec![
                Author {
                    author_type: "Organization".to_string(),
                    name: config
                        .author_org
                        .clone()
                        .unwrap_or_else(|| "Organization".to_string()),
                },
                Author {
                    author_type: "Person".to_string(),
                    name: config
                        .author_name
                        .clone()
                        .unwrap_or_else(|| "Unknown".to_string()),
                },
            ],
        }),
        Assertion::Action(ActionAssertion {
            actions: vec![Action {
                action: match asset_kind {
                    AssetKind::Evaluation => "c2pa.evaluation".to_string(),
                    _ => "c2pa.created".to_string(),
                },
                software_agent: Some("c2pa-cli".to_string()),
                parameters: Some(match asset_kind {
                    AssetKind::Evaluation => {
                        // Merge evaluation parameters with standard parameters
                        let mut params = serde_json::json!({
                            "name": config.name,
                            "description": config.description,
                            "author": {
                                "organization": config.author_org,
                                "name": config.author_name
                            }
                        });

                        // Add evaluation-specific parameters if present
                        if let Some(config_params) = &config.custom_fields {
                            if let Some(eval_params) = config_params.get("evaluation") {
                                if let Some(obj) = params.as_object_mut() {
                                    obj.insert(
                                        "model_id".to_string(),
                                        eval_params
                                            .get("model_id")
                                            .cloned()
                                            .unwrap_or(serde_json::Value::Null),
                                    );
                                    obj.insert(
                                        "dataset_id".to_string(),
                                        eval_params
                                            .get("dataset_id")
                                            .cloned()
                                            .unwrap_or(serde_json::Value::Null),
                                    );
                                    obj.insert(
                                        "metrics".to_string(),
                                        eval_params
                                            .get("metrics")
                                            .cloned()
                                            .unwrap_or(serde_json::Value::Null),
                                    );
                                }
                            }
                        }
                        params
                    }
                    AssetKind::Software => {
                        let mut params = serde_json::json!({
                            "name": config.name,
                            "description": config.description,
                            "author": {
                                "organization": config.author_org,
                                "name": config.author_name
                            }
                        });

                        if let Some(software_type) = &config.software_type {
                            params.as_object_mut().unwrap().insert(
                                "software_type".to_string(),
                                serde_json::Value::String(software_type.clone()),
                            );
                        }
                        if let Some(version) = &config.version {
                            params.as_object_mut().unwrap().insert(
                                "version".to_string(),
                                serde_json::Value::String(version.clone()),
                            );
                        }
                        params
                    }
                    _ => serde_json::json!({
                        "name": config.name,
                        "description": config.description,
                        "author": {
                            "organization": config.author_org,
                            "name": config.author_name
                        }
                    }),
                }),
                digital_source_type: Some(digital_source_type),
                instance_id: None,
            }],
        }),
    ];

    // if we're creating the manifest in a CC environment, create
    // an assertion for the CC attestation
    if config.with_cc {
        // the assertion contents will depend on the detected platform
        let cc_assertion = get_cc_attestation_assertion().unwrap();

        assertions.push(Assertion::CustomAssertion(cc_assertion));
    }

    // Create claim
    let mut claim = ClaimV2 {
        instance_id: format!("urn:c2pa:{}", Uuid::new_v4()),
        ingredients: ingredients.clone(),
        created_assertions: assertions,
        claim_generator_info: "c2pa-cli".to_string(),
        signature: None,
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
    };

    // Sign if key is provided
    if let Some(key_file) = &config.key_path {
        let private_key = signing::load_private_key(key_file)?;

        // Serialize claim to CBOR for signing
        let claim_cbor =
            serde_cbor::to_vec(&claim).map_err(|e| Error::Serialization(e.to_string()))?;

        // Use the signing module with the specified algorithm
        let signature =
            signing::sign_data_with_algorithm(&claim_cbor, &private_key, &config.hash_alg)?;

        // Add signature to claim
        claim.signature = Some(STANDARD.encode(&signature));
    }

    // Create the manifest
    let mut manifest = Manifest {
        claim_generator: "c2pa-cli/0.1.0".to_string(),
        title: config.name.clone(),
        instance_id: format!("urn:c2pa:{}", Uuid::new_v4()),
        ingredients,
        claim: claim.clone(),
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
        cross_references: vec![],
        claim_v2: Some(claim),
        is_active: true,
    };

    if let Some(manifest_ids) = &config.linked_manifests {
        if let Some(storage_backend) = &config.storage {
            for linked_id in manifest_ids {
                match storage_backend.retrieve_manifest(linked_id) {
                    Ok(linked_manifest) => {
                        // Create a JSON representation of the linked manifest
                        let linked_json = serde_json::to_string(&linked_manifest)
                            .map_err(|e| Error::Serialization(e.to_string()))?;

                        // Create a hash of the linked manifest
                        let linked_hash = hex::encode(Sha256::digest(linked_json.as_bytes()));

                        // Create a cross-reference
                        let cross_ref = CrossReference {
                            manifest_url: linked_id.clone(),
                            manifest_hash: linked_hash,
                            media_type: Some("application/json".to_string()),
                        };

                        // Add the cross-reference to the manifest
                        manifest.cross_references.push(cross_ref);

                        println!("Added link to manifest: {linked_id}");
                    }
                    Err(e) => {
                        println!("Warning: Could not link to manifest {linked_id}: {e}");
                    }
                }
            }
        } else {
            println!("Warning: Cannot link manifests without a storage backend");
        }
    }

    // Output manifest if requested
    if config.print || config.storage.is_none() {
        match config.output_format.to_lowercase().as_str() {
            "json" => {
                let manifest_json =
                    to_string_pretty(&manifest).map_err(|e| Error::Serialization(e.to_string()))?;
                println!("{manifest_json}");
            }
            "cbor" => {
                let manifest_cbor = serde_cbor::to_vec(&manifest)
                    .map_err(|e| Error::Serialization(e.to_string()))?;
                println!("{}", hex::encode(&manifest_cbor));
            }
            _ => {
                return Err(Error::Validation(format!(
                    "Invalid output format '{}'. Valid options are: json, cbor",
                    config.output_format
                )))
            }
        }
    }

    // Store manifest if storage is provided
    if let Some(storage) = &config.storage {
        if !config.print {
            let id = storage.store_manifest(&manifest)?;
            println!("Manifest stored successfully with ID: {id}");
        }
    }

    Ok(())
}

pub fn list_manifests(storage: &dyn StorageBackend, asset_kind: Option<AssetKind>) -> Result<()> {
    let manifests = storage.list_manifests()?;

    // Filter manifests by type if asset_kind is specified
    let filtered_manifests = if let Some(kind) = asset_kind {
        manifests
            .into_iter()
            .filter(|m| match kind {
                AssetKind::Model => {
                    matches!(m.manifest_type, crate::storage::traits::ManifestType::Model)
                }
                AssetKind::Dataset => matches!(
                    m.manifest_type,
                    crate::storage::traits::ManifestType::Dataset
                ),
                AssetKind::Software => matches!(
                    m.manifest_type,
                    crate::storage::traits::ManifestType::Software
                ),
                AssetKind::Evaluation => {
                    // Check if manifest title or name contains "Evaluation"
                    m.name.contains("Evaluation") || m.name.contains("evaluation")
                }
            })
            .collect::<Vec<_>>()
    } else {
        manifests
    };

    // Display the manifests
    for metadata in filtered_manifests {
        println!(
            "Manifest: {} (ID: {}, Type: {:?}, Created: {})",
            metadata.name, metadata.id, metadata.manifest_type, metadata.created_at
        );
    }

    Ok(())
}

/// Verify a manifest
pub fn verify_manifest(id: &str, storage: &dyn StorageBackend) -> Result<()> {
    let manifest = storage.retrieve_manifest(id)?;

    // Step 1: Verify the manifest structure
    atlas_c2pa_lib::manifest::validate_manifest(&manifest)
        .map_err(|e| crate::error::Error::Validation(e.to_string()))?;

    println!("Verifying manifest with ID: {id}");

    // Step 2: Verify each ingredient's hash
    for ingredient in &manifest.ingredients {
        println!("Verifying ingredient: {}", ingredient.title);

        if ingredient.data.url.starts_with("file://") {
            let path = PathBuf::from(ingredient.data.url.trim_start_matches("file://"));

            // Create ArtifactLocation for verification
            let location = ArtifactLocation {
                url: ingredient.data.url.clone(),
                file_path: Some(path),
                hash: ingredient.data.hash.clone(),
            };

            // Verify the hash and handle the result
            match location.verify() {
                Ok(true) => {
                    println!(
                        "✓ Successfully verified hash for component: {}",
                        ingredient.title
                    );
                }
                Ok(false) => {
                    return Err(Error::Validation(format!(
                        "Hash verification failed for component: {}. The file may have been modified.",
                        ingredient.title
                    )));
                }
                Err(e) => {
                    return Err(Error::Validation(format!(
                        "Error verifying component {}: {}. The file may be missing or inaccessible.",
                        ingredient.title, e
                    )));
                }
            }
        } else {
            // For non-file URLs, try direct hash verification
            match calculate_file_hash(PathBuf::from(&ingredient.data.url)) {
                Ok(calculated_hash) => {
                    if calculated_hash != ingredient.data.hash {
                        return Err(Error::Validation(format!(
                            "Hash mismatch for ingredient: {}",
                            ingredient.title
                        )));
                    }
                    println!(
                        "✓ Successfully verified hash for component: {}",
                        ingredient.title
                    );
                }
                Err(_) => {
                    println!(
                        "⚠ Warning: Component {} does not use file:// URL scheme and could not be verified directly",
                        ingredient.title
                    );
                }
            }
        }
    }

    // Step 3: Verify cross-references if present
    if !manifest.cross_references.is_empty() {
        println!("Verifying cross-references...");

        for cross_ref in &manifest.cross_references {
            let linked_manifest = storage.retrieve_manifest(&cross_ref.manifest_url)?;
            let manifest_json = serde_json::to_string(&linked_manifest)
                .map_err(|e| Error::Serialization(e.to_string()))?;
            let calculated_hash = hex::encode(Sha256::digest(manifest_json.as_bytes()));

            if calculated_hash != cross_ref.manifest_hash {
                return Err(Error::Validation(format!(
                    "Cross-reference verification failed for linked manifest: {}. Hash mismatch: stored={}, calculated={}",
                    cross_ref.manifest_url, cross_ref.manifest_hash, calculated_hash
                )));
            }
            println!(
                "✓ Verified cross-reference to manifest: {}",
                cross_ref.manifest_url
            );
        }
    }

    // Step 4: Verify asset-specific requirements
    verify_asset_specific_requirements(&manifest)?;

    println!("✓ Manifest verification successful");
    Ok(())
}

// Verify asset-specific requirements based on the manifest content
fn verify_asset_specific_requirements(manifest: &Manifest) -> Result<()> {
    // Determines the asset type from the manifest contents
    let is_dataset = is_dataset_manifest(manifest);
    let is_model = is_model_manifest(manifest);
    let is_software = is_software_manifest(manifest);
    let is_evaluation = is_evaluation_manifest(manifest);

    // Verify that at least one ingredient exists (except for evaluations)
    if !is_evaluation && manifest.ingredients.is_empty() {
        return Err(Error::Validation(
            "Manifest must contain at least one ingredient".to_string(),
        ));
    }

    // Check for dataset, model, software, or evaluation assertion
    if let Some(claim) = &manifest.claim_v2 {
        if is_dataset {
            let has_dataset_assertion = claim.created_assertions.iter().any(|assertion| {
                matches!(assertion, Assertion::CreativeWork(creative_work) if creative_work.creative_type == "Dataset")
            });

            let has_dataset_assertion_in_claim = if !has_dataset_assertion {
                manifest.claim.created_assertions.iter().any(|assertion| {
                    matches!(assertion, Assertion::CreativeWork(creative_work) if creative_work.creative_type == "Dataset")
                })
            } else {
                false
            };

            if !has_dataset_assertion && !has_dataset_assertion_in_claim {
                println!(
                    "WARNING: Dataset manifest doesn't contain a Dataset creative work assertion"
                );

                return Err(Error::Validation(
                    "Dataset manifest must contain a Dataset creative work assertion".to_string(),
                ));
            }
        }

        if is_model {
            let has_model_assertion = claim.created_assertions.iter().any(|assertion| {
                matches!(assertion, Assertion::CreativeWork(creative_work) if creative_work.creative_type == "Model")
            });

            let has_model_assertion_in_claim = if !has_model_assertion {
                manifest.claim.created_assertions.iter().any(|assertion| {
                    matches!(assertion, Assertion::CreativeWork(creative_work) if creative_work.creative_type == "Model")
                })
            } else {
                false
            };

            if !has_model_assertion && !has_model_assertion_in_claim {
                println!("WARNING: Model manifest doesn't contain a Model creative work assertion");

                return Err(Error::Validation(
                    "Model manifest must contain a Model creative work assertion".to_string(),
                ));
            }
        }

        if is_software {
            let has_software_assertion = claim.created_assertions.iter().any(|assertion| {
                matches!(assertion, Assertion::CreativeWork(creative_work) if creative_work.creative_type == "Software")
            });

            let has_software_parameters = claim.created_assertions.iter().any(|assertion| {
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

            if !has_software_assertion && !has_software_parameters {
                println!("WARNING: Software manifest doesn't contain a Software creative work assertion or software_type parameter");

                return Err(Error::Validation(
                    "Software manifest must contain a Software creative work assertion or software_type parameter".to_string(),
                ));
            }
        }

        if is_evaluation {
            let has_evaluation_assertion = claim.created_assertions.iter().any(|assertion| {
                matches!(assertion, Assertion::CreativeWork(creative_work) if creative_work.creative_type == "EvaluationResult")
            });

            if !has_evaluation_assertion {
                println!("WARNING: Evaluation manifest doesn't contain an EvaluationResult creative work assertion");

                return Err(Error::Validation(
                    "Evaluation manifest must contain an EvaluationResult creative work assertion"
                        .to_string(),
                ));
            }
        }
    }

    Ok(())
}

// Helper function to determine if a manifest is for a dataset
fn is_dataset_manifest(manifest: &Manifest) -> bool {
    // Check if it's an evaluation manifest - if so, it's NOT a dataset
    if is_evaluation_manifest(manifest) {
        return false;
    }

    // Now proceed with the regular dataset checking
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

    let has_dataset_assertion = if let Some(claim) = &manifest.claim_v2 {
        claim.created_assertions.iter().any(|assertion| {
            matches!(assertion, Assertion::CreativeWork(creative_work) if creative_work.creative_type == "Dataset")
        })
    } else {
        false
    };

    has_dataset_ingredients || has_dataset_assertion
}

// Helper function to determine if a manifest is for a model
fn is_model_manifest(manifest: &Manifest) -> bool {
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
    } else if let Some(Assertion::CreativeWork(creative_work)) = manifest
        .claim
        .created_assertions
        .iter()
        .find(|a| matches!(a, Assertion::CreativeWork(_)))
    {
        // Check in the old claim field as a fallback
        creative_work.creative_type == "Model"
    } else {
        false
    };

    // Returns true if either condition is met
    has_model_ingredients || has_model_assertion
}

// Helper function to check if a manifest is a software manifest
fn is_software_manifest(manifest: &Manifest) -> bool {
    // Check if any ingredients have software type
    let has_software_ingredients = manifest.ingredients.iter().any(|ingredient| {
        ingredient
            .data
            .data_types
            .iter()
            .any(|t| matches!(t, AssetType::Generator))
    });

    // Check for software assertion
    let has_software_assertion = if let Some(claim) = &manifest.claim_v2 {
        claim.created_assertions.iter().any(|assertion| {
            matches!(assertion, Assertion::CreativeWork(creative_work) if creative_work.creative_type == "Software")
        })
    } else {
        false
    };

    // Check for software parameters in actions
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

    has_software_ingredients || has_software_assertion || has_software_parameters
}

// Helper function to check if a manifest is an evaluation manifest
fn is_evaluation_manifest(manifest: &Manifest) -> bool {
    if let Some(claim) = &manifest.claim_v2 {
        claim.created_assertions.iter().any(|assertion| {
            matches!(assertion, Assertion::CreativeWork(creative_work) if creative_work.creative_type == "EvaluationResult")
        })
    } else {
        false
    }
}

/// Create an ingredient from a path
pub fn create_ingredient_from_path(
    path: &Path,
    name: &str,
    asset_type: AssetType,
    format: String,
) -> Result<Ingredient> {
    let ingredient_data = IngredientData {
        url: path.to_string_lossy().to_string(),
        alg: "sha256".to_string(),
        hash: crate::hash::calculate_file_hash(path)?,
        data_types: vec![asset_type],
        linked_ingredient_url: None,
        linked_ingredient_hash: None,
    };

    Ok(Ingredient {
        title: name.to_string(),
        format,
        relationship: "componentOf".to_string(),
        document_id: format!("uuid:{}", Uuid::new_v4()),
        instance_id: format!("uuid:{}", Uuid::new_v4()),
        data: ingredient_data,
        linked_ingredient: None,
        public_key: None,
    })
}

fn get_cc_attestation_assertion() -> Result<CustomAssertion> {
    let report = match cc_attestation::get_report(false) {
        Ok(r) => r,
        Err(e) => {
            return Err(Error::CCAttestationError(format!(
                "Failed to get attestation: {e}"
            )))
        }
    };

    // detect the underlying CC platform
    let platform = match get_platform_name() {
        Ok(p) => p,
        Err(e) => {
            return Err(Error::CCAttestationError(format!(
                "Error detecting attestation platform: {e}"
            )))
        }
    };

    let cc_assertion = CustomAssertion {
        label: platform,
        data: serde_json::Value::String(report),
    };

    Ok(cc_assertion)
}
