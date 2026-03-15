use super::*;

pub(super) fn normalize_persisted_source_type(source_type: &str) -> &str {
    match source_type {
        "market" | "github" | "local" => source_type,
        _ => "market",
    }
}

pub(super) fn normalize_persisted_source_market(
    request: &InstallSkillRequest,
    persisted_source_type: &str,
) -> Option<String> {
    match persisted_source_type {
        "market" => Some(request.provider.clone()),
        "github" => Some("github".to_string()),
        "local" => None,
        _ => None,
    }
}

pub(super) fn display_source_url(source_type: &str, source_url: Option<String>) -> Option<String> {
    match (source_type, source_url) {
        ("local", Some(value)) => Some(display_path(&value)),
        (_, value) => value,
    }
}

pub(super) fn repo_url_from_metadata(metadata_json: Option<String>) -> Option<String> {
    metadata_json
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .and_then(|metadata| {
            metadata
                .get("repoUrl")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
}

pub(super) fn metadata_from_json(metadata_json: Option<String>) -> Value {
    metadata_json
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .filter(|value| value.is_object())
        .unwrap_or_else(|| json!({}))
}

pub(super) fn metadata_string_field(metadata_json: Option<String>, field: &str) -> Option<String> {
    metadata_from_json(metadata_json)
        .get(field)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn normalize_github_repo_url(value: &str) -> Option<String> {
    let parsed = Url::parse(value.trim()).ok()?;
    let host = parsed.host_str()?;
    if host != "github.com" && host != "www.github.com" {
        return None;
    }

    let segments = parsed
        .path_segments()?
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.len() < 2 {
        return None;
    }

    let owner = segments[0];
    let repo = segments[1].trim_end_matches(".git");
    if owner.is_empty() || repo.is_empty() {
        return None;
    }

    Some(format!("https://github.com/{owner}/{repo}"))
}

pub fn resolve_github_repo_url(
    repo_url: Option<String>,
    source_url: Option<String>,
) -> Option<String> {
    repo_url
        .as_deref()
        .and_then(normalize_github_repo_url)
        .or_else(|| source_url.as_deref().and_then(normalize_github_repo_url))
}

pub(super) fn can_update_from_source(repo_url: Option<String>, source_url: Option<String>) -> bool {
    resolve_github_repo_url(repo_url, source_url).is_some()
}

pub(super) fn risk_override_from_metadata(metadata_json: Option<String>) -> bool {
    metadata_json
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .and_then(|metadata| metadata.get("riskOverrideApplied").and_then(Value::as_bool))
        .unwrap_or(false)
}