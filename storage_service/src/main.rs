use actix_web::{App, HttpRequest, HttpResponse, HttpServer, http::header, web};
use base64::{Engine as _, engine::general_purpose};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use log::{debug, error, info};
use mongodb::{Client, Database};
use ring::digest::{Context, SHA256};
use ring::signature::Ed25519KeyPair;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ContentFormat {
    #[serde(rename = "json")]
    JSON,
    #[serde(rename = "cbor")]
    CBOR,
    #[serde(rename = "binary")]
    Binary,
}

// Default to json
impl Default for ContentFormat {
    fn default() -> Self {
        ContentFormat::JSON
    }
}

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
    key_pair: Arc<Ed25519KeyPair>,
    merkle_tree: Arc<parking_lot::RwLock<MerkleTree>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestEntry {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub manifest_id: String,
    pub manifest_type: String,
    pub content_format: ContentFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_json: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_cbor: Option<String>, // Base64 encoded CBOR
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_binary: Option<String>, // Base64 encoded binary
    pub created_at: DateTime<Utc>,
    pub sequence_number: u64,
    pub hash: String,
    pub signature: String,
}

// Function to detect content type from request
pub fn detect_content_type(req: &HttpRequest) -> ContentFormat {
    if let Some(content_type) = req.headers().get(header::CONTENT_TYPE) {
        match content_type.to_str() {
            Ok(ct) => {
                if ct.contains("application/cbor") {
                    return ContentFormat::CBOR;
                } else if ct.contains("application/octet-stream") {
                    return ContentFormat::Binary;
                }
            }
            Err(_) => {}
        }
    }
    // Default to JSON
    ContentFormat::JSON
}

// Hash binary data
pub fn hash_binary(data: &[u8]) -> String {
    let mut context = Context::new(&SHA256);
    context.update(data);
    let digest = context.finish();
    general_purpose::STANDARD.encode(digest.as_ref())
}

// Sign binary data
pub fn sign_data(key_pair: &Ed25519KeyPair, data: &[u8]) -> String {
    let signature = key_pair.sign(data);
    general_purpose::STANDARD.encode(signature.as_ref())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LogLeaf {
    manifest_id: String,
    hash: String,
    sequence_number: u64,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MerkleTree {
    leaves: Vec<LogLeaf>,
    root_hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct InclusionProof {
    manifest_id: String,
    leaf_hash: String,
    merkle_path: Vec<String>,
    root_hash: String,
}

impl MerkleTree {
    fn new() -> Self {
        MerkleTree {
            leaves: Vec::new(),
            root_hash: None,
        }
    }

    fn add_leaf(&mut self, leaf: LogLeaf) {
        self.leaves.push(leaf);
        self.update_root_hash();
    }

    fn update_root_hash(&mut self) {
        if self.leaves.is_empty() {
            self.root_hash = None;
            return;
        }

        let mut hashes: Vec<String> = self.leaves.iter().map(|leaf| leaf.hash.clone()).collect();

        while hashes.len() > 1 {
            let mut new_hashes = Vec::new();

            for chunk in hashes.chunks(2) {
                if chunk.len() == 2 {
                    let combined = format!("{}{}", chunk[0], chunk[1]);
                    new_hashes.push(hash_string(&combined));
                } else {
                    new_hashes.push(chunk[0].clone());
                }
            }

            hashes = new_hashes;
        }

        self.root_hash = Some(hashes[0].clone());
    }

    fn generate_inclusion_proof(&self, manifest_id: &str) -> Option<InclusionProof> {
        if self.leaves.is_empty() || self.root_hash.is_none() {
            return None;
        }

        let position = self
            .leaves
            .iter()
            .position(|leaf| leaf.manifest_id == manifest_id)?;
        let leaf_hash = self.leaves[position].hash.clone();
        let mut merkle_path = Vec::new();
        let mut current_pos = position;

        // Calculate the merkle path going up the tree
        let mut level_size = self.leaves.len();
        let mut level_hashes: Vec<String> =
            self.leaves.iter().map(|leaf| leaf.hash.clone()).collect();

        while level_size > 1 {
            // Find sibling position
            let sibling_pos = if current_pos % 2 == 0 {
                current_pos + 1 // Right sibling
            } else {
                current_pos - 1 // Left sibling
            };

            // Add sibling hash to the proof path if it exists
            if sibling_pos < level_size {
                merkle_path.push(level_hashes[sibling_pos].clone());
            }

            // Move to the parent level
            current_pos /= 2;

            // Calculate the parent level hashes
            let mut new_level_hashes = Vec::new();
            for i in (0..level_size).step_by(2) {
                if i + 1 < level_size {
                    // If we have a pair, hash them together
                    let combined = format!("{}{}", level_hashes[i], level_hashes[i + 1]);
                    new_level_hashes.push(hash_string(&combined));
                } else {
                    // If we have an odd node at the end, it's carried up without changes
                    new_level_hashes.push(level_hashes[i].clone());
                }
            }

            level_hashes = new_level_hashes;
            level_size = level_hashes.len();
        }

        Some(InclusionProof {
            manifest_id: manifest_id.to_string(),
            leaf_hash,
            merkle_path,
            root_hash: self.root_hash.clone().unwrap(),
        })
    }

    fn verify_inclusion_proof(&self, proof: &InclusionProof) -> bool {
        // Start with the leaf hash
        let mut current_hash = proof.leaf_hash.clone();

        // Get the position of the leaf in the tree (if it exists)
        if let Some(position) = self
            .leaves
            .iter()
            .position(|leaf| leaf.manifest_id == proof.manifest_id)
        {
            // Calculate sibling positions and combine hashes according to position
            let mut level_pos = position;

            for sibling_hash in &proof.merkle_path {
                // Determine if the current node is left or right in its pair
                let is_left = level_pos % 2 == 0;

                // Combine hashes in the correct order based on position
                if is_left {
                    // If we're the left node, combine: current + sibling
                    let combined = format!("{}{}", current_hash, sibling_hash);
                    current_hash = hash_string(&combined);
                } else {
                    // If we're the right node, combine: sibling + current
                    let combined = format!("{}{}", sibling_hash, current_hash);
                    current_hash = hash_string(&combined);
                }

                // Move to parent level
                level_pos /= 2;
            }

            // Final hash should match the root hash
            return current_hash == proof.root_hash;
        }

        false
    }
}

fn hash_string(data: &str) -> String {
    let mut context = Context::new(&SHA256);
    context.update(data.as_bytes());
    let digest = context.finish();
    general_purpose::STANDARD.encode(digest.as_ref())
}

// Store manifest with content type support
async fn store_manifest(
    state: web::Data<AppState>,
    req: HttpRequest,
    bytes: Bytes, // Accept raw bytes instead of JSON
    path: web::Path<String>,
    query: web::Query<ManifestQuery>,
) -> HttpResponse {
    let collection = state.db.collection::<ManifestEntry>("manifests");
    let manifest_type_param = &query.manifest_type;

    debug!(
        "Received manifest with ID: {}, manifest_type param: {:?}",
        &*path, manifest_type_param
    );

    // Detect content format
    let content_format = detect_content_type(&req);

    // Create hash and signature from raw content
    let hash = hash_binary(&bytes);
    let signature = sign_data(&state.key_pair, &hash.as_bytes());

    // Get next sequence number
    let sequence_count = collection.count_documents(None, None).await.unwrap_or(0);
    let sequence_number = sequence_count + 1;

    let manifest_id = path.to_string();
    let now = Utc::now();

    // Default manifest type from query parameter or "unknown"
    let manifest_type = manifest_type_param
        .as_ref()
        .map(|s| s.clone())
        .unwrap_or_else(|| "unknown".to_string());

    // Build the manifest entry based on content type
    let mut entry = ManifestEntry {
        id: None,
        manifest_id: manifest_id.clone(),
        manifest_type,
        content_format: content_format.clone(),
        manifest_json: None,
        manifest_cbor: None,
        manifest_binary: None,
        created_at: now,
        sequence_number: sequence_number as u64,
        hash: hash.clone(), // Clone the hash so wt can be used later
        signature,
    };

    match content_format {
        ContentFormat::JSON => {
            match serde_json::from_slice::<serde_json::Value>(&bytes) {
                Ok(json_value) => {
                    // Extract manifest_type from JSON
                    let json_manifest_type = json_value
                        .get("manifest")
                        .and_then(|m| m.get("manifest_type"))
                        .or_else(|| json_value.get("manifest_type"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    // Use the manifest_type from JSON if found and not overridden by query parameter
                    if let Some(mt) = json_manifest_type {
                        if manifest_type_param.is_none() {
                            entry.manifest_type = mt;
                        }
                    }

                    debug!("Using manifest_type: {}", entry.manifest_type);
                    entry.manifest_json = Some(json_value);
                }
                Err(e) => {
                    error!("Failed to parse JSON: {:?}", e);
                    return HttpResponse::BadRequest().body(format!("Invalid JSON format: {}", e));
                }
            }
        }
        ContentFormat::CBOR => {
            // Store as base64 encoded string
            let encoded = general_purpose::STANDARD.encode(&bytes);
            entry.manifest_cbor = Some(encoded);

            // Try to decode CBOR to extract manifest_type if possible
            match serde_cbor::from_slice::<serde_json::Value>(&bytes) {
                Ok(cbor_value) => {
                    let cbor_manifest_type = cbor_value
                        .get("manifest_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    // Use the manifest_type from CBOR if found and not overridden by query parameter
                    if let Some(mt) = cbor_manifest_type {
                        if manifest_type_param.is_none() {
                            entry.manifest_type = mt;
                        }
                    } else if manifest_type_param.is_none() {
                        entry.manifest_type = "cbor_manifest".to_string();
                    }
                }
                Err(e) => {
                    debug!("Could not extract manifest_type from CBOR: {:?}", e);
                    if manifest_type_param.is_none() {
                        entry.manifest_type = "cbor_manifest".to_string();
                    }
                }
            }
        }
        ContentFormat::Binary => {
            // Store as base64 encoded string
            let encoded = general_purpose::STANDARD.encode(&bytes);
            entry.manifest_binary = Some(encoded);

            // Use the manifest_type from query parameter or default
            if manifest_type_param.is_none() {
                entry.manifest_type = "binary_manifest".to_string();
            }
        }
    }

    match collection.insert_one(entry, None).await {
        Ok(result) => {
            info!(
                "Successfully stored manifest with ID: {}",
                result.inserted_id
            );

            // Add to Merkle Tree
            let leaf = LogLeaf {
                manifest_id: manifest_id,
                hash,
                sequence_number: sequence_number as u64,
                timestamp: now,
            };

            // Update the Merkle tree
            {
                let mut tree = state.merkle_tree.write();
                tree.add_leaf(leaf.clone());

                // Persist the updated Merkle tree to the database
                if let Err(e) = persist_merkle_tree(&state.db, &tree).await {
                    error!("Failed to persist Merkle tree: {:?}", e);
                }
            }

            HttpResponse::Created().json(result.inserted_id)
        }
        Err(e) => {
            error!("Failed to store manifest: {:?}", e);
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}
async fn persist_merkle_tree(
    db: &Database,
    tree: &MerkleTree,
) -> Result<(), mongodb::error::Error> {
    let collection = db.collection::<MerkleTree>("merkle_tree");

    // Clear existing tree
    collection.delete_many(mongodb::bson::doc! {}, None).await?;

    // Insert new tree
    collection.insert_one(tree, None).await?;

    Ok(())
}

async fn load_merkle_tree(db: &Database) -> MerkleTree {
    let collection = db.collection::<MerkleTree>("merkle_tree");

    match collection.find_one(None, None).await {
        Ok(Some(tree)) => tree,
        _ => {
            // If no tree exists or error occurs, create a new one
            let tree = MerkleTree::new();

            // Attempt to rebuild from existing manifests
            let manifests_collection = db.collection::<ManifestEntry>("manifests");
            if let Ok(cursor) = manifests_collection.find(None, None).await {
                if let Ok(manifests) =
                    futures::stream::TryStreamExt::try_collect::<Vec<_>>(cursor).await
                {
                    let mut new_tree = MerkleTree::new();

                    for manifest in manifests {
                        let leaf = LogLeaf {
                            manifest_id: manifest.manifest_id,
                            hash: manifest.hash,
                            sequence_number: manifest.sequence_number,
                            timestamp: manifest.created_at,
                        };
                        new_tree.add_leaf(leaf);
                    }

                    return new_tree;
                }
            }

            tree
        }
    }
}

// List manifests with pagination
async fn list_manifests(state: web::Data<AppState>, query: web::Query<ListQuery>) -> HttpResponse {
    let collection = state.db.collection::<ManifestEntry>("manifests");

    let limit = query.limit.unwrap_or(100) as i64;
    let skip = query.skip.unwrap_or(0) as u64;

    // Build filter document based on query parameters see documentation
    let mut filter = mongodb::bson::Document::new();

    if let Some(manifest_type) = &query.manifest_type {
        filter.insert("manifest_type", manifest_type);
    }

    if let Some(format) = &query.format {
        let content_format = match format.as_str() {
            "json" => "JSON",
            "cbor" => "CBOR",
            "binary" => "Binary",
            _ => "JSON",
        };
        filter.insert("content_format", content_format);
    }

    let find_options = mongodb::options::FindOptions::builder()
        .sort(mongodb::bson::doc! { "sequence_number": 1 })
        .skip(skip)
        .limit(limit)
        .build();

    let filter_doc = if filter.is_empty() {
        None
    } else {
        Some(filter)
    };

    match collection.find(filter_doc, find_options).await {
        Ok(cursor) => match futures::stream::TryStreamExt::try_collect::<Vec<_>>(cursor).await {
            Ok(manifests) => HttpResponse::Ok().json(manifests),
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// Query parameters for manifest operations
#[derive(Debug, Deserialize)]
struct ManifestQuery {
    manifest_type: Option<String>,
}

// Enhanced listing query parameters
#[derive(Debug, Deserialize)]
struct ListQuery {
    limit: Option<usize>,
    skip: Option<u64>,
    manifest_type: Option<String>,
    format: Option<String>,
}

// List manifests by type
async fn list_manifests_by_type(
    state: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<ListQuery>,
) -> HttpResponse {
    let manifest_type = path.into_inner();
    let collection = state.db.collection::<ManifestEntry>("manifests");

    let limit = query.limit.unwrap_or(100) as i64;
    let skip = query.skip.unwrap_or(0) as u64;

    let filter = mongodb::bson::doc! { "manifest_type": manifest_type };

    let find_options = mongodb::options::FindOptions::builder()
        .sort(mongodb::bson::doc! { "sequence_number": 1 })
        .skip(skip)
        .limit(limit)
        .build();

    match collection.find(filter, find_options).await {
        Ok(cursor) => match futures::stream::TryStreamExt::try_collect::<Vec<_>>(cursor).await {
            Ok(manifests) => HttpResponse::Ok().json(manifests),
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// Get manifest by ID
async fn get_manifest(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    let collection = state.db.collection::<ManifestEntry>("manifests");
    debug!("Searching for manifest with ID: {}", &*path);

    match collection
        .find_one(mongodb::bson::doc! { "manifest_id": &*path }, None)
        .await
    {
        Ok(Some(manifest)) => {
            info!("Found manifest for ID: {}", &*path);

            // Check Accept header for content negotiation
            let accept_cbor = req
                .headers()
                .get(header::ACCEPT)
                .and_then(|h| h.to_str().ok())
                .map(|s| s.contains("application/cbor"))
                .unwrap_or(false);

            // Return appropriate format based on what's available and what's requested
            match manifest.content_format {
                ContentFormat::CBOR if accept_cbor => {
                    if let Some(ref cbor_data) = manifest.manifest_cbor {
                        if let Ok(decoded) = general_purpose::STANDARD.decode(cbor_data) {
                            return HttpResponse::Ok()
                                .content_type("application/cbor")
                                .body(decoded);
                        }
                    }
                }
                ContentFormat::Binary => {
                    if let Some(ref binary_data) = manifest.manifest_binary {
                        if let Ok(decoded) = general_purpose::STANDARD.decode(binary_data) {
                            return HttpResponse::Ok()
                                .content_type("application/octet-stream")
                                .body(decoded);
                        }
                    }
                }
                _ => {} // default to JSON response it maybe naive
            }

            // Default: return as JSON
            HttpResponse::Ok().json(manifest)
        }
        Ok(None) => {
            debug!("No manifest found for ID: {}", &*path);
            HttpResponse::NotFound().body(format!("Manifest not found for ID: {}", &*path))
        }
        Err(e) => {
            error!("Error fetching manifest {}: {:?}", &*path, e);
            HttpResponse::InternalServerError().body(format!("Error fetching manifest: {}", e))
        }
    }
}

// Get inclusion proof for a manifest
async fn get_inclusion_proof(state: web::Data<AppState>, path: web::Path<String>) -> HttpResponse {
    let manifest_id = path.into_inner();

    let tree = state.merkle_tree.read();
    match tree.generate_inclusion_proof(&manifest_id) {
        Some(proof) => HttpResponse::Ok().json(proof),
        None => HttpResponse::NotFound()
            .body(format!("No proof available for manifest: {}", manifest_id)),
    }
}

// Get latest Merkle root
async fn get_merkle_root(state: web::Data<AppState>) -> HttpResponse {
    let tree = state.merkle_tree.read();
    match &tree.root_hash {
        Some(root) => HttpResponse::Ok().json(serde_json::json!({
            "root_hash": root,
            "tree_size": tree.leaves.len()
        })),
        None => HttpResponse::NotFound().body("No Merkle root available yet"),
    }
}

// Verify an inclusion proof
async fn verify_proof(
    state: web::Data<AppState>,
    proof: web::Json<InclusionProof>,
) -> HttpResponse {
    let tree = state.merkle_tree.read();
    let is_valid = tree.verify_inclusion_proof(&proof);

    HttpResponse::Ok().json(serde_json::json!({
        "valid": is_valid
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Get MongoDB URI from environment variable or use default
    let mongodb_uri =
        std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

    // Get server host and port from environment variables or use defaults
    let server_host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let server_port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());

    // Combine host and port
    let server_addr = format!("{}:{}", server_host, server_port);

    // Generate or load keys
    let key_path =
        std::env::var("KEY_PATH").unwrap_or_else(|_| "transparency_log_key.pem".to_string());
    let key_pair = match std::fs::read(&key_path) {
        Ok(pkcs8_bytes) => Ed25519KeyPair::from_pkcs8(&pkcs8_bytes).expect("Failed to parse key"),
        Err(_) => {
            // Generate new key
            let rng = ring::rand::SystemRandom::new();
            let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).expect("Failed to generate key");
            std::fs::write(&key_path, pkcs8_bytes.as_ref()).expect("Failed to save key");
            Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
                .expect("Failed to parse newly generated key")
        }
    };

    let client = Client::with_uri_str(&mongodb_uri)
        .await
        .expect("Failed to connect to MongoDB");

    // Configurable database name
    let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "c2pa_manifests".to_string());

    let db = Arc::new(client.database(&db_name));

    // Load Merkle Tree from database or create new one
    let merkle_tree = Arc::new(parking_lot::RwLock::new(load_merkle_tree(&db).await));

    let state = web::Data::new(AppState {
        db: db.clone(),
        key_pair: Arc::new(key_pair),
        merkle_tree,
    });

    println!(
        "Starting transparency log server at http://{}:{}",
        if server_host == "0.0.0.0" {
            "localhost"
        } else {
            &server_host
        },
        server_port
    );

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .app_data(web::PayloadConfig::new(10 * 1024 * 1024)) // It max 10MB
            .route("/manifests", web::get().to(list_manifests))
            .route("/manifests/{id}", web::post().to(store_manifest))
            .route("/manifests/{id}", web::get().to(get_manifest))
            .route("/manifests/{id}/proof", web::get().to(get_inclusion_proof))
            .route("/merkle/root", web::get().to(get_merkle_root))
            .route("/merkle/verify", web::post().to(verify_proof))
            .route(
                "/types/{manifest_type}/manifests",
                web::get().to(list_manifests_by_type),
            )
    })
    .bind(&server_addr)?
    .run()
    .await
}

// Include the test module
#[cfg(test)]
mod tests;
