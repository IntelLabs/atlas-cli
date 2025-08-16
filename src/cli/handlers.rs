use crate::error::{Error, Result};

use super::commands::{
    CCAttestationCommands, DatasetCommands, EvaluationCommands, ManifestCommands, ModelCommands,
    SoftwareCommands,
};
use crate::cc_attestation;
use crate::manifest;
use crate::manifest::config::ManifestCreationConfig;
use crate::manifest::dataset::list_dataset_manifests;
use crate::storage::database::DatabaseStorage;
use crate::storage::filesystem::FilesystemStorage;
use crate::storage::rekor::RekorStorage;

use crate::StorageBackend;

pub fn handle_dataset_command(cmd: DatasetCommands) -> Result<()> {
    let _storage = RekorStorage::new()?;
    match cmd {
        DatasetCommands::Create {
            paths,
            ingredient_names,
            name,
            author_org,
            author_name,
            description,
            linked_manifests,
            storage_type,
            storage_url,
            print,
            format,
            key,
            hash_alg,
            with_tdx,
        } => {
            let storage: Option<&'static dyn StorageBackend> = match storage_type.as_str() {
                "database" => {
                    let db_storage = Box::new(DatabaseStorage::new(*storage_url.clone())?);
                    Some(Box::leak(db_storage))
                }
                "rekor" => {
                    let rekor_storage = Box::new(RekorStorage::new_with_url(*storage_url.clone())?);
                    Some(Box::leak(rekor_storage))
                }
                "local-fs" => {
                    let fs_storage = Box::new(FilesystemStorage::new(storage_url.as_str())?);
                    Some(Box::leak(fs_storage))
                }
                _ => None,
            };

            let config = ManifestCreationConfig {
                paths,
                ingredient_names,
                name,
                author_org,
                author_name,
                description,
                linked_manifests,
                storage,
                print,
                output_encoding: format,
                key_path: key,
                hash_alg: hash_alg.to_cose_algorithm(),
                with_cc: with_tdx,
                software_type: None,
                version: None,
                custom_fields: None,
            };

            manifest::create_dataset_manifest(config)
        }
        DatasetCommands::List {
            storage_type,
            storage_url,
        } => {
            let storage: Box<dyn StorageBackend> = match storage_type.as_str() {
                "database" => Box::new(DatabaseStorage::new(*storage_url.clone())?),
                "rekor" => Box::new(RekorStorage::new_with_url(*storage_url.clone())?),
                "local-fs" => Box::new(FilesystemStorage::new(storage_url.as_str())?),
                _ => return Err(Error::Validation("Invalid storage type".to_string())),
            };

            list_dataset_manifests(storage.as_ref())
        }
        DatasetCommands::Verify {
            id,
            storage_type,
            storage_url,
        } => {
            let storage: Box<dyn StorageBackend> = match storage_type.as_str() {
                "database" => Box::new(DatabaseStorage::new(*storage_url.clone())?),
                "rekor" => Box::new(RekorStorage::new_with_url(*storage_url.clone())?),
                "local-fs" => Box::new(FilesystemStorage::new(storage_url.as_str())?),
                _ => return Err(Error::Validation("Invalid storage type".to_string())),
            };

            manifest::verify_dataset_manifest(&id, storage.as_ref())
        }
    }
}

pub fn handle_model_command(cmd: ModelCommands) -> Result<()> {
    let _storage = RekorStorage::new()?;
    match cmd {
        ModelCommands::Create {
            paths,
            ingredient_names,
            name,
            author_org,
            author_name,
            description,
            linked_manifests,
            storage_type,
            storage_url,
            print,
            encoding,
            format,
            key,
            hash_alg,
            with_tdx,
        } => {
            let storage: Option<&'static dyn StorageBackend> = match storage_type.as_str() {
                "database" => {
                    let db_storage = Box::new(DatabaseStorage::new(*storage_url.clone())?);
                    Some(Box::leak(db_storage))
                }
                "rekor" => {
                    let rekor_storage = Box::new(RekorStorage::new_with_url(*storage_url.clone())?);
                    Some(Box::leak(rekor_storage))
                }
                "local-fs" => {
                    let fs_storage = Box::new(FilesystemStorage::new(storage_url.as_str())?);
                    Some(Box::leak(fs_storage))
                }
                _ => None,
            };

            let config = ManifestCreationConfig {
                paths,
                ingredient_names,
                name,
                author_org,
                author_name,
                description,
                linked_manifests,
                storage,
                print,
                output_encoding: encoding,
                key_path: key,
                hash_alg: hash_alg.to_cose_algorithm(),
                with_cc: with_tdx,
                software_type: None,
                version: None,
                custom_fields: None,
            };

            match format.as_str() {
                "standalone" => manifest::create_model_manifest(config),
                "oms" => manifest::common::create_oms_manifest(config),
                _ => {
                    return Err(Error::InitializationError(
                        "Unsupported output format".to_string(),
                    ));
                }
            }
        }
        ModelCommands::List {
            storage_type,
            storage_url,
        } => {
            let storage: Box<dyn StorageBackend> = match storage_type.as_str() {
                "database" => Box::new(DatabaseStorage::new(*storage_url.clone())?),
                "rekor" => Box::new(RekorStorage::new_with_url(*storage_url.clone())?),
                "local-fs" => Box::new(FilesystemStorage::new(storage_url.as_str())?),
                _ => return Err(Error::Validation("Invalid storage type".to_string())),
            };

            manifest::list_model_manifest(storage.as_ref())
        }
        ModelCommands::Verify {
            id,
            storage_type,
            storage_url,
        } => {
            let storage: Box<dyn StorageBackend> = match storage_type.as_str() {
                "database" => Box::new(DatabaseStorage::new(*storage_url.clone())?),
                "rekor" => Box::new(RekorStorage::new_with_url(*storage_url.clone())?),
                "local-fs" => Box::new(FilesystemStorage::new(storage_url.as_str())?),
                _ => return Err(Error::Validation("Invalid storage type".to_string())),
            };

            manifest::verify_model_manifest(&id, storage.as_ref())
        }
        ModelCommands::LinkDataset {
            model_id,
            dataset_id,
            storage_type,
            storage_url,
        } => {
            let storage: Box<dyn StorageBackend> = match storage_type.as_str() {
                "database" => Box::new(DatabaseStorage::new(*storage_url.clone())?),
                "rekor" => Box::new(RekorStorage::new_with_url(*storage_url.clone())?),
                "local-fs" => Box::new(FilesystemStorage::new(storage_url.as_str())?),
                _ => return Err(Error::Validation("Invalid storage type".to_string())),
            };

            let updated_manifest =
                manifest::linking::link_dataset_to_model(&model_id, &dataset_id, storage.as_ref())?;

            println!("Successfully linked dataset {dataset_id} to model {model_id}");
            println!("Updated manifest ID: {}", updated_manifest.instance_id);

            Ok(())
        }
    }
}

pub fn handle_manifest_command(cmd: ManifestCommands) -> Result<()> {
    match cmd {
        ManifestCommands::Link {
            source,
            target,
            storage_type,
            storage_url,
        } => {
            let storage: Box<dyn StorageBackend> = match storage_type.as_str() {
                "database" => Box::new(DatabaseStorage::new(*storage_url.clone())?),
                "rekor" => Box::new(RekorStorage::new_with_url(*storage_url.clone())?),
                "local-fs" => Box::new(FilesystemStorage::new(storage_url.as_str())?),
                _ => return Err(Error::Validation("Invalid storage type".to_string())),
            };

            manifest::link_manifests(&source, &target, &*storage)
        }
        ManifestCommands::Show {
            id,
            storage_type,
            storage_url,
        } => {
            let storage: Box<dyn StorageBackend> = match storage_type.as_str() {
                "database" => Box::new(DatabaseStorage::new(*storage_url.clone())?),
                "rekor" => Box::new(RekorStorage::new_with_url(*storage_url.clone())?),
                "local-fs" => Box::new(FilesystemStorage::new(storage_url.as_str())?),
                _ => return Err(Error::Validation("Invalid storage type".to_string())),
            };

            manifest::show_manifest(&id, &*storage)
        }
        ManifestCommands::Validate {
            id,
            storage_type,
            storage_url,
        } => {
            let storage: Box<dyn StorageBackend> = match storage_type.as_str() {
                "database" => Box::new(DatabaseStorage::new(*storage_url.clone())?),
                "rekor" => Box::new(RekorStorage::new_with_url(*storage_url.clone())?),
                "local-fs" => Box::new(FilesystemStorage::new(storage_url.as_str())?),
                _ => return Err(Error::Validation("Invalid storage type".to_string())),
            };

            manifest::validate_linked_manifests(&id, &*storage)
        }
        ManifestCommands::VerifyLink {
            source,
            target,
            storage_type,
            storage_url,
        } => {
            let storage: Box<dyn StorageBackend> = match storage_type.as_str() {
                "database" => Box::new(DatabaseStorage::new(*storage_url.clone())?),
                "rekor" => Box::new(RekorStorage::new_with_url(*storage_url.clone())?),
                "local-fs" => Box::new(FilesystemStorage::new(storage_url.as_str())?),
                _ => return Err(Error::Validation("Invalid storage type".to_string())),
            };

            let result = manifest::verify_manifest_link(&source, &target, &*storage)?;
            if result {
                println!("Link verification successful");
                Ok(())
            } else {
                Err(Error::Validation("Link verification failed".to_string()))
            }
        }
        ManifestCommands::Export {
            id,
            storage_type,
            storage_url,
            format,
            output,
            max_depth,
        } => {
            let storage: Box<dyn StorageBackend> = match storage_type.as_str() {
                "database" => Box::new(DatabaseStorage::new(*storage_url.clone())?),
                "rekor" => Box::new(RekorStorage::new_with_url(*storage_url.clone())?),
                "local-fs" => Box::new(FilesystemStorage::new(storage_url.as_str())?),
                _ => return Err(Error::Validation("Invalid storage type".to_string())),
            };

            manifest::export_provenance(
                &id,
                &*storage,
                format.as_str(),
                output.as_deref(),
                max_depth,
            )
        }
    }
}

pub fn handle_evaluation_command(cmd: EvaluationCommands) -> Result<()> {
    match cmd {
        EvaluationCommands::Create {
            path,
            name,
            model_id,
            dataset_id,
            metrics,
            author_org,
            author_name,
            description,
            storage_type,
            storage_url,
            print,
            format,
            key,
            hash_alg,
        } => {
            let storage: Option<&'static dyn StorageBackend> = match storage_type.as_str() {
                "database" => {
                    let db_storage = Box::new(DatabaseStorage::new(*storage_url.clone())?);
                    Some(Box::leak(db_storage))
                }
                "rekor" => {
                    let rekor_storage = Box::new(RekorStorage::new_with_url(*storage_url.clone())?);
                    Some(Box::leak(rekor_storage))
                }
                "local-fs" => {
                    let fs_storage = Box::new(FilesystemStorage::new(storage_url.as_str())?);
                    Some(Box::leak(fs_storage))
                }
                _ => None,
            };

            let config = ManifestCreationConfig {
                paths: vec![path],
                ingredient_names: vec!["Evaluation Results".to_string()],
                name,
                author_org,
                author_name,
                description,
                linked_manifests: None, // Will be populated by create_manifest
                storage,
                print,
                output_encoding: format,
                key_path: key,
                hash_alg: hash_alg.to_cose_algorithm(),
                with_cc: false,
                software_type: None,
                version: None,
                custom_fields: None, // Will be populated by create_manifest
            };

            manifest::evaluation::create_manifest(config, model_id, dataset_id, metrics)
        }
        EvaluationCommands::List {
            storage_type,
            storage_url,
        } => {
            let storage: Box<dyn StorageBackend> = match storage_type.as_str() {
                "database" => Box::new(DatabaseStorage::new(*storage_url.clone())?),
                "rekor" => Box::new(RekorStorage::new_with_url(*storage_url.clone())?),
                "local-fs" => Box::new(FilesystemStorage::new(storage_url.as_str())?),
                _ => return Err(Error::Validation("Invalid storage type".to_string())),
            };

            manifest::evaluation::list_evaluation_manifests(storage.as_ref())
        }
        EvaluationCommands::Verify {
            id,
            storage_type,
            storage_url,
        } => {
            let storage: Box<dyn StorageBackend> = match storage_type.as_str() {
                "database" => Box::new(DatabaseStorage::new(*storage_url.clone())?),
                "rekor" => Box::new(RekorStorage::new_with_url(*storage_url.clone())?),
                "local-fs" => Box::new(FilesystemStorage::new(storage_url.as_str())?),
                _ => return Err(Error::Validation("Invalid storage type".to_string())),
            };

            manifest::evaluation::verify_evaluation_manifest(&id, storage.as_ref())
        }
    }
}

pub fn handle_cc_attestation_command(cmd: CCAttestationCommands) -> Result<()> {
    match cmd {
        CCAttestationCommands::Show => {
            let _r = cc_attestation::get_report(true).unwrap();
            Ok(())
        }

        CCAttestationCommands::GetLaunchMeasurement => {
            let m = cc_attestation::get_launch_measurement().unwrap();
            println!("Launch measurement raw bytes: 0x{}", hex::encode(m));
            Ok(())
        }

        CCAttestationCommands::VerifyLaunch { host_platform } => {
            let result = cc_attestation::verify_launch_endorsement(&host_platform).unwrap();
            if result {
                println!(
                    "Passed: launch endorsement verification for {host_platform} host platform"
                );
            } else {
                println!(
                    "Failed: launch endorsement verification for {host_platform} host platform"
                );
            }
            Ok(())
        }
    }
}

pub fn handle_software_command(cmd: SoftwareCommands) -> Result<()> {
    match cmd {
        SoftwareCommands::Create {
            paths,
            ingredient_names,
            name,
            software_type,
            version,
            author_org,
            author_name,
            description,
            linked_manifests,
            storage_type,
            storage_url,
            print,
            format,
            key,
            hash_alg,
            with_tdx,
        } => {
            let storage: Option<&'static dyn StorageBackend> = match storage_type.as_str() {
                "database" => {
                    let db_storage = Box::new(DatabaseStorage::new(*storage_url.clone())?);
                    Some(Box::leak(db_storage))
                }
                "rekor" => {
                    let rekor_storage = Box::new(RekorStorage::new_with_url(*storage_url.clone())?);
                    Some(Box::leak(rekor_storage))
                }
                "local-fs" => {
                    let fs_storage = Box::new(FilesystemStorage::new(storage_url.as_str())?);
                    Some(Box::leak(fs_storage))
                }
                _ => None,
            };

            let config = ManifestCreationConfig {
                paths,
                ingredient_names,
                name,
                author_org,
                author_name,
                description,
                linked_manifests,
                storage,
                print,
                output_encoding: format,
                key_path: key,
                hash_alg: hash_alg.to_cose_algorithm(),
                with_cc: with_tdx,
                software_type: Some(software_type.clone()),
                version: version.clone(),
                custom_fields: None,
            };

            manifest::software::create_manifest(config, software_type, version)
        }
        SoftwareCommands::List {
            storage_type,
            storage_url,
        } => {
            let storage: Box<dyn StorageBackend> = match storage_type.as_str() {
                "database" => Box::new(DatabaseStorage::new(*storage_url.clone())?),
                "rekor" => Box::new(RekorStorage::new_with_url(*storage_url.clone())?),
                "local-fs" => Box::new(FilesystemStorage::new(storage_url.as_str())?),
                _ => return Err(Error::Validation("Invalid storage type".to_string())),
            };

            manifest::software::list_software_manifests(storage.as_ref())
        }
        SoftwareCommands::Verify {
            id,
            storage_type,
            storage_url,
        } => {
            let storage: Box<dyn StorageBackend> = match storage_type.as_str() {
                "database" => Box::new(DatabaseStorage::new(*storage_url.clone())?),
                "rekor" => Box::new(RekorStorage::new_with_url(*storage_url.clone())?),
                "local-fs" => Box::new(FilesystemStorage::new(storage_url.as_str())?),
                _ => return Err(Error::Validation("Invalid storage type".to_string())),
            };

            manifest::software::verify_software_manifest(&id, storage.as_ref())
        }
        SoftwareCommands::LinkModel {
            software_id,
            model_id,
            storage_type,
            storage_url,
        } => {
            let storage: Box<dyn StorageBackend> = match storage_type.as_str() {
                "database" => Box::new(DatabaseStorage::new(*storage_url.clone())?),
                "rekor" => Box::new(RekorStorage::new_with_url(*storage_url.clone())?),
                "local-fs" => Box::new(FilesystemStorage::new(storage_url.as_str())?),
                _ => return Err(Error::Validation("Invalid storage type".to_string())),
            };

            // Link software to model
            manifest::link_manifests(&model_id, &software_id, storage.as_ref())
        }
        SoftwareCommands::LinkDataset {
            software_id,
            dataset_id,
            storage_type,
            storage_url,
        } => {
            let storage: Box<dyn StorageBackend> = match storage_type.as_str() {
                "database" => Box::new(DatabaseStorage::new(*storage_url.clone())?),
                "rekor" => Box::new(RekorStorage::new_with_url(*storage_url.clone())?),
                "local-fs" => Box::new(FilesystemStorage::new(storage_url.as_str())?),
                _ => return Err(Error::Validation("Invalid storage type".to_string())),
            };

            // Link software to dataset
            manifest::link_manifests(&dataset_id, &software_id, storage.as_ref())
        }
    }
}
