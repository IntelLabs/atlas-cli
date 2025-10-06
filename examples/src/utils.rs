use crate::error::{AtlasError, Result};
use colored::*;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Extract manifest ID from Atlas CLI output
pub fn extract_manifest_id(output: &str) -> Option<String> {
    let patterns = vec![
        r"Manifest stored successfully with ID: ([^\s]+)",
        r"Manifest ID: ([^\s]+)",
        r"ID: ([^\s]+)",
        r"Created manifest: ([^\s]+)",
        r"stored with id: ([^\s]+)",
        r"Updated manifest ID: ([^\s]+)",
    ];

    for pattern in patterns {
        let re = Regex::new(pattern).ok()?;
        if let Some(captures) = re.captures(output) {
            if let Some(id) = captures.get(1) {
                let manifest_id = id.as_str().trim().to_string();
                debug!("Extracted manifest ID: {}", manifest_id);
                return Some(manifest_id);
            }
        }
    }

    debug!(
        "Could not extract ID from output: {}",
        &output[..500.min(output.len())]
    );
    None
}

/// Resolve path with special prefixes
pub fn resolve_path(base_dir: &Path, path: &str, shared_dir: Option<&Path>) -> PathBuf {
    if path.is_empty() {
        return base_dir.to_path_buf();
    }

    // Handle special prefixes
    if let Some(shared) = shared_dir {
        if path.starts_with("@shared/") {
            return shared.join(&path[8..]);
        }
    }

    if path.starts_with("@example/") {
        return base_dir.join(&path[9..]);
    }

    if path.starts_with("./") {
        return base_dir
            .join(path)
            .canonicalize()
            .unwrap_or_else(|_| base_dir.join(path));
    }

    if path.starts_with("/") {
        return PathBuf::from(path);
    }

    // Default to relative to base directory
    base_dir.join(path)
}

/// Resolve variable references in text
pub fn resolve_variables(text: &str, variables: &HashMap<String, String>) -> String {
    let mut result = text.to_string();

    // Pattern for ${VARIABLE_NAME}
    let re = Regex::new(r"\$\{([^}]+)\}").unwrap();

    for captures in re.captures_iter(text) {
        if let Some(var_name) = captures.get(1) {
            let var_key = var_name.as_str();
            if let Some(value) = variables.get(var_key) {
                let pattern = format!("${{{}}}", var_key);
                result = result.replace(&pattern, value);
            }
        }
    }

    result
}

/// Recursively resolve variables in any JSON value
pub fn resolve_value(
    value: serde_json::Value,
    variables: &HashMap<String, String>,
) -> serde_json::Value {
    match value {
        serde_json::Value::String(s) => serde_json::Value::String(resolve_variables(&s, variables)),
        serde_json::Value::Object(mut map) => {
            for (_, v) in map.iter_mut() {
                *v = resolve_value(v.take(), variables);
            }
            serde_json::Value::Object(map)
        }
        serde_json::Value::Array(mut arr) => {
            for item in arr.iter_mut() {
                *item = resolve_value(item.take(), variables);
            }
            serde_json::Value::Array(arr)
        }
        other => other,
    }
}

/// Setup logging with colored output
pub fn setup_logging(verbose: bool) {
    let level = if verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .init();
}

/// Print colored command
pub fn print_command(cmd: &str) {
    println!("{} {}", "$".blue().bold(), cmd);
}

/// Print step header
pub fn print_step_header(step_num: usize, total: usize, name: &str, description: Option<&str>) {
    println!(
        "\n{} Step {}/{}: {}",
        "▶️".green(),
        step_num,
        total,
        name.bold()
    );

    if let Some(desc) = description {
        println!("   {}", desc.dimmed());
    }
}

/// Print success message
pub fn print_success(message: &str) {
    println!("   {} {}", "✅".green(), message.green());
}

/// Print error message
pub fn print_error(message: &str) {
    println!("   {} {}", "❌".red(), message.red());
}

/// Print warning message
pub fn print_warning(message: &str) {
    println!("   {} {}", "⚠️".yellow(), message.yellow());
}

/// Print info message
pub fn print_info(message: &str) {
    println!("   {} {}", "ℹ️".blue(), message);
}

/// Check if Atlas CLI is available
pub fn check_atlas_cli() -> Result<String> {
    let output = std::process::Command::new("atlas-cli")
        .arg("--version")
        .output()
        .map_err(|_| AtlasError::AtlasCliNotFound)?;

    if !output.status.success() {
        return Err(AtlasError::AtlasCliNotFound);
    }

    let version = String::from_utf8_lossy(&output.stdout);
    Ok(version.trim().to_string())
}

/// Create directory if it doesn't exist
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Find project root by looking for pyproject.toml or Cargo.toml
pub fn find_project_root(start: &Path) -> PathBuf {
    let mut current = start.to_path_buf();

    while current.parent().is_some() {
        if current.join("pyproject.toml").exists() || current.join("Cargo.toml").exists() {
            return current;
        }
        current = current.parent().unwrap().to_path_buf();
    }

    start.to_path_buf()
}

/// Generate signing keys using OpenSSL
pub async fn generate_signing_keys(private_key_path: &Path, public_key_path: &Path) -> Result<()> {
    use tokio::process::Command;

    // Ensure directory exists
    if let Some(parent) = private_key_path.parent() {
        ensure_dir_exists(parent)?;
    }
    if let Some(parent) = public_key_path.parent() {
        ensure_dir_exists(parent)?;
    }

    // Generate private key
    let output = Command::new("openssl")
        .args(&["genpkey", "-algorithm", "RSA", "-out"])
        .arg(private_key_path)
        .args(&["-pkeyopt", "rsa_keygen_bits:4096"])
        .output()
        .await?;

    if !output.status.success() {
        return Err(AtlasError::CommandError(format!(
            "Failed to generate private key: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    // Extract public key
    let output = Command::new("openssl")
        .args(&["rsa", "-pubout", "-in"])
        .arg(private_key_path)
        .arg("-out")
        .arg(public_key_path)
        .output()
        .await?;

    if !output.status.success() {
        return Err(AtlasError::CommandError(format!(
            "Failed to extract public key: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Parse key-value pairs from a string (e.g., "key1=value1,key2=value2")
pub fn parse_key_value_pairs(s: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for pair in s.split(',') {
        if let Some(eq_pos) = pair.find('=') {
            let key = pair[..eq_pos].trim().to_string();
            let value = pair[eq_pos + 1..].trim().to_string();
            map.insert(key, value);
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_extract_manifest_id() {
        let output = "Manifest stored successfully with ID: abc123def456";
        assert_eq!(
            extract_manifest_id(output),
            Some("abc123def456".to_string())
        );

        let output = "Random text\nID: xyz789\nMore text";
        assert_eq!(extract_manifest_id(output), Some("xyz789".to_string()));

        let output = "No manifest ID here";
        assert_eq!(extract_manifest_id(output), None);
    }

    #[test]
    fn test_resolve_path() {
        let base_dir = Path::new("/home/user/project");
        let shared_dir = Path::new("/home/user/shared");

        assert_eq!(
            resolve_path(base_dir, "@shared/data.csv", Some(shared_dir)),
            PathBuf::from("/home/user/shared/data.csv")
        );

        assert_eq!(
            resolve_path(base_dir, "@example/test.yaml", None),
            PathBuf::from("/home/user/project/test.yaml")
        );

        assert_eq!(
            resolve_path(base_dir, "./subdir/file.txt", None),
            PathBuf::from("/home/user/project/subdir/file.txt")
        );

        assert_eq!(
            resolve_path(base_dir, "/absolute/path.txt", None),
            PathBuf::from("/absolute/path.txt")
        );
    }

    #[test]
    fn test_resolve_variables() {
        let mut vars = HashMap::new();
        vars.insert("MODEL_ID".to_string(), "model_123".to_string());
        vars.insert("DATASET_ID".to_string(), "data_456".to_string());

        let text = "Using model ${MODEL_ID} with dataset ${DATASET_ID}";
        let resolved = resolve_variables(text, &vars);
        assert_eq!(resolved, "Using model model_123 with dataset data_456");

        let text = "No variables here";
        let resolved = resolve_variables(text, &vars);
        assert_eq!(resolved, "No variables here");
    }

    #[test]
    fn test_parse_key_value_pairs() {
        let input = "key1=value1,key2=value2,key3=value3";
        let map = parse_key_value_pairs(input);

        assert_eq!(map.get("key1"), Some(&"value1".to_string()));
        assert_eq!(map.get("key2"), Some(&"value2".to_string()));
        assert_eq!(map.get("key3"), Some(&"value3".to_string()));
    }
}
