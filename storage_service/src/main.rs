use actix_web::{web, App, HttpResponse, HttpServer};
use mongodb::{Client, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use log::{debug, error, info}; 

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestEntry {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<mongodb::bson::oid::ObjectId>,
    manifest_id: String,
    manifest_type: String, // "model" or "dataset"
    manifest: serde_json::Value,
    created_at: String,
}

// Store manifest
async fn store_manifest(
    state: web::Data<AppState>,
    manifest: web::Json<serde_json::Value>,
    path: web::Path<String>,
) -> HttpResponse {
    let collection = state.db.collection::<ManifestEntry>("manifests");

    debug!("Received manifest: {}", serde_json::to_string_pretty(&manifest).unwrap_or_default());
    
    let manifest_type = manifest.get("manifest")
        .and_then(|m| m.get("manifest_type"))
        .or_else(|| manifest.get("manifest_type"))
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| {
            debug!("Could not find manifest_type in expected locations");
            "unknown"
        })
        .to_string();
    
    info!("Extracted manifest_type: {}", manifest_type);

    let entry = ManifestEntry {
        id: None,
        manifest_id: path.into_inner(),
        manifest_type,
        manifest: manifest.into_inner(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    match collection.insert_one(entry, None).await {
        Ok(result) => {
            info!("Successfully stored manifest with ID: {}", result.inserted_id);
            HttpResponse::Created().json(result.inserted_id)
        }
        Err(e) => {
            error!("Failed to store manifest: {:?}", e);
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

// List manifests
async fn list_manifests(state: web::Data<AppState>) -> HttpResponse {
    let collection = state.db.collection::<ManifestEntry>("manifests");
    
    match collection.find(None, None).await {
        Ok(cursor) => {
            match futures::stream::TryStreamExt::try_collect::<Vec<_>>(cursor).await {
                Ok(manifests) => HttpResponse::Ok().json(manifests),
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// Get manifest by ID
async fn get_manifest(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let collection = state.db.collection::<ManifestEntry>("manifests");
    debug!("Searching for manifest with ID: {}", &*path);

    match collection
        .find_one(
            mongodb::bson::doc! { "manifest_id": &*path },
            None,
        )
        .await
    {
        Ok(Some(manifest)) => {
            info!("Found manifest for ID: {}", &*path);
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


// Delete manifest
async fn delete_manifest(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let collection = state.db.collection::<ManifestEntry>("manifests");
    
    match collection
        .delete_one(mongodb::bson::doc! { "manifest_id": &*path }, None)
        .await
    {
        Ok(result) if result.deleted_count > 0 => HttpResponse::Ok().body("Manifest deleted"),
        Ok(_) => HttpResponse::NotFound().body("Manifest not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let mongodb_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    
    let client = Client::with_uri_str(&mongodb_uri)
        .await
        .expect("Failed to connect to MongoDB");
    
    let db = Arc::new(client.database("c2pa_manifests"));
    let state = web::Data::new(AppState { db });

    println!("Starting server at http://localhost:8080");
    
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/manifests", web::get().to(list_manifests))
            .route("/manifests/{id}", web::post().to(store_manifest))
            .route("/manifests/{id}", web::get().to(get_manifest))
            .route("/manifests/{id}", web::delete().to(delete_manifest))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}