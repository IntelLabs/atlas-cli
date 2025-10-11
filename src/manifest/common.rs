use crate::cc_attestation;
use crate::error::{Error, Result};
use crate::hash;
use crate::in_toto;
use crate::manifest::config::ManifestCreationConfig;
use crate::manifest::utils::{
    determine_dataset_type, determine_format, determine_model_type, determine_software_type,
};
use crate::signing::signable::Signable;
use crate::storage::traits::{ArtifactLocation, StorageBackend};
use atlas_c2pa_lib::assertion::{
    Action, ActionAssertion, Assertion, Author, CreativeWorkAssertion, CustomAssertion,
};
use atlas_c2pa_lib::asset_type::AssetType;
use atlas_c2pa_lib::claim::ClaimV2;
use atlas_c2pa_lib::cose::HashAlgorithm;
use atlas_c2pa_lib::cross_reference::CrossReference;
use atlas_c2pa_lib::datetime_wrapper::OffsetDateTimeWrapper;
use atlas_c2pa_lib::ingredient::{Ingredient, IngredientData};
use atlas_c2pa_lib::manifest::Manifest;
use serde_json::{to_string, to_string_pretty};
use std::path::{Path, PathBuf};
use tdx_workload_attestation::get_platform_name;
use time::OffsetDateTime;
use uuid::Uuid;

const CLAIM_GENERATOR: &str = "atlas-cli:0.1.1";

/// Asset type enum to distinguish between models, datasets, software, and evaluations
pub enum AssetKind {
    Model,
    Dataset,
    Software,
    Evaluation,
}

/// Generates C2PA Assertions based on the asset kind and configuration.
///
/// This function creates standardized C2PA assertions including creative work and action assertions
/// that are tailored to the specific type of asset being attested (model, dataset, software, or evaluation).
/// It also optionally includes confidential computing (CC) attestations when enabled.
fn generate_c2pa_assertions(
    config: &ManifestCreationConfig,
    asset_kind: AssetKind,
) -> Result<Vec<Assertion>> {
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
                software_agent: Some(CLAIM_GENERATOR.to_string()),
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
                    // don't need to repeat info for created action assertions that's
                    // already in the CreativeWork assertion
                    _ => serde_json::json!({}),
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

    Ok(assertions)
}

/// Generates a C2PA claim with ingredients and assertions.
///
/// This function creates a complete C2PA claim by generating ingredients from the provided file paths,
/// sorting them alphabetically (as required by the OpenSSF Model Signing specification), and combining
/// them with generated assertions. The claim includes metadata such as instance ID, creation timestamp,
/// and claim generator information.
fn generate_c2pa_claim(config: &ManifestCreationConfig, asset_kind: AssetKind) -> Result<ClaimV2> {
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
        let ingredient = create_ingredient_from_path_with_algorithm(
            path,
            ingredient_name,
            asset_type,
            format,
            &config.hash_alg,
        )?;
        ingredients.push(ingredient);
    }

    // Per the OMS spec, ingredients must be hashed in alphabetical order of the
    // artifact name, so always canonicalize the order regardless of format
    // because the manifest must provide references to all artifacts needed to
    // recompute the model hash.
    // See https://github.com/sigstore/model-transparency/blob/de2f935ad437218d577a3f39378c482bf3aafcec/src/model_signing/_signing/signing.py#L188-L192
    ingredients.sort_by_key(|ingredient| ingredient.title.to_lowercase());

    let assertions = generate_c2pa_assertions(config, asset_kind)?;

    // Create claim
    Ok(ClaimV2 {
        instance_id: format!("urn:c2pa:{}", Uuid::new_v4()),
        ingredients: ingredients.clone(),
        created_assertions: assertions,
        claim_generator_info: CLAIM_GENERATOR.to_string(),
        signature: None,
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
    })
}

/// Creates a manifest for a model, dataset, software, or evaluation
pub fn create_manifest(config: ManifestCreationConfig, asset_kind: AssetKind) -> Result<()> {
    let claim = generate_c2pa_claim(&config, asset_kind)?;

    // Create the manifest
    let mut manifest = Manifest {
        claim_generator: CLAIM_GENERATOR.to_string(),
        title: config.name.clone(),
        instance_id: format!("urn:c2pa:{}", Uuid::new_v4()),
        claim: claim.clone(),
        ingredients: vec![],
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
        cross_references: vec![],
        claim_v2: Some(claim),
        is_active: true,
    };

    // Sign if key is provided
    if let Some(key_file) = &config.key_path {
        manifest.sign(key_file.to_path_buf(), config.hash_alg)?;
    }

    if let Some(manifest_ids) = &config.linked_manifests {
        if let Some(storage_backend) = &config.storage {
            for linked_id in manifest_ids {
                match storage_backend.retrieve_manifest(linked_id) {
                    Ok(linked_manifest) => {
                        // Create a JSON representation of the linked manifest
                        let linked_json = serde_json::to_string(&linked_manifest)
                            .map_err(|e| Error::Serialization(e.to_string()))?;

                        // Create a hash of the linked manifest
                        let linked_hash = hash::calculate_hash(linked_json.as_bytes());

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
        match config.output_encoding.to_lowercase().as_str() {
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
                    "Invalid output encoding '{}'. Valid options are: json, cbor",
                    config.output_encoding
                )));
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

/// Creates an OpenSSF Model Signing (OMS) compliant C2PA manifest for a model.
///
/// This function generates a manifest that conforms to the OpenSSF Model Signing specification,
/// creating an in-toto format Statement with a DSSE (Dead Simple Signing Envelope). The manifest
/// is specifically designed for model artifacts and includes proper subject hash calculation
/// according to OMS requirements.
///
/// # Arguments
///
/// * `config` - The manifest creation configuration, must include a signing key for OMS format
///
/// # Returns
///
/// `Ok(())` on successful manifest creation, or an error if creation fails.
///
/// # Errors
///
/// Returns an error if:
/// - No signing key is provided (OMS format requires signing)
/// - Subject hash calculation fails
/// - Manifest serialization fails
/// - Storage operations fail
///
/// # Examples
///
/// ```no_run
/// use atlas_cli::manifest::config::ManifestCreationConfig;
/// use atlas_cli::manifest::common::create_oms_manifest;
/// use atlas_c2pa_lib::cose::HashAlgorithm;
/// use std::path::PathBuf;
///
/// let config = ManifestCreationConfig {
///     name: "test-model".to_string(),
///     description: Some("A test model".to_string()),
///     author_name: Some("Test Author".to_string()),
///     author_org: Some("Test Org".to_string()),
///     paths: vec![PathBuf::from("model.onnx")],
///     ingredient_names: vec!["model".to_string()],
///     hash_alg: HashAlgorithm::Sha384,
///     key_path: Some(PathBuf::from("private_key.pem")),
///     output_encoding: "json".to_string(),
///     print: true,
///     storage: None,
///     with_cc: false,
///     linked_manifests: None,
///     custom_fields: None,
///     software_type: None,
///     version: None,
/// };
///
/// create_oms_manifest(config).unwrap();
/// ```
pub fn create_oms_manifest(config: ManifestCreationConfig) -> Result<()> {
    let claim = generate_c2pa_claim(&config, AssetKind::Model)?;

    // Create the manifest
    let mut manifest = Manifest {
        claim_generator: "".to_string(),
        title: "".to_string(),
        instance_id: format!("urn:c2pa:{}", Uuid::new_v4()),
        claim: claim.clone(),
        ingredients: vec![],
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
        cross_references: vec![],
        claim_v2: None,
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
                        let linked_hash = hash::calculate_hash(linked_json.as_bytes());

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

    // Generate the in-toto format Statement and sign the DSSE

    // we need to convert this into a string to serialize into the Struct proto expected by in-toto
    let manifest_json = to_string(&manifest).map_err(|e| Error::Serialization(e.to_string()))?;
    let manifest_proto = in_toto::json_to_struct_proto(&manifest_json)?;

    let subject_hash = generate_oms_subject_hash(&manifest, &config.hash_alg)?;

    let subject = in_toto::make_minimal_resource_descriptor(
        &config.name,
        hash::algorithm_to_string(&config.hash_alg),
        &subject_hash,
    );

    let key_path = config
        .key_path
        .ok_or_else(|| Error::Validation("OMS format requires a signing key".to_string()))?;

    let envelope = in_toto::generate_signed_statement_v1(
        &[subject],
        "https://spec.c2pa.org/specifications/specifications/2.2",
        &manifest_proto,
        key_path.to_path_buf(),
        config.hash_alg,
    )?;

    // Output manifest if requested
    if config.print || config.storage.is_none() {
        match config.output_encoding.to_lowercase().as_str() {
            "json" => {
                let envelope_json =
                    to_string_pretty(&envelope).map_err(|e| Error::Serialization(e.to_string()))?;
                println!("{envelope_json}");
            }
            "cbor" => {
                let envelope_cbor = serde_cbor::to_vec(&envelope)
                    .map_err(|e| Error::Serialization(e.to_string()))?;
                println!("{}", hex::encode(&envelope_cbor));
            }
            _ => {
                return Err(Error::Validation(format!(
                    "Invalid output encoding '{}'. Valid options are: json, cbor",
                    config.output_encoding
                )));
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

/// Lists manifests from storage, optionally filtered by asset type.
///
/// This function retrieves all manifests from the provided storage backend and optionally
/// filters them by asset kind (Model, Dataset, Software, or Evaluation). The filtered
/// manifests are then displayed with their metadata including ID, name, type, and creation time.
///
/// # Arguments
///
/// * `storage` - The storage backend to retrieve manifests from
/// * `asset_kind` - Optional filter for asset type; if None, all manifests are listed
///
/// # Returns
///
/// `Ok(())` on successful listing, or an error if storage retrieval fails.
///
/// # Examples
///
/// ```no_run
/// use atlas_cli::manifest::common::{AssetKind, list_manifests};
/// use atlas_cli::storage::traits::StorageBackend;
/// use atlas_cli::storage::filesystem::FilesystemStorage;
///
/// // Create or obtain a storage backend instance
/// let storage_backend: FilesystemStorage = FilesystemStorage::new("/path/to/storage").unwrap();
///
/// // List all manifests
/// list_manifests(&storage_backend, None).unwrap();
///
/// // List only model manifests
/// list_manifests(&storage_backend, Some(AssetKind::Model)).unwrap();
/// ```
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

/// Performs comprehensive verification of a manifest.
///
/// This function verifies a manifest by performing multiple validation steps:
/// 1. Validates the manifest structure against C2PA specifications
/// 2. Verifies hash integrity of all ingredients (file-based and URL-based)
/// 3. Validates cross-references to linked manifests
/// 4. Checks asset-specific requirements based on manifest type
///
/// The verification process ensures that the manifest is structurally valid and that
/// all referenced artifacts maintain their integrity since the manifest was created.
///
/// # Arguments
///
/// * `id` - The unique identifier of the manifest to verify
/// * `storage` - The storage backend to retrieve the manifest and linked manifests
///
/// # Returns
///
/// `Ok(())` if verification succeeds, or an error describing the verification failure.
///
/// # Errors
///
/// Returns an error if:
/// - Manifest cannot be retrieved from storage
/// - Manifest structure is invalid
/// - Any ingredient hash verification fails
/// - Cross-reference verification fails
/// - Asset-specific requirements are not met
///
/// # Examples
///
/// ```no_run
/// use atlas_cli::manifest::common::verify_manifest;
/// use atlas_cli::storage::traits::StorageBackend;
/// use atlas_cli::storage::filesystem::FilesystemStorage;
///
/// // Create or obtain a storage backend instance
/// let storage_backend: FilesystemStorage = FilesystemStorage::new("/path/to/storage").unwrap();
///
/// let manifest_id = "manifest-123";
/// verify_manifest(manifest_id, &storage_backend).unwrap();
/// println!("✓ Manifest verification successful");
/// ```
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
            match hash::calculate_file_hash(PathBuf::from(&ingredient.data.url)) {
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
            let algorithm = hash::detect_hash_algorithm(&cross_ref.manifest_hash);
            let calculated_hash =
                hash::calculate_hash_with_algorithm(manifest_json.as_bytes(), &algorithm);

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
                println!(
                    "WARNING: Software manifest doesn't contain a Software creative work assertion or software_type parameter"
                );

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
                println!(
                    "WARNING: Evaluation manifest doesn't contain an EvaluationResult creative work assertion"
                );

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

/// Create a C2PA Ingredient from a path
pub fn create_ingredient_from_path(
    path: &Path,
    name: &str,
    asset_type: AssetType,
    format: String,
) -> Result<Ingredient> {
    create_ingredient_from_path_with_algorithm(
        path,
        name,
        asset_type,
        format,
        &HashAlgorithm::Sha384,
    )
}

/// Create a C2PA Ingredient from a path with a specified hash algorithm
pub fn create_ingredient_from_path_with_algorithm(
    path: &Path,
    name: &str,
    asset_type: AssetType,
    format: String,
    algorithm: &HashAlgorithm,
) -> Result<Ingredient> {
    let ingredient_data = IngredientData {
        url: format!("file://{}", path.to_string_lossy()),
        alg: algorithm.as_str().to_string(),
        hash: hash::calculate_file_hash_with_algorithm(path, algorithm)?,
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

/// Helper function to generate a CC attestation assertion
fn get_cc_attestation_assertion() -> Result<CustomAssertion> {
    let report = match cc_attestation::get_report(false) {
        Ok(r) => r,
        Err(e) => {
            return Err(Error::CCAttestationError(format!(
                "Failed to get attestation: {e}"
            )));
        }
    };

    // detect the underlying CC platform
    let platform = match get_platform_name() {
        Ok(p) => p,
        Err(e) => {
            return Err(Error::CCAttestationError(format!(
                "Error detecting attestation platform: {e}"
            )));
        }
    };

    let cc_assertion = CustomAssertion {
        label: platform,
        data: serde_json::Value::String(report),
    };

    Ok(cc_assertion)
}

// Compute the OMS subject hash as specified in https://github.com/sigstore/model-transparency/blob/de2f935ad437218d577a3f39378c482bf3aafcec/src/model_signing/_signing/signing.py#L181-L186
fn generate_oms_subject_hash(manifest: &Manifest, hash_alg: &HashAlgorithm) -> Result<String> {
    // generate the hash over all ingredient hashes for the model
    if manifest.claim.ingredients.is_empty() {
        return Err(Error::Validation(
            "OMS requires at least one ingredient".to_string(),
        ));
    }

    // Per the OMS spec, the ingredients must be hashed in a canonical order
    // (alphabetical order of artifact name)
    // Since we cannot assume that the ingredients in the manifest are sorted
    // as expected (e.g., during verification), we sort every time we hash.
    let mut ingredients_to_hash = manifest.claim.ingredients.clone();
    ingredients_to_hash.sort_by_key(|ingredient| ingredient.title.to_lowercase());

    let mut ingredient_hashes: Vec<u8> = Vec::new();
    for ingredient in &ingredients_to_hash {
        let raw_bytes = hex::decode(&ingredient.data.hash).map_err(|e| {
            Error::Validation(format!(
                "Invalid hash for ingredient {}: {}",
                ingredient.title, e
            ))
        })?;
        ingredient_hashes.extend_from_slice(&raw_bytes);
    }

    Ok(hash::calculate_hash_with_algorithm(
        &ingredient_hashes,
        hash_alg,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signing::test_utils::generate_temp_key;

    fn make_test_manifest_config() -> ManifestCreationConfig {
        let (_secure_key, tmp_dir) = generate_temp_key().unwrap();

        ManifestCreationConfig {
            name: "test-model".to_string(),
            description: Some("A test model".to_string()),
            author_name: Some("Test Author".to_string()),
            author_org: Some("Test Org".to_string()),
            paths: vec![],
            ingredient_names: vec![],
            hash_alg: HashAlgorithm::Sha384,
            key_path: Some(tmp_dir.path().join("test_key.pem")),
            output_encoding: "json".to_string(),
            print: false,
            storage: None,
            with_cc: false,
            linked_manifests: None,
            custom_fields: None,
            software_type: None,
            version: None,
        }
    }

    #[test]
    fn test_generate_c2pa_assertions() {
        let config = make_test_manifest_config();

        let assertions = generate_c2pa_assertions(&config, AssetKind::Model).unwrap();
        assert!(!assertions.is_empty()); // Should have at least the CreativeWork assertion
    }

    #[test]
    fn test_generate_c2pa_claim() {
        let config = make_test_manifest_config();
        let claim = generate_c2pa_claim(&config, AssetKind::Model).unwrap();
        assert!(claim.instance_id.starts_with("urn:c2pa:"));
        assert_eq!(claim.claim_generator_info, "atlas-cli:0.1.1");
    }

    // #[test]
    // fn test_create_manifest() -> Result<()>{
    //     let config = make_test_manifest_config();
    //     let result = create_manifest(config, AssetKind::Model);
    //     assert!(result.is_ok()); // Should succeed even with no ingredients

    //     Ok(())
    // }

    // #[test]
    // fn test_create_oms_manifest() -> Result<()> {
    //     let config = make_test_manifest_config();
    //     let result = create_oms_manifest(config);
    //     assert!(result.is_ok()); // Should succeed with the provided key

    //     Ok(())
    // }

    #[test]
    fn test_create_oms_manifest_no_key() {
        let mut config = make_test_manifest_config();
        config.key_path = None; // Remove the key path to simulate missing key
        let result = create_oms_manifest(config);
        assert!(result.is_err()); // Should fail because OMS requires a signing key
    }
}
