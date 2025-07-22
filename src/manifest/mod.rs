use crate::error::{Error, Result};
use crate::hash;
use crate::storage::traits::StorageBackend;
use atlas_c2pa_lib::cose::HashAlgorithm;
use atlas_c2pa_lib::cross_reference::CrossReference;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use uuid::Uuid;
pub mod common;
pub mod config;
pub mod dataset;
pub mod evaluation;
pub mod model;
pub mod software;
pub mod utils;
pub use dataset::create_manifest as create_dataset_manifest;
pub use dataset::list_dataset_manifests as list_dataset_manifest;
pub use dataset::verify_dataset_manifest;

pub use model::create_manifest as create_model_manifest;
pub use model::list_model_manifests as list_model_manifest;
pub use model::verify_model_manifest;

pub use software::create_manifest as create_software_manifest;
pub use software::list_software_manifests;
pub use software::verify_software_manifest;

pub use evaluation::create_manifest as create_evaluation_manifest;

pub use utils::{
    determine_manifest_type, manifest_type_to_str, manifest_type_to_string, parse_manifest_type,
};

/// Validate that a hash string is in the correct format.
/// Supported formats are SHA-256, SHA-384, or SHA-512.
///
/// # Examples
///
/// ```
/// use atlas_cli::manifest::validate_hash_format;
///
/// // Valid 96-character hex string
/// let valid_hash = "a".repeat(96);
/// assert!(validate_hash_format(&valid_hash).is_ok());
///
/// // Invalid: wrong length
/// let short_hash = "abc123";
/// assert!(validate_hash_format(&short_hash).is_err());
///
/// // Invalid: non-hex characters
/// let invalid_chars = "g".repeat(96);
/// assert!(validate_hash_format(&invalid_chars).is_err());
/// ```
pub fn validate_hash_format(hash: &str) -> Result<()> {
    if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(crate::error::Error::Validation(
            "Invalid hash format".to_string(),
        ));
    }
    // Check if the hash has the expected length for SHA-256, SHA-384 or SHA-512
    if !is_supported_c2pa_hash_length(hash.len()) {
        return Err(Error::Validation(format!(
            "Expected 64, 96 or 128 characters for SHA-256, SHA-384, or SHA-512 got {}",
            hash.len()
        )));
    }
    Ok(())
}

pub fn link_manifests(
    source_id: &str,
    target_id: &str,
    storage: &(impl StorageBackend + ?Sized),
) -> Result<()> {
    // Validate IDs format
    validate_manifest_id(source_id)?;
    validate_manifest_id(target_id)?;

    // Retrieve both manifests
    let mut source_manifest = match storage.retrieve_manifest(source_id) {
        Ok(manifest) => manifest,
        Err(e) => {
            return Err(Error::Manifest(format!(
                "Failed to retrieve source manifest {source_id}: {e}"
            )));
        }
    };

    let target_manifest = match storage.retrieve_manifest(target_id) {
        Ok(manifest) => manifest,
        Err(e) => {
            return Err(Error::Manifest(format!(
                "Failed to retrieve target manifest {target_id}: {e}"
            )));
        }
    };

    // Detect the hash algorithm used in the source manifest
    let algorithm = if let Some(first_ingredient) = source_manifest.ingredients.first() {
        hash::parse_algorithm(first_ingredient.data.alg.as_str())?
    } else {
        // If no ingredients, check if source manifest has any cross-references
        if let Some(first_cross_ref) = source_manifest.cross_references.first() {
            // Detect algorithm from existing cross-reference hash length
            hash::detect_hash_algorithm(&first_cross_ref.manifest_hash) // This already returns HashAlgorithm
        } else {
            HashAlgorithm::Sha384 // Default if no ingredients or cross-references
        }
    };

    // Check if a cross-reference to this target already exists
    let duplicate_ref = source_manifest
        .cross_references
        .iter()
        .find(|cr| cr.manifest_url == target_id);

    if let Some(existing_ref) = duplicate_ref {
        println!("Warning: A cross-reference to {target_id} already exists");

        // Check if hash matches (if it doesn't, this could indicate a conflict)
        let target_json = serde_json::to_string(&target_manifest)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        let target_hash = hash::calculate_hash_with_algorithm(target_json.as_bytes(), &algorithm);

        if existing_ref.manifest_hash != target_hash {
            // Handle conflict by creating a versioned reference
            println!("Manifest hash conflict detected, creating versioned reference");
            return create_versioned_link(
                source_manifest,
                target_manifest,
                source_id,
                target_id,
                storage,
                &algorithm,
            );
        } else {
            println!("Existing cross-reference is identical, no changes needed");
            return Ok(());
        }
    }

    // Create a hash of the target manifest using the detected algorithm
    let target_json =
        serde_json::to_string(&target_manifest).map_err(|e| Error::Serialization(e.to_string()))?;
    let target_hash = hash::calculate_hash_with_algorithm(target_json.as_bytes(), &algorithm);

    // Convert IDs to proper C2PA URNs if they're not already
    let target_urn = ensure_c2pa_urn(target_id);

    // Create a cross-reference from source to target
    let cross_reference = CrossReference::new(target_urn, target_hash);

    // Add the cross-reference to the source manifest
    source_manifest.cross_references.push(cross_reference);

    // Update the source manifest in storage
    let updated_id = storage.store_manifest(&source_manifest)?;

    println!("Successfully linked manifest {source_id} to {target_id}");
    println!("Updated manifest ID: {updated_id}");
    println!("Using hash algorithm: {}", algorithm.as_str());

    Ok(())
}

// Create a versioned link when there's a conflict
fn create_versioned_link(
    mut source_manifest: atlas_c2pa_lib::manifest::Manifest,
    target_manifest: atlas_c2pa_lib::manifest::Manifest,
    source_id: &str,
    target_id: &str,
    storage: &(impl StorageBackend + ?Sized),
    algorithm: &HashAlgorithm,
) -> Result<()> {
    // Generate a versioned ID following C2PA spec section 8.2
    // Format: original_urn:claim_generator:version_reason
    // where version_reason is version_number_reason_code

    // Parse the existing ID to maintain the UUID part
    let parts: Vec<&str> = target_id.split(':').collect();
    let uuid_part = if parts.len() >= 3 {
        parts[2] // Extract UUID from urn:c2pa:UUID format
    } else {
        target_id // Use as-is if not in expected format
    };

    // Extract claim generator info
    let claim_generator = target_manifest.claim.claim_generator_info.clone();

    // Find next version number by looking at existing references
    let mut max_version = 0;
    for cr in &source_manifest.cross_references {
        if cr
            .manifest_url
            .starts_with(&format!("urn:c2pa:{uuid_part}:"))
        {
            let parts: Vec<&str> = cr.manifest_url.split(':').collect();
            if parts.len() >= 5 {
                if let Some(version_reason) = parts.get(4) {
                    if let Some(version_str) = version_reason.split('_').next() {
                        if let Ok(version) = version_str.parse::<i32>() {
                            max_version = max_version.max(version);
                        }
                    }
                }
            }
        }
    }

    // Create new versioned ID
    // Reason code 1 = Conflict with another C2PA Manifest
    let versioned_id = format!(
        "urn:c2pa:{}:{}:{}_{}",
        uuid_part,
        claim_generator,
        max_version + 1,
        1
    );

    // Create a hash of the target manifest using the specified algorithm
    let target_json =
        serde_json::to_string(&target_manifest).map_err(|e| Error::Serialization(e.to_string()))?;
    let target_hash = hash::calculate_hash_with_algorithm(target_json.as_bytes(), algorithm);

    // Create a cross-reference with the versioned ID
    let cross_reference = CrossReference::new(versioned_id.clone(), target_hash);

    // Add the cross-reference to the source manifest
    source_manifest.cross_references.push(cross_reference);

    // Update the source manifest in storage
    let updated_id = storage.store_manifest(&source_manifest)?;

    println!(
        "Successfully linked manifest {source_id} to {target_id} (versioned as {versioned_id})"
    );
    println!("Updated manifest ID: {updated_id}");
    println!("Using hash algorithm: {}", algorithm.as_str());

    Ok(())
}

pub fn show_manifest(id: &str, storage: &(impl StorageBackend + ?Sized)) -> Result<()> {
    let manifest = storage.retrieve_manifest(id)?;

    println!("============ Manifest Details ============");
    println!("ID: {}", manifest.instance_id);
    println!("Title: {}", manifest.title);
    println!("Created: {}", manifest.created_at.0);
    println!("Claim Generator: {}", manifest.claim_generator);
    println!("Active: {}", manifest.is_active);

    // Display claim details
    println!("\n------------ Claim Details -------------");
    println!("Claim ID: {}", manifest.claim.instance_id);
    println!("Claim Generated: {}", manifest.claim.created_at.0);
    println!("Claim Generator: {}", manifest.claim.claim_generator_info);

    if let Some(signature) = &manifest.claim.signature {
        println!("\nSignature: {signature}");
    } else {
        println!("\nSignature: None (unsigned)");
    }

    // Display assertions
    println!("\n------------ Assertions -------------");
    for (i, assertion) in manifest.claim.created_assertions.iter().enumerate() {
        println!("\nAssertion #{}", i + 1);
        match assertion {
            atlas_c2pa_lib::assertion::Assertion::CreativeWork(creative) => {
                println!("  Type: CreativeWork");
                println!("  Context: {}", creative.context);
                println!("  Creative Type: {}", creative.creative_type);

                println!("  Authors:");
                for author in &creative.author {
                    println!("    - {} ({})", author.name, author.author_type);
                }
            }
            atlas_c2pa_lib::assertion::Assertion::Action(action) => {
                println!("  Type: Action");
                println!("  Actions:");
                for action in &action.actions {
                    println!("    - Action: {}", action.action);
                    if let Some(agent) = &action.software_agent {
                        println!("      Software Agent: {agent}");
                    }
                    if let Some(source_type) = &action.digital_source_type {
                        println!("      Digital Source Type: {source_type}");
                    }
                    if let Some(params) = &action.parameters {
                        println!(
                            "      Parameters: {}",
                            serde_json::to_string_pretty(params)
                                .unwrap_or_else(|_| format!("{params:?}"))
                        );
                    }
                }
            }
            _ => println!("  Unknown assertion type"),
        }
    }

    // Display ingredients
    println!("\n------------ Ingredients -------------");
    for (i, ingredient) in manifest.ingredients.iter().enumerate() {
        println!("\nIngredient #{}: {}", i + 1, ingredient.title);
        println!("  Document ID: {}", ingredient.document_id);
        println!("  Instance ID: {}", ingredient.instance_id);
        println!("  Format: {}", ingredient.format);
        println!("  Relationship: {}", ingredient.relationship);

        println!("  Data:");
        println!("    URL: {}", ingredient.data.url);
        println!("    Hash Algorithm: {}", ingredient.data.alg);
        println!("    Hash: {}", ingredient.data.hash);

        println!("    Data Types:");
        for data_type in &ingredient.data.data_types {
            println!("      - {data_type:?}");
        }

        if let Some(linked) = &ingredient.linked_ingredient {
            println!("  Linked Ingredient: {linked:?}");
        }

        if let Some(key) = &ingredient.public_key {
            println!("  Public Key: {key:?}");
        }
    }

    // Display cross-references if any
    if !manifest.cross_references.is_empty() {
        println!("\n------------ Cross References -------------");
        for (i, cross_ref) in manifest.cross_references.iter().enumerate() {
            println!("\nReference #{}", i + 1);
            println!("  URL: {}", cross_ref.manifest_url);
            println!("  Hash: {}", cross_ref.manifest_hash);
        }
    }

    Ok(())
}

pub mod linking {
    use crate::error::{Error, Result};
    use crate::storage::traits::StorageBackend;
    use atlas_c2pa_lib::ingredient::{Ingredient, LinkedIngredient};
    use atlas_c2pa_lib::manifest::Manifest;

    /// Links a dataset ingredient to a model ingredient
    pub fn link_dataset_to_model(
        model_manifest_id: &str,
        dataset_manifest_id: &str,
        storage: &dyn StorageBackend,
    ) -> Result<Manifest> {
        // Retrieve both manifests
        let mut model_manifest = storage.retrieve_manifest(model_manifest_id)?;
        let dataset_manifest = storage.retrieve_manifest(dataset_manifest_id)?;

        // Verify the dataset manifest type
        if !is_dataset_manifest(&dataset_manifest) {
            return Err(Error::Validation(format!(
                "Manifest {dataset_manifest_id} is not a dataset manifest"
            )));
        }

        // Get all dataset ingredients
        let dataset_ingredients = dataset_manifest.ingredients;

        // Update each model ingredient with dataset links
        for model_ingredient in &mut model_manifest.ingredients {
            for dataset_ingredient in &dataset_ingredients {
                // Create linked ingredient
                let linked_ingredient = create_linked_ingredient(dataset_ingredient)?;
                model_ingredient.data.linked_ingredient_url =
                    Some(dataset_ingredient.data.url.clone());
                model_ingredient.data.linked_ingredient_hash =
                    Some(dataset_ingredient.data.hash.clone());
                model_ingredient.linked_ingredient = Some(linked_ingredient);
            }
        }

        // Store updated model manifest
        storage.store_manifest(&model_manifest)?;

        Ok(model_manifest)
    }

    /// Checks if a manifest is a dataset manifest
    fn is_dataset_manifest(manifest: &Manifest) -> bool {
        manifest.ingredients.iter().any(|i| {
            matches!(
                i.data.data_types[0],
                atlas_c2pa_lib::asset_type::AssetType::Dataset
                    | atlas_c2pa_lib::asset_type::AssetType::DatasetOnnx
                    | atlas_c2pa_lib::asset_type::AssetType::DatasetTensorFlow
                    | atlas_c2pa_lib::asset_type::AssetType::DatasetPytorch
            )
        })
    }

    /// Creates linked ingredient from a dataset ingredient
    fn create_linked_ingredient(dataset_ingredient: &Ingredient) -> Result<LinkedIngredient> {
        Ok(LinkedIngredient {
            url: dataset_ingredient.data.url.clone(),
            hash: dataset_ingredient.data.hash.clone(),
            media_type: dataset_ingredient.format.clone(),
        })
    }
}

pub fn validate_linked_manifests(
    manifest_id: &str,
    storage: &(impl StorageBackend + ?Sized),
) -> Result<()> {
    let manifest = storage.retrieve_manifest(manifest_id)?;

    println!("Validating cross-references for manifest: {manifest_id}");

    if manifest.cross_references.is_empty() {
        println!("No cross-references found in manifest");
        return Ok(());
    }

    println!("Found {} cross-references", manifest.cross_references.len());

    let mut validation_errors = Vec::new();

    for (index, cross_ref) in manifest.cross_references.iter().enumerate() {
        println!(
            "\nValidating cross-reference #{}: {}",
            index + 1,
            cross_ref.manifest_url
        );

        // Validate the hash format first
        if let Err(hash_err) = validate_hash_format(&cross_ref.manifest_hash) {
            let error = format!("Invalid hash format: {hash_err}");
            validation_errors.push(error.clone());
            println!("  ❌ {error}");
            continue;
        }

        // Try to retrieve the referenced manifest
        match storage.retrieve_manifest(&cross_ref.manifest_url) {
            Ok(referenced_manifest) => {
                // Calculate hash of the referenced manifest
                let ref_json = match serde_json::to_string(&referenced_manifest) {
                    Ok(json) => json,
                    Err(e) => {
                        let error = format!("Failed to serialize referenced manifest: {e}");
                        validation_errors.push(error.clone());
                        println!("  ❌ {error}");
                        continue;
                    }
                };

                let algorithm = hash::detect_hash_algorithm(&cross_ref.manifest_hash);

                let calculated_hash =
                    hash::calculate_hash_with_algorithm(ref_json.as_bytes(), &algorithm);

                // Compare calculated hash with stored hash
                if calculated_hash == cross_ref.manifest_hash {
                    println!("  ✓ Hash verification successful");
                } else {
                    let error = format!(
                        "Hash mismatch for manifest {}: stored={}, calculated={}",
                        cross_ref.manifest_url, cross_ref.manifest_hash, calculated_hash
                    );
                    validation_errors.push(error.clone());
                    println!("  ❌ {error}");
                }

                // Check manifest structure
                match atlas_c2pa_lib::manifest::validate_manifest(&referenced_manifest) {
                    Ok(_) => println!("  ✓ Manifest structure validation successful"),
                    Err(e) => {
                        let error = format!("Manifest structure validation failed: {e}");
                        validation_errors.push(error.clone());
                        println!("  ❌ {error}");
                    }
                }
            }
            Err(e) => {
                let error = format!("Failed to retrieve referenced manifest: {e}");
                validation_errors.push(error.clone());
                println!("  ❌ {error}");
            }
        }
    }

    // Summarize validation results
    if validation_errors.is_empty() {
        println!("\nAll cross-references validated successfully");
        Ok(())
    } else {
        println!(
            "\nValidation failed with {} errors:",
            validation_errors.len()
        );
        for (i, error) in validation_errors.iter().enumerate() {
            println!("  {}. {}", i + 1, error);
        }
        Err(Error::Validation(
            "Cross-reference validation failed".to_string(),
        ))
    }
}

/// Check whether a given hash (hex-encoded) length matches one of the
/// C2PA-supported algorithms (must be one of SHA-256, SHA-384, SHA-512).
fn is_supported_c2pa_hash_length(hash_len: usize) -> bool {
    matches!(hash_len, 64 | 96 | 128)
}

/// Helper function to verify a single manifest link
pub fn verify_manifest_link(
    source_id: &str,
    target_id: &str,
    storage: &(impl StorageBackend + ?Sized),
) -> Result<bool> {
    let source_manifest = storage.retrieve_manifest(source_id)?;

    // Find the cross-reference to the target
    let target_urn = ensure_c2pa_urn(target_id);
    let cross_ref = source_manifest
        .cross_references
        .iter()
        .find(|cr| cr.manifest_url == target_id || cr.manifest_url == target_urn);

    match cross_ref {
        Some(reference) => {
            // Target reference found, verify hash
            let target_manifest = storage.retrieve_manifest(target_id)?;
            let target_json = serde_json::to_string(&target_manifest)
                .map_err(|e| Error::Serialization(e.to_string()))?;
            let algorithm = hash::detect_hash_algorithm(&reference.manifest_hash);
            let calculated_hash =
                hash::calculate_hash_with_algorithm(target_json.as_bytes(), &algorithm);

            if calculated_hash == reference.manifest_hash {
                println!("Manifest link verified: {source_id} -> {target_id}");
                println!("Hash verification successful");
                Ok(true)
            } else {
                println!("Hash mismatch for linked manifest: {target_id}");
                println!("  Stored hash:     {}", reference.manifest_hash);
                println!("  Calculated hash: {calculated_hash}");
                Ok(false)
            }
        }
        None => {
            println!("No link found from {source_id} to {target_id}");
            Ok(false)
        }
    }
}

/// Validate a manifest ID format
///
/// # Examples
///
/// ```
/// use atlas_cli::manifest::validate_manifest_id;
/// use uuid::Uuid;
///
/// // Valid UUID
/// let uuid = Uuid::new_v4().to_string();
/// assert!(validate_manifest_id(&uuid).is_ok());
///
/// // Valid C2PA URN
/// let urn = format!("urn:c2pa:{}", uuid);
/// assert!(validate_manifest_id(&urn).is_ok());
///
/// // Invalid: empty string
/// assert!(validate_manifest_id("").is_err());
///
/// // Valid: alphanumeric ID
/// assert!(validate_manifest_id("model-123").is_ok());
/// ```
pub fn validate_manifest_id(id: &str) -> Result<()> {
    // Basic validation
    if id.is_empty() {
        return Err(Error::Validation("Manifest ID cannot be empty".to_string()));
    }

    // Check if it's already a C2PA URN
    if id.starts_with("urn:c2pa:") {
        // Full validation according to spec
        let parts: Vec<&str> = id.split(':').collect();

        // Minimum: urn:c2pa:UUID
        if parts.len() < 3 {
            return Err(Error::Validation(
                "Invalid C2PA URN format. Expected urn:c2pa:UUID[:claim_generator[:version_reason]]".to_string()
            ));
        }

        // Validate UUID part
        if Uuid::parse_str(parts[2]).is_err() {
            return Err(Error::Validation(format!(
                "Invalid UUID in C2PA URN: '{}'",
                parts[2]
            )));
        }

        // If version_reason is present, validate it (format: version_reason)
        if parts.len() >= 5 {
            let version_reason = parts[4];
            let vr_parts: Vec<&str> = version_reason.split('_').collect();

            if vr_parts.len() != 2 {
                return Err(Error::Validation(format!(
                    "Invalid version_reason format: expected 'version_reason', got '{version_reason}'"
                )));
            }

            // Validate that both parts are numeric
            if vr_parts[0].parse::<u32>().is_err() {
                return Err(Error::Validation(format!(
                    "Invalid version number in version_reason: '{}'",
                    vr_parts[0]
                )));
            }

            if vr_parts[1].parse::<u32>().is_err() {
                return Err(Error::Validation(format!(
                    "Invalid reason code in version_reason: '{}'",
                    vr_parts[1]
                )));
            }
        }
    } else {
        // If not a URN, try to validate as UUID or other format
        if Uuid::parse_str(id).is_ok() {
            // Valid UUID, which is good
        } else if !id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            // Basic validation for other ID formats
            return Err(Error::Validation(format!(
                "Invalid manifest ID format: '{id}'. Expected URN, UUID, or alphanumeric ID"
            )));
        }
    }

    Ok(())
}

/// Ensure an ID is in C2PA URN format
///
/// # Examples
///
/// ```
/// use atlas_cli::manifest::ensure_c2pa_urn;
/// use uuid::Uuid;
///
/// // UUID gets converted to URN
/// let uuid = Uuid::new_v4().to_string();
/// let urn = ensure_c2pa_urn(&uuid);
/// assert!(urn.starts_with("urn:c2pa:"));
///
/// // Already a URN remains unchanged
/// let existing_urn = "urn:c2pa:12345678-1234-1234-1234-123456789012";
/// assert_eq!(ensure_c2pa_urn(existing_urn), existing_urn);
///
/// // Non-UUID gets new UUID generated
/// let result = ensure_c2pa_urn("custom-id");
/// assert!(result.starts_with("urn:c2pa:"));
/// ```
pub fn ensure_c2pa_urn(id: &str) -> String {
    if id.starts_with("urn:c2pa:") {
        id.to_string() // Already in correct format
    } else if Uuid::parse_str(id).is_ok() {
        // It's a valid UUID, convert to URN
        format!("urn:c2pa:{id}")
    } else {
        // Not a UUID, generate a new one
        let uuid = Uuid::new_v4(); // Using new_v4() instead of new_v5()
        format!("urn:c2pa:{uuid}")
    }
}

/// Extract UUID from a C2PA URN
///
/// # Examples
///
/// ```
/// use atlas_cli::manifest::extract_uuid_from_urn;
/// use uuid::Uuid;
///
/// let uuid = Uuid::new_v4();
/// let urn = format!("urn:c2pa:{}", uuid);
///
/// let extracted = extract_uuid_from_urn(&urn).unwrap();
/// assert_eq!(extracted, uuid);
///
/// // Invalid URN format
/// assert!(extract_uuid_from_urn("invalid:urn").is_err());
/// ```
pub fn extract_uuid_from_urn(urn: &str) -> Result<Uuid> {
    let parts: Vec<&str> = urn.split(':').collect();

    if parts.len() < 3 || parts[0] != "urn" || parts[1] != "c2pa" {
        return Err(Error::Validation(format!(
            "Invalid C2PA URN format: '{urn}'"
        )));
    }

    Uuid::parse_str(parts[2])
        .map_err(|e| Error::Validation(format!("Invalid UUID in C2PA URN '{urn}': {e}")))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestNode {
    pub id: String,
    pub title: String,
    pub manifest_type: String,
    pub created_at: String,
    pub ingredients: Vec<String>,
    pub assertions: Vec<AssertionInfo>,
    pub references: Vec<ReferenceInfo>,
    pub signature: Option<bool>,
}

/// Simplified representation of an assertion for export
#[derive(Debug, Serialize, Deserialize)]
pub struct AssertionInfo {
    pub type_name: String,
    pub details: serde_json::Value,
}

/// Simplified representation of a cross-reference for export
#[derive(Debug, Serialize, Deserialize)]
pub struct ReferenceInfo {
    pub target_id: String,
    pub relation_type: String, // "references", "isReferencedBy", etc.
}

/// Full provenance graph representation
#[derive(Debug, Serialize, Deserialize)]
pub struct ProvenanceGraph {
    pub root_id: String,
    pub nodes: HashMap<String, ManifestNode>,
    pub edges: Vec<Edge>,
}

/// Edge in the provenance graph
#[derive(Debug, Serialize, Deserialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub relation_type: String,
}

/// Export the full provenance graph for a manifest
pub fn export_provenance(
    id: &str,
    storage: &(impl StorageBackend + ?Sized),
    format: &str,
    output_path: Option<&str>,
    max_depth: u32,
) -> Result<()> {
    // Retrieve the root manifest, we just care if exisit, so _
    let _root_manifest = match storage.retrieve_manifest(id) {
        Ok(manifest) => manifest,
        Err(e) => {
            return Err(Error::Manifest(format!(
                "Failed to retrieve root manifest {id}: {e}"
            )));
        }
    };

    // Initialize provenance graph data structure
    let mut graph = ProvenanceGraph {
        root_id: id.to_string(),
        nodes: HashMap::new(),
        edges: Vec::new(),
    };

    // Keep track of visited manifests to avoid cycles
    let mut visited = HashSet::new();

    // Build the graph recursively starting from the root manifest
    build_provenance_graph(id, storage, &mut graph, &mut visited, max_depth, 0)?;

    // Serialize the graph based on the requested format
    let serialized = match format.to_lowercase().as_str() {
        "json" => serde_json::to_string_pretty(&graph)
            .map_err(|e| Error::Serialization(format!("Failed to serialize to JSON: {e}")))?,
        "yaml" => {
            #[cfg(feature = "yaml")]
            {
                serde_yaml::to_string(&graph).map_err(|e| {
                    Error::Serialization(format!("Failed to serialize to YAML: {e}"))
                })?
            }

            #[cfg(not(feature = "yaml"))]
            {
                return Err(Error::Validation("YAML format not supported. Add serde_yaml to dependencies and enable the 'yaml' feature.".to_string()));
            }
        }
        _ => {
            return Err(Error::Validation(format!(
                "Invalid output format '{format}'. Valid options are: json, yaml"
            )));
        }
    };

    // Output the serialized graph
    if let Some(path) = output_path {
        let mut file = File::create(path).map_err(Error::Io)?;
        file.write_all(serialized.as_bytes()).map_err(Error::Io)?;
        println!("Provenance graph exported to: {path}");
    } else {
        // Print to stdout
        println!("{serialized}");
    }

    Ok(())
}
/// Recursively build the provenance graph
fn build_provenance_graph(
    id: &str,
    storage: &(impl StorageBackend + ?Sized),
    graph: &mut ProvenanceGraph,
    visited: &mut HashSet<String>,
    max_depth: u32,
    current_depth: u32,
) -> Result<()> {
    // Check if we've already visited this manifest or exceeded max depth
    if visited.contains(id) || current_depth > max_depth {
        return Ok(());
    }

    // Mark as visited
    visited.insert(id.to_string());

    // Retrieve the manifest
    let manifest = match storage.retrieve_manifest(id) {
        Ok(manifest) => manifest,
        Err(e) => {
            return Err(Error::Manifest(format!(
                "Failed to retrieve manifest {id}: {e}"
            )));
        }
    };

    // Determine manifest type using the new function
    let manifest_type = determine_manifest_type(&manifest);

    // Extract assertions
    let mut assertions = Vec::new();
    if let Some(claim) = &manifest.claim_v2 {
        for assertion in &claim.created_assertions {
            let details = extract_assertion_details(assertion);
            let type_name = match assertion {
                atlas_c2pa_lib::assertion::Assertion::CreativeWork(_) => "CreativeWork",
                atlas_c2pa_lib::assertion::Assertion::Action(_) => "Action",
                atlas_c2pa_lib::assertion::Assertion::DoNotTrain(_) => "DoNotTrain",
                atlas_c2pa_lib::assertion::Assertion::CustomAssertion(_) => "TrustedHardware",
                _ => "Other",
            };
            assertions.push(AssertionInfo {
                type_name: type_name.to_string(),
                details,
            });
        }
    }

    // Extract ingredient IDs
    let ingredient_ids = manifest
        .ingredients
        .iter()
        .map(|ingredient| ingredient.instance_id.clone())
        .collect::<Vec<String>>();

    // Create node for this manifest
    let node = ManifestNode {
        id: id.to_string(),
        title: manifest.title.clone(),
        manifest_type: manifest_type_to_string(&manifest_type),
        created_at: manifest.created_at.0.to_string(),
        ingredients: ingredient_ids,
        assertions,
        references: Vec::new(), // Will populate below
        signature: manifest.claim_v2.as_ref().map(|c| c.signature.is_some()),
    };

    // Add node to graph
    graph.nodes.insert(id.to_string(), node);

    // Process cross-references
    for cross_ref in &manifest.cross_references {
        let target_id = &cross_ref.manifest_url;

        // Add references to the node
        if let Some(node) = graph.nodes.get_mut(id) {
            node.references.push(ReferenceInfo {
                target_id: target_id.clone(),
                relation_type: "references".to_string(),
            });
        }

        // Add edge to the graph
        graph.edges.push(Edge {
            source: id.to_string(),
            target: target_id.clone(),
            relation_type: "references".to_string(),
        });

        // Recursively process the referenced manifest
        build_provenance_graph(
            target_id,
            storage,
            graph,
            visited,
            max_depth,
            current_depth + 1,
        )?;

        // Add backward edge for the referenced manifest
        if let Some(node) = graph.nodes.get_mut(target_id) {
            node.references.push(ReferenceInfo {
                target_id: id.to_string(),
                relation_type: "isReferencedBy".to_string(),
            });
        }

        // Add backward edge to the graph
        graph.edges.push(Edge {
            source: target_id.clone(),
            target: id.to_string(),
            relation_type: "isReferencedBy".to_string(),
        });
    }

    Ok(())
}

/// Extract details from an assertion in a simplified form
fn extract_assertion_details(
    assertion: &atlas_c2pa_lib::assertion::Assertion,
) -> serde_json::Value {
    match assertion {
        atlas_c2pa_lib::assertion::Assertion::CreativeWork(creative) => {
            serde_json::json!({
                "creative_type": creative.creative_type,
                "authors": creative.author.iter().map(|a| {
                    serde_json::json!({
                        "type": a.author_type,
                        "name": a.name,
                    })
                }).collect::<Vec<_>>(),
            })
        }
        atlas_c2pa_lib::assertion::Assertion::Action(action) => {
            serde_json::json!({
                "actions": action.actions.iter().map(|a| {
                    let mut action_obj = serde_json::json!({
                        "action": a.action,
                    });

                    if let Some(agent) = &a.software_agent {
                        action_obj.as_object_mut().unwrap().insert(
                            "software_agent".to_string(),
                            serde_json::Value::String(agent.clone())
                        );
                    }

                    if let Some(params) = &a.parameters {
                        action_obj.as_object_mut().unwrap().insert(
                            "parameters".to_string(),
                            params.clone()
                        );
                    }

                    action_obj
                }).collect::<Vec<_>>(),
            })
        }
        atlas_c2pa_lib::assertion::Assertion::DoNotTrain(do_not_train) => {
            serde_json::json!({
                "reason": do_not_train.reason,
                "enforced": do_not_train.enforced,
            })
        }
        _ => serde_json::json!({"type": "Unknown"}),
    }
}
