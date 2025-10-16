use crate::{
    config::Step,
    error::{AtlasError, Result},
    framework::AtlasTestFramework,
};
use std::fs;

pub fn execute_shell_command(
    framework: &mut AtlasTestFramework,
    step: &Step,
) -> Result<Option<String>> {
    let params = &step.parameters;

    let command = params
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AtlasError::MissingField("command".to_string()))?;

    let check = params
        .get("check")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // Execute the command
    let result = framework.run_command(command, check)?;
    let stdout = String::from_utf8_lossy(&result.stdout);

    // Capture output if requested
    if let Some(capture_as) = params.get("capture_as").and_then(|v| v.as_str()) {
        let value = stdout.trim().to_string();
        framework
            .variables
            .insert(capture_as.to_string(), value.clone());
        tracing::info!("   📌 Captured as: {}", capture_as);
    }

    Ok(Some(stdout.to_string()))
}

pub fn execute_file_action(
    framework: &mut AtlasTestFramework,
    action: &str,
    step: &Step,
) -> Result<Option<String>> {
    match action {
        "tamper" => tamper_file(framework, step),
        "copy" => copy_file(framework, step),
        "delete" => delete_file(framework, step),
        "create" => create_file(framework, step),
        _ => Err(AtlasError::UnknownAction(format!("file:{}", action))),
    }
}

fn tamper_file(framework: &mut AtlasTestFramework, step: &Step) -> Result<Option<String>> {
    let params = &step.parameters;

    let file_path = params
        .get("file")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AtlasError::MissingField("file".to_string()))?;

    let file_path = framework.resolve_path(file_path);

    if !file_path.exists() {
        return Err(AtlasError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", file_path.display()),
        )));
    }

    tracing::info!("   ⚠️  Tampering with: {}", file_path.display());

    // Determine how to tamper based on file extension
    let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match extension {
        "txt" | "csv" | "json" | "yaml" | "yml" | "xml" | "py" | "rs" | "sh" => {
            // Text file - append some data
            let mut content = fs::read_to_string(&file_path)?;
            content.push_str("\n# TAMPERED DATA\n");
            fs::write(&file_path, content)?;
        }
        _ => {
            // Binary file - flip first byte
            let mut content = fs::read(&file_path)?;
            if !content.is_empty() {
                content[0] ^= 0xFF;
            }
            fs::write(&file_path, content)?;
        }
    }

    tracing::info!("   ✔ File tampered");
    Ok(Some("tampered".to_string()))
}

fn copy_file(framework: &mut AtlasTestFramework, step: &Step) -> Result<Option<String>> {
    let params = &step.parameters;

    let source = params
        .get("source")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AtlasError::MissingField("source".to_string()))?;

    let destination = params
        .get("destination")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AtlasError::MissingField("destination".to_string()))?;

    let source_path = framework.resolve_path(source);
    let dest_path = framework.resolve_path(destination);

    // Ensure destination directory exists
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(&source_path, &dest_path)?;

    tracing::info!(
        "   ✔ Copied {} to {}",
        source_path.display(),
        dest_path.display()
    );
    Ok(Some("copied".to_string()))
}

fn delete_file(framework: &mut AtlasTestFramework, step: &Step) -> Result<Option<String>> {
    let params = &step.parameters;

    let file_path = params
        .get("file")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AtlasError::MissingField("file".to_string()))?;

    let file_path = framework.resolve_path(file_path);

    if file_path.exists() {
        if file_path.is_dir() {
            fs::remove_dir_all(&file_path)?;
        } else {
            fs::remove_file(&file_path)?;
        }
        tracing::info!("   ✔ Deleted: {}", file_path.display());
    } else {
        tracing::warn!("   ⚠️  File not found: {}", file_path.display());
    }

    Ok(Some("deleted".to_string()))
}

fn create_file(framework: &mut AtlasTestFramework, step: &Step) -> Result<Option<String>> {
    let params = &step.parameters;

    let file_path = params
        .get("file")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AtlasError::MissingField("file".to_string()))?;

    let content = params.get("content").and_then(|v| v.as_str()).unwrap_or("");

    let file_path = framework.resolve_path(file_path);

    // Ensure directory exists
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&file_path, content)?;

    tracing::info!("   ✔ Created: {}", file_path.display());
    Ok(Some("created".to_string()))
}
