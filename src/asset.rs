use c2pa_ml::asset_type::AssetType;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct AssetInfo {
    pub asset_type: AssetType,
    pub digital_source_type: String,
    pub format: String,
    pub media_type: String,
}

lazy_static! {
    static ref ASSET_MAPPINGS: HashMap<&'static str, AssetInfo> = {

        // Add mappings as before...
        HashMap::new()
    };
}

pub fn get_asset_info(path: &Path) -> Option<AssetInfo> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| ASSET_MAPPINGS.get(ext))
        .cloned()
}
