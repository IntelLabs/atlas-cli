use crate::{
    command::AtlasCommand,
    config::Step,
    error::{AtlasError, Result},
    framework::AtlasTestFramework,
    utils::extract_manifest_id,
};

pub fn execute_manifest_action(
    framework: &mut AtlasTestFramework,
    action: &str,
    step: &Step,
) -> Result<Option<String>> {
    match action {
        "link" => link_manifests(framework, step),
        "validate" => validate_manifest(framework, step),
        "verify" => verify_manifest(framework, step),
        "show" => show_manifest(framework, step),
        "export" => export_manifest(framework, step),
        "list" => list_manifests(framework, step),
        _ => Err(AtlasError::UnknownAction(format!("manifest:{}", action))),
    }
}

fn link_manifests(framework: &mut AtlasTestFramework, step: &Step) -> Result<Option<String>> {
    let params = &step.parameters;

    let mut cmd = AtlasCommand::new("atlas-cli");
    cmd.add_subcommand("manifest", "link");

    let source = params
        .get("source")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AtlasError::MissingField("source".to_string()))?;
    cmd.add_flag("source", Some(source));

    let target = params
        .get("target")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AtlasError::MissingField("target".to_string()))?;
    cmd.add_flag("target", Some(target));

    framework.build_common_flags_with_options(&mut cmd, false);

    let command_str = cmd.build();
    let result = framework.run_command(&command_str, true)?;

    let stdout = String::from_utf8_lossy(&result.stdout);

    if let Some(id) = extract_manifest_id(&stdout) {
        tracing::debug!("Link created, updated manifest ID: {}", id);
        return Ok(Some(id));
    }

    if result.status.success() {
        tracing::debug!("Link created, using original source ID: {}", source);
        return Ok(Some(source.to_string()));
    }

    Err(AtlasError::CommandError(
        "Link operation failed".to_string(),
    ))
}

fn validate_manifest(framework: &mut AtlasTestFramework, step: &Step) -> Result<Option<String>> {
    let params = &step.parameters;

    let mut cmd = AtlasCommand::new("atlas-cli");
    cmd.add_subcommand("manifest", "validate");

    let manifest_id = params
        .get("manifest_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AtlasError::MissingField("manifest_id".to_string()))?;

    cmd.add_flag("id", Some(manifest_id));

    framework.build_common_flags_with_options(&mut cmd, false);

    let command_str = cmd.build();
    let result = framework.run_command(&command_str, false)?;

    let success = result.status.success();

    if !success {
        let stderr = String::from_utf8_lossy(&result.stderr);
        tracing::warn!("Validation failed: {}", stderr);
    }

    Ok(Some(if success { "valid" } else { "invalid" }.to_string()))
}

fn verify_manifest(framework: &mut AtlasTestFramework, step: &Step) -> Result<Option<String>> {
    let params = &step.parameters;

    let mut cmd = AtlasCommand::new("atlas-cli");
    cmd.add_subcommand("manifest", "verify");

    let manifest_id = params
        .get("manifest_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AtlasError::MissingField("manifest_id".to_string()))?;

    cmd.add_flag("id", Some(manifest_id));

    framework.build_common_flags(&mut cmd);

    let command_str = cmd.build();
    let result = framework.run_command(&command_str, false)?;

    let success = result.status.success();

    if let Some(expected) = step.expect.as_deref() {
        match expected {
            "success" if !success => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                return Err(AtlasError::ValidationError(format!(
                    "Expected verification to succeed but it failed: {}",
                    stderr
                )));
            }
            "failure" if success => {
                return Err(AtlasError::ValidationError(
                    "Expected verification to fail but it succeeded".to_string(),
                ));
            }
            _ => {}
        }
    }

    Ok(Some(if success { "valid" } else { "invalid" }.to_string()))
}

fn show_manifest(framework: &mut AtlasTestFramework, step: &Step) -> Result<Option<String>> {
    let params = &step.parameters;

    let mut cmd = AtlasCommand::new("atlas-cli");
    cmd.add_subcommand("manifest", "show");

    let manifest_id = params
        .get("manifest_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AtlasError::MissingField("manifest_id".to_string()))?;

    cmd.add_flag("id", Some(manifest_id));

    framework.build_common_flags_with_options(&mut cmd, false);

    let command_str = cmd.build();
    let result = framework.run_command(&command_str, true)?;

    let stdout = String::from_utf8_lossy(&result.stdout);

    if let Some(save_to) = params.get("save_to").and_then(|v| v.as_str()) {
        let output_file = framework.resolve_path(save_to);
        if let Some(parent) = output_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&output_file, stdout.as_bytes())?;
        tracing::info!("Saved manifest to: {}", output_file.display());
    }

    Ok(Some(stdout.to_string()))
}

fn export_manifest(framework: &mut AtlasTestFramework, step: &Step) -> Result<Option<String>> {
    let params = &step.parameters;

    let mut cmd = AtlasCommand::new("atlas-cli");
    cmd.add_subcommand("manifest", "export");

    let manifest_id = params
        .get("manifest_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AtlasError::MissingField("manifest_id".to_string()))?;

    cmd.add_flag("id", Some(manifest_id));

    let output_file = if let Some(file) = params.get("output_file").and_then(|v| v.as_str()) {
        framework.resolve_path(file)
    } else {
        let id_short = &manifest_id[..8.min(manifest_id.len())];
        framework.resolve_path(&format!("./output/provenance_{}.json", id_short))
    };

    if let Some(parent) = output_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    cmd.add_flag("output", Some(output_file.display().to_string()));

    let format = params
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("json");
    cmd.add_flag("format", Some(format));

    let max_depth = params
        .get("max_depth")
        .and_then(|v| v.as_i64())
        .unwrap_or(10);
    cmd.add_flag("max-depth", Some(max_depth));

    framework.build_common_flags_with_options(&mut cmd, false);

    let command_str = cmd.build();
    framework.run_command(&command_str, true)?;

    tracing::info!("Exported provenance graph to: {}", output_file.display());
    Ok(Some(output_file.display().to_string()))
}

fn list_manifests(framework: &mut AtlasTestFramework, _step: &Step) -> Result<Option<String>> {
    let mut cmd = AtlasCommand::new("atlas-cli");
    cmd.add_subcommand("manifest", "list");

    framework.build_common_flags_with_options(&mut cmd, false);

    let command_str = cmd.build();
    let result = framework.run_command(&command_str, false)?;

    let stdout = String::from_utf8_lossy(&result.stdout);
    if !stdout.trim().is_empty() {
        tracing::info!("Manifests found:\n{}", stdout);
    } else {
        tracing::info!("No manifests found in storage");
    }

    Ok(Some(stdout.to_string()))
}
