use super::common::MockStorageBackend;
use crate::error::Result;
use crate::hash::calculate_file_hash;
use crate::manifest::{common::AssetKind, dataset, model};
use crate::utils::safe_create_file;
use atlas_c2pa_lib::assertion::{
    Action, ActionAssertion, Assertion, Author, CreativeWorkAssertion,
};
use atlas_c2pa_lib::asset_type::AssetType;
use atlas_c2pa_lib::claim::ClaimV2;
use atlas_c2pa_lib::datetime_wrapper::OffsetDateTimeWrapper;
use atlas_c2pa_lib::ingredient::{Ingredient, IngredientData};
use atlas_c2pa_lib::manifest::Manifest;
use std::io::Write;
use tempfile::tempdir;
use time::OffsetDateTime;
use uuid::Uuid;

#[test]
fn test_dataset_verification() -> Result<()> {
    // Create a temporary directory and dataset file
    let dir = tempdir()?;
    let dataset_path = dir.path().join("test_dataset.csv");

    // Create and write initial content
    let content = b"test,data\n1,2,3";
    {
        let mut file = safe_create_file(&dataset_path, false)?;
        file.write_all(content)?;
    }

    // Create ingredient with proper file path and hash
    let manifest_id = format!("test_manifest_{}", Uuid::new_v4());
    let ingredient = create_test_ingredient_internal(
        &dataset_path,
        "Test Dataset",
        AssetType::Dataset,
        "text/csv",
    )?;

    // Create complete manifest
    let manifest = create_test_manifest_internal(
        manifest_id.clone(),
        vec![ingredient],
        "Test Dataset Manifest",
        AssetKind::Dataset,
    )?;

    // Initialize storage
    let storage = MockStorageBackend::new(manifest);

    // First verification should succeed
    assert!(
        dataset::verify_dataset_manifest(&manifest_id, &storage).is_ok(),
        "Initial verification should succeed"
    );

    // Modify file
    {
        let mut file = safe_create_file(&dataset_path, false)?;
        file.write_all(b"modified,data\n4,5,6")?;
    }

    // Verification should fail after modification
    assert!(
        dataset::verify_dataset_manifest(&manifest_id, &storage).is_err(),
        "Verification should fail after file modification"
    );

    Ok(())
}

#[test]
fn test_model_verification() -> Result<()> {
    // Create a temporary directory and model file
    let dir = tempdir()?;
    let model_path = dir.path().join("test_model.onnx");

    // Create and write initial content
    let content = b"mock model data";
    {
        let mut file = safe_create_file(&model_path, false)?;
        file.write_all(content)?;
    }

    // Create ingredient with proper file path and hash
    let manifest_id = format!("test_manifest_{}", Uuid::new_v4());
    let ingredient = create_test_ingredient_internal(
        &model_path,
        "Test Model",
        AssetType::ModelOnnx,
        "application/onnx",
    )?;

    // Create complete manifest
    let manifest = create_test_manifest_internal(
        manifest_id.clone(),
        vec![ingredient],
        "Test Model Manifest",
        AssetKind::Model,
    )?;

    // Initialize storage
    let storage = MockStorageBackend::new(manifest);

    // First verification should succeed
    assert!(
        model::verify_model_manifest(&manifest_id, &storage).is_ok(),
        "Initial verification should succeed"
    );

    // Modify file
    {
        let mut file = safe_create_file(&model_path, false)?;
        file.write_all(b"modified model data")?;
    }

    // Verification should fail after modification
    assert!(
        model::verify_model_manifest(&manifest_id, &storage).is_err(),
        "Verification should fail after file modification"
    );

    Ok(())
}

fn create_test_ingredient_internal(
    path: &std::path::Path,
    title: &str,
    asset_type: AssetType,
    format: &str,
) -> Result<Ingredient> {
    let url = format!("file://{}", path.to_string_lossy());
    let hash = calculate_file_hash(path)?;

    Ok(Ingredient {
        title: title.to_string(),
        format: format.to_string(),
        relationship: "componentOf".to_string(),
        document_id: format!("uuid:{}", Uuid::new_v4()),
        instance_id: format!("uuid:{}", Uuid::new_v4()),
        data: IngredientData {
            url,
            alg: "sha256".to_string(),
            hash,
            data_types: vec![asset_type],
            linked_ingredient_url: None,
            linked_ingredient_hash: None,
        },
        linked_ingredient: None,
        public_key: None,
    })
}

fn create_test_manifest_internal(
    id: String,
    ingredients: Vec<Ingredient>,
    title: &str,
    asset_kind: AssetKind,
) -> Result<Manifest> {
    // Create appropriate assertions based on asset kind
    let creative_type = match asset_kind {
        AssetKind::Model => "Model",
        AssetKind::Dataset => "Dataset",
        AssetKind::Software => "Software",
        AssetKind::Evaluation => "EvaluationResult",
    };

    let digital_source_type = match asset_kind {
        AssetKind::Model => "http://cv.iptc.org/newscodes/digitalsourcetype/algorithmicMedia",
        AssetKind::Dataset => "http://cv.iptc.org/newscodes/digitalsourcetype/dataset",
        AssetKind::Software => "http://cv.iptc.org/newscodes/digitalsourcetype/software",
        AssetKind::Evaluation => "http://cv.iptc.org/newscodes/digitalsourcetype/evaluationResult",
    };

    let assertions = vec![
        Assertion::CreativeWork(CreativeWorkAssertion {
            context: "http://schema.org/".to_string(),
            creative_type: creative_type.to_string(),
            author: vec![Author {
                author_type: "Organization".to_string(),
                name: "Test Organization".to_string(),
            }],
        }),
        Assertion::Action(ActionAssertion {
            actions: vec![Action {
                action: match asset_kind {
                    AssetKind::Evaluation => "c2pa.evaluation".to_string(),
                    _ => "c2pa.created".to_string(),
                },
                software_agent: Some("test".to_string()),
                parameters: Some(serde_json::json!({
                    "name": title,
                })),
                digital_source_type: Some(digital_source_type.to_string()),
                instance_id: None,
            }],
        }),
    ];

    // Create claim with the assertions
    let claim = ClaimV2 {
        instance_id: format!("xmp:iid:{}", Uuid::new_v4()),
        ingredients: ingredients.clone(),
        created_assertions: assertions,
        claim_generator_info: "test".to_string(),
        signature: None,
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
    };

    Ok(Manifest {
        claim_generator: "test".to_string(),
        title: title.to_string(),
        instance_id: id,
        ingredients,
        claim: claim.clone(),
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
        cross_references: vec![],
        claim_v2: Some(claim),
        is_active: true,
    })
}

#[test]
fn test_cross_reference_verification() -> Result<()> {
    // Create a temporary directory and files
    let dir = tempdir()?;
    let dataset_path = dir.path().join("cross_ref_dataset.csv");
    let model_path = dir.path().join("cross_ref_model.onnx");

    // Create and write content to files
    {
        let mut file = safe_create_file(&dataset_path, false)?;
        file.write_all(b"test,data\n1,2,3")?;

        let mut file = safe_create_file(&model_path, false)?;
        file.write_all(b"model data")?;
    }

    // Create dataset manifest
    let dataset_id = format!("dataset_{}", Uuid::new_v4());
    let dataset_ingredient = create_test_ingredient_internal(
        &dataset_path,
        "Test Dataset",
        AssetType::Dataset,
        "text/csv",
    )?;
    let dataset_manifest = create_test_manifest_internal(
        dataset_id.clone(),
        vec![dataset_ingredient],
        "Test Dataset Manifest",
        AssetKind::Dataset,
    )?;

    // Create model manifest with cross-reference to dataset
    let model_id = format!("model_{}", Uuid::new_v4());
    let model_ingredient = create_test_ingredient_internal(
        &model_path,
        "Test Model",
        AssetType::ModelOnnx,
        "application/onnx",
    )?;

    // Create model manifest (will add cross-reference later)
    let mut model_manifest = create_test_manifest_internal(
        model_id.clone(),
        vec![model_ingredient],
        "Test Model Manifest",
        AssetKind::Model,
    )?;

    // Initialize storage with both manifests
    let mut storage = MockStorageBackend::new(dataset_manifest.clone());

    // Create cross-reference from model to dataset
    use sha2::{Digest, Sha256};
    let dataset_json = serde_json::to_string(&dataset_manifest).unwrap();
    let dataset_hash = hex::encode(Sha256::digest(dataset_json.as_bytes()));

    // Add cross-reference to model manifest
    let cross_ref = atlas_c2pa_lib::cross_reference::CrossReference {
        manifest_url: dataset_id.clone(),
        manifest_hash: dataset_hash,
        media_type: Some("application/json".to_string()),
    };
    model_manifest.cross_references.push(cross_ref);

    // Update the model manifest in storage
    storage.add_manifest(model_manifest);

    // Verification should succeed with proper cross-reference
    assert!(
        model::verify_model_manifest(&model_id, &storage).is_ok(),
        "Verification should succeed with valid cross-reference"
    );

    // Now modify the dataset manifest in storage to cause a hash mismatch
    let mut modified_dataset_manifest = dataset_manifest;
    modified_dataset_manifest.title = "Modified Dataset Title".to_string();
    storage.add_manifest(modified_dataset_manifest);

    // Verification should now fail due to cross-reference hash mismatch
    assert!(
        model::verify_model_manifest(&model_id, &storage).is_err(),
        "Verification should fail with invalid cross-reference hash"
    );

    Ok(())
}
#[test]
fn test_evaluation_with_both_conditions() -> Result<()> {
    use crate::storage::traits::StorageBackend;
    use sha2::{Digest, Sha256};

    // Create necessary files
    let dir = tempdir()?;
    let eval_path = dir.path().join("test_eval_results.json");
    let model_path = dir.path().join("test_model.onnx");
    let dataset_path = dir.path().join("test_dataset.csv");

    // Create file content
    {
        let mut file = safe_create_file(&eval_path, false)?;
        file.write_all(b"{\"accuracy\": 0.92, \"f1\": 0.89}")?;

        let mut file = safe_create_file(&model_path, false)?;
        file.write_all(b"model data")?;

        let mut file = safe_create_file(&dataset_path, false)?;
        file.write_all(b"test,data\n1,2,3")?;
    }

    // Create storage
    let mut storage = MockStorageBackend::new_empty();

    // DATASET PART - Both conditions
    // 1. Create dataset ingredient with correct AssetType
    let dataset_id = format!("dataset_{}", Uuid::new_v4());

    // Make sure to use AssetType::Dataset here
    let dataset_ingredient = create_test_ingredient_internal(
        &dataset_path,
        "Test Dataset",
        AssetType::Dataset, // Critical: Use Dataset AssetType
        "text/csv",
    )?;

    // 2. Create dataset assertion with creative_type "Dataset"
    let dataset_assertions = vec![
        Assertion::CreativeWork(CreativeWorkAssertion {
            context: "http://schema.org/".to_string(),
            creative_type: "Dataset".to_string(), // Critical: Use "Dataset" string
            author: vec![Author {
                author_type: "Organization".to_string(),
                name: "Test Organization".to_string(),
            }],
        }),
        Assertion::Action(ActionAssertion {
            actions: vec![Action {
                action: "c2pa.created".to_string(),
                software_agent: Some("c2pa-cli".to_string()),
                parameters: Some(serde_json::json!({
                    "name": "Test Dataset Manifest"
                })),
                digital_source_type: Some(
                    "http://cv.iptc.org/newscodes/digitalsourcetype/dataset".to_string(),
                ),
                instance_id: None,
            }],
        }),
    ];

    let dataset_claim = ClaimV2 {
        instance_id: format!("xmp:iid:{}", Uuid::new_v4()),
        ingredients: vec![dataset_ingredient.clone()],
        created_assertions: dataset_assertions,
        claim_generator_info: "c2pa-cli".to_string(),
        signature: None,
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
    };

    let dataset_manifest = Manifest {
        claim_generator: "c2pa-cli/0.1.0".to_string(),
        title: "Test Dataset Manifest".to_string(),
        instance_id: dataset_id.clone(),
        ingredients: vec![dataset_ingredient],
        claim: dataset_claim.clone(),
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
        cross_references: vec![],
        claim_v2: Some(dataset_claim),
        is_active: true,
    };

    // MODEL PART
    let model_id = format!("model_{}", Uuid::new_v4());
    let model_ingredient = create_test_ingredient_internal(
        &model_path,
        "Test Model",
        AssetType::ModelOnnx, // Use correct model type
        "application/onnx",
    )?;

    let model_assertions = vec![
        Assertion::CreativeWork(CreativeWorkAssertion {
            context: "http://schema.org/".to_string(),
            creative_type: "Model".to_string(),
            author: vec![Author {
                author_type: "Organization".to_string(),
                name: "Test Organization".to_string(),
            }],
        }),
        Assertion::Action(ActionAssertion {
            actions: vec![Action {
                action: "c2pa.created".to_string(),
                software_agent: Some("c2pa-cli".to_string()),
                parameters: Some(serde_json::json!({
                    "name": "Test Model Manifest"
                })),
                digital_source_type: Some(
                    "http://cv.iptc.org/newscodes/digitalsourcetype/algorithmicMedia".to_string(),
                ),
                instance_id: None,
            }],
        }),
    ];

    let model_claim = ClaimV2 {
        instance_id: format!("xmp:iid:{}", Uuid::new_v4()),
        ingredients: vec![model_ingredient.clone()],
        created_assertions: model_assertions,
        claim_generator_info: "c2pa-cli".to_string(),
        signature: None,
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
    };

    let model_manifest = Manifest {
        claim_generator: "c2pa-cli/0.1.0".to_string(),
        title: "Test Model Manifest".to_string(),
        instance_id: model_id.clone(),
        ingredients: vec![model_ingredient],
        claim: model_claim.clone(),
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
        cross_references: vec![],
        claim_v2: Some(model_claim),
        is_active: true,
    };

    // Store dataset and model manifests
    storage.add_manifest(dataset_manifest);
    storage.add_manifest(model_manifest);

    // Print info about stored manifests for debugging
    println!("\n=== DATASET MANIFEST ===");
    if let Ok(dataset) = storage.retrieve_manifest(&dataset_id) {
        println!("Dataset ID: {}", dataset.instance_id);

        // Verify that the dataset has AssetType::Dataset
        let has_dataset_type = dataset.ingredients.iter().any(|ingredient| {
            ingredient
                .data
                .data_types
                .iter()
                .any(|t| matches!(t, AssetType::Dataset))
        });
        println!("Has AssetType::Dataset: {has_dataset_type}");

        // Verify the dataset has a CreativeWork assertion with type "Dataset"
        let has_dataset_creative_type = if let Some(claim) = &dataset.claim_v2 {
            claim.created_assertions.iter().any(|assertion| {
                if let Assertion::CreativeWork(creative) = assertion {
                    creative.creative_type == "Dataset"
                } else {
                    false
                }
            })
        } else {
            false
        };
        println!("Has creative_type 'Dataset': {has_dataset_creative_type}");
    }

    // EVALUATION PART
    let eval_id = format!("eval_{}", Uuid::new_v4());
    let eval_ingredient = create_test_ingredient_internal(
        &eval_path,
        "Evaluation Results",
        AssetType::Dataset,
        "application/json",
    )?;

    let eval_assertions = vec![
        Assertion::CreativeWork(CreativeWorkAssertion {
            context: "http://schema.org/".to_string(),
            creative_type: "EvaluationResult".to_string(),
            author: vec![Author {
                author_type: "Organization".to_string(),
                name: "Test Organization".to_string(),
            }],
        }),
        Assertion::Action(ActionAssertion {
            actions: vec![Action {
                action: "c2pa.evaluation".to_string(),
                software_agent: Some("c2pa-cli".to_string()),
                parameters: Some(serde_json::json!({
                    "name": "Test Evaluation",
                    "model_id": model_id,
                    "dataset_id": dataset_id,
                    "metrics": {
                        "accuracy": "0.92",
                        "f1": "0.89"
                    }
                })),
                digital_source_type: Some(
                    "http://cv.iptc.org/newscodes/digitalsourcetype/evaluationResult".to_string(),
                ),
                instance_id: None,
            }],
        }),
    ];

    // Create cross-references
    let mut cross_references = Vec::new();

    // Add model cross-reference
    if let Ok(model_manifest) = storage.retrieve_manifest(&model_id) {
        let model_json = serde_json::to_string(&model_manifest)
            .map_err(|e| crate::error::Error::Serialization(e.to_string()))?;
        let model_hash = hex::encode(Sha256::digest(model_json.as_bytes()));

        cross_references.push(atlas_c2pa_lib::cross_reference::CrossReference {
            manifest_url: model_id.clone(),
            manifest_hash: model_hash,
            media_type: Some("application/json".to_string()),
        });
    }

    // Add dataset cross-reference
    if let Ok(dataset_manifest) = storage.retrieve_manifest(&dataset_id) {
        let dataset_json = serde_json::to_string(&dataset_manifest)
            .map_err(|e| crate::error::Error::Serialization(e.to_string()))?;
        let dataset_hash = hex::encode(Sha256::digest(dataset_json.as_bytes()));

        cross_references.push(atlas_c2pa_lib::cross_reference::CrossReference {
            manifest_url: dataset_id.clone(),
            manifest_hash: dataset_hash,
            media_type: Some("application/json".to_string()),
        });
    }

    // Create evaluation claim
    let eval_claim = ClaimV2 {
        instance_id: format!("xmp:iid:{}", Uuid::new_v4()),
        ingredients: vec![eval_ingredient.clone()],
        created_assertions: eval_assertions,
        claim_generator_info: "c2pa-cli".to_string(),
        signature: None,
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
    };

    // Create evaluation manifest
    let eval_manifest = Manifest {
        claim_generator: "c2pa-cli/0.1.0".to_string(),
        title: "Test Evaluation".to_string(),
        instance_id: eval_id.clone(),
        ingredients: vec![eval_ingredient],
        claim: eval_claim.clone(),
        created_at: OffsetDateTimeWrapper(OffsetDateTime::now_utc()),
        cross_references,
        claim_v2: Some(eval_claim),
        is_active: true,
    };

    // Add evaluation manifest to storage
    storage.add_manifest(eval_manifest);

    // Verify the evaluation manifest
    println!("\n=== PERFORMING VERIFICATION ===");
    println!("Attempting to verify evaluation manifest with ID: {eval_id}");

    match crate::manifest::evaluation::verify_evaluation_manifest(&eval_id, &storage) {
        Ok(_) => {
            println!("✓ Evaluation verification successful");
        }
        Err(e) => {
            println!("✗ Evaluation verification failed: {e}");
            return Err(e);
        }
    }

    Ok(())
}
