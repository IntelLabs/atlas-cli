use crate::{
    command::AtlasCommand,
    config::Step,
    error::{AtlasError, Result},
    framework::AtlasTestFramework,
    utils::extract_manifest_id,
};

pub fn execute_evaluation_action(
    framework: &mut AtlasTestFramework,
    action: &str,
    step: &Step,
) -> Result<Option<String>> {
    match action {
        "create" => create_evaluation(framework, step),
        "verify" => verify_evaluation(framework, step),
        _ => Err(AtlasError::UnknownAction(format!("evaluation:{}", action))),
    }
}

fn create_evaluation(framework: &mut AtlasTestFramework, step: &Step) -> Result<Option<String>> {
    let params = &step.parameters;
    let env = &framework.config.environment;

    let mut cmd = AtlasCommand::new("atlas-cli");
    cmd.add_subcommand("evaluation", "create");

    if let Some(path) = params.get("path").and_then(|v| v.as_str()) {
        let resolved_path = framework.resolve_path(path);
        cmd.add_flag("path", Some(resolved_path.display().to_string()));
    } else {
        return Err(AtlasError::MissingField("path".to_string()));
    }

    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AtlasError::MissingField("name".to_string()))?;
    cmd.add_flag("name", Some(name));

    if let Some(model_id) = params.get("model_id").and_then(|v| v.as_str()) {
        cmd.add_flag("model-id", Some(model_id));
    } else {
        return Err(AtlasError::MissingField("model_id".to_string()));
    }

    if let Some(dataset_id) = params.get("dataset_id").and_then(|v| v.as_str()) {
        cmd.add_flag("dataset-id", Some(dataset_id));
    } else {
        return Err(AtlasError::MissingField("dataset_id".to_string()));
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

    if let Some(metrics) = params.get("metrics").and_then(|v| v.as_object()) {
        for (key, value) in metrics {
            let metric_value = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => continue,
            };
            cmd.add_flag("metrics", Some(format!("{}={}", key, metric_value)));
        }
    }

    framework.build_common_flags(&mut cmd);

    let command_str = cmd.build();
    let result = framework.run_command(&command_str, true)?;

    let stdout = String::from_utf8_lossy(&result.stdout);
    let manifest_id = extract_manifest_id(&stdout).ok_or_else(|| {
        tracing::error!("Failed to extract manifest ID from output:\n{}", stdout);
        AtlasError::ManifestIdError(
            "Could not extract manifest ID from atlas-cli output".to_string(),
        )
    })?;

    tracing::debug!("Created evaluation manifest: {}", manifest_id);
    Ok(Some(manifest_id))
}

fn verify_evaluation(framework: &mut AtlasTestFramework, step: &Step) -> Result<Option<String>> {
    let params = &step.parameters;

    let mut cmd = AtlasCommand::new("atlas-cli");
    cmd.add_subcommand("evaluation", "verify");

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
