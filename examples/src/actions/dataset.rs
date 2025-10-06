use crate::{
    command::AtlasCommand,
    config::Step,
    error::{AtlasError, Result},
    framework::AtlasTestFramework,
    utils::extract_manifest_id,
};

pub fn execute_dataset_action(
    framework: &mut AtlasTestFramework,
    action: &str,
    step: &Step,
) -> Result<Option<String>> {
    match action {
        "create" => create_dataset(framework, step),
        "verify" => verify_dataset(framework, step),
        "list" => list_datasets(framework, step),
        _ => Err(AtlasError::UnknownAction(format!("dataset:{}", action))),
    }
}

fn create_dataset(framework: &mut AtlasTestFramework, step: &Step) -> Result<Option<String>> {
    let params = &step.parameters;
    let env = &framework.config.environment;

    let mut cmd = AtlasCommand::new("atlas-cli");
    cmd.add_subcommand("dataset", "create");

    if let Some(paths) = params.get("paths") {
        if let Some(paths_array) = paths.as_array() {
            let paths: Vec<String> = paths_array
                .iter()
                .filter_map(|p| p.as_str())
                .map(|p| framework.resolve_path(p).display().to_string())
                .collect();

            if paths.is_empty() {
                return Err(AtlasError::MissingField(
                    "paths cannot be empty".to_string(),
                ));
            }

            cmd.add_flag("paths", Some(paths.join(",")));
        } else {
            return Err(AtlasError::InvalidParameter(
                "paths must be an array".to_string(),
            ));
        }
    } else {
        return Err(AtlasError::MissingField("paths".to_string()));
    }

    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AtlasError::MissingField("name".to_string()))?;
    cmd.add_flag("name", Some(name));

    let ingredient_names = if let Some(ingredient_names) = params.get("ingredient_names") {
        match ingredient_names {
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str())
                .map(String::from)
                .collect::<Vec<_>>(),
            serde_json::Value::String(s) => vec![s.clone()],
            _ => vec![name.to_string()],
        }
    } else {
        vec![name.to_string()]
    };

    if !ingredient_names.is_empty() {
        cmd.add_flag("ingredient-names", Some(ingredient_names.join(",")));
    }

    let author_org = params
        .get("author_org")
        .and_then(|v| v.as_str())
        .or(env.author_org.as_deref())
        .unwrap_or("Unknown Organization");
    cmd.add_flag("author-org", Some(author_org));

    let author_name = params
        .get("author_name")
        .and_then(|v| v.as_str())
        .or(env.author_name.as_deref())
        .unwrap_or("Unknown Author");
    cmd.add_flag("author-name", Some(author_name));

    if let Some(desc) = params.get("description").and_then(|v| v.as_str()) {
        cmd.add_flag("description", Some(desc));
    }

    framework.build_common_flags(&mut cmd);

    if let Some(linked) = params.get("linked_manifests") {
        if let Some(manifests) = linked.as_array() {
            for manifest in manifests {
                if let Some(id) = manifest.as_str() {
                    cmd.add_flag("linked-manifests", Some(id));
                }
            }
        }
    }

    if params
        .get("with_tdx")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        cmd.add_flag("with-tdx", Some(true));
    }

    let command_str = cmd.build();
    let result = framework.run_command(&command_str, true)?;

    let stdout = String::from_utf8_lossy(&result.stdout);
    let manifest_id = extract_manifest_id(&stdout).ok_or_else(|| {
        tracing::error!("Failed to extract manifest ID from output:\n{}", stdout);
        AtlasError::ManifestIdError(
            "Could not extract manifest ID from atlas-cli output. Check logs for details."
                .to_string(),
        )
    })?;

    tracing::debug!("Created dataset manifest: {}", manifest_id);
    Ok(Some(manifest_id))
}

fn verify_dataset(framework: &mut AtlasTestFramework, step: &Step) -> Result<Option<String>> {
    let params = &step.parameters;

    let mut cmd = AtlasCommand::new("atlas-cli");
    cmd.add_subcommand("dataset", "verify");

    let manifest_id = params
        .get("manifest_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AtlasError::MissingField("manifest_id".to_string()))?;

    cmd.add_flag("id", Some(manifest_id));

    framework.build_common_flags_with_options(&mut cmd, false);

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

fn list_datasets(framework: &mut AtlasTestFramework, _step: &Step) -> Result<Option<String>> {
    let mut cmd = AtlasCommand::new("atlas-cli");
    cmd.add_subcommand("dataset", "list");

    framework.build_common_flags_with_options(&mut cmd, false);

    let command_str = cmd.build();
    let result = framework.run_command(&command_str, false)?;

    let stdout = String::from_utf8_lossy(&result.stdout);
    if !stdout.trim().is_empty() {
        tracing::info!("Datasets found:\n{}", stdout);
    } else {
        tracing::info!("No datasets found in storage");
    }

    Ok(Some(stdout.to_string()))
}
