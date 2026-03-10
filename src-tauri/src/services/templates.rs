use anyhow::{anyhow, Result};
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use crate::{
    domain::{
        app_state::AppState,
        types::{
            InjectTemplateItemResult, InjectTemplateRequest, InjectTemplateResult,
            SaveTemplateRequest, TemplateRecord,
        },
    },
    repositories::{
        distributions as distributions_repository, settings as settings_repository,
        skills as skills_repository, templates as templates_repository,
    },
    services::distribution,
};

const BUILTIN_TEMPLATE_TARGETS: &[(&str, &str)] = &[
    ("universal", ".agents/skills"),
    ("antigravity", ".agent/skills"),
    ("augment", ".augment/skills"),
    ("claude-code", ".claude/skills"),
    ("openclaw", "skills"),
    ("codebuddy", ".codebuddy/skills"),
    ("command-code", ".commandcode/skills"),
    ("continue", ".continue/skills"),
    ("cortex-code", ".cortex/skills"),
    ("crush", ".crush/skills"),
    ("droid", ".factory/skills"),
    ("goose", ".goose/skills"),
    ("junie", ".junie/skills"),
    ("iflow-cli", ".iflow/skills"),
    ("kilo-code", ".kilocode/skills"),
    ("kiro-cli", ".kiro/skills"),
    ("kode", ".kode/skills"),
    ("mcpjam", ".mcpjam/skills"),
    ("mistral-vibe", ".vibe/skills"),
    ("mux", ".mux/skills"),
    ("openhands", ".openhands/skills"),
    ("pi", ".pi/skills"),
    ("qoder", ".qoder/skills"),
    ("qwen-code", ".qwen/skills"),
    ("roo-code", ".roo/skills"),
    ("trae", ".trae/skills"),
    ("trae-cn", ".trae/skills"),
    ("windsurf", ".windsurf/skills"),
    ("zencoder", ".zencoder/skills"),
    ("neovate", ".neovate/skills"),
    ("pochi", ".pochi/skills"),
    ("adal", ".adal/skills"),
];

fn validate_template(request: &SaveTemplateRequest) -> Result<()> {
    if request.name.trim().is_empty() {
        return Err(anyhow!("template name is required"));
    }

    for item in &request.items {
        if item.skill_ref_type.trim().is_empty() {
            return Err(anyhow!("template item skill_ref_type is required"));
        }
        if item.skill_ref.trim().is_empty() {
            return Err(anyhow!("template item skill_ref is required"));
        }
    }

    Ok(())
}

fn normalize_relative_path(relative_path: &str) -> String {
    relative_path
        .replace('\\', "/")
        .trim()
        .trim_matches('/')
        .to_string()
}

fn validate_custom_relative_path(relative_path: &str) -> Result<String> {
    let normalized = normalize_relative_path(relative_path);
    if normalized.is_empty() {
        return Err(anyhow!("custom relative path is required"));
    }
    if normalized.starts_with('/') || normalized.starts_with('\\') {
        return Err(anyhow!("custom relative path must be relative"));
    }
    if normalized.split('/').any(|segment| segment == "..") {
        return Err(anyhow!("custom relative path cannot contain parent directory segments"));
    }
    Ok(normalized)
}

fn resolve_tag_relative_path(state: &AppState, target_agent_id: &str) -> Result<String> {
    if let Some((_, relative_path)) = BUILTIN_TEMPLATE_TARGETS
        .iter()
        .find(|(id, _)| *id == target_agent_id)
    {
        return Ok((*relative_path).to_string());
    }

    let settings = settings_repository::load_settings(&state.paths.db_file)?
        .unwrap_or_else(|| settings_repository::default_settings("en-US".into()));

    settings
        .custom_skills_targets
        .into_iter()
        .find(|target| target.id == target_agent_id)
        .map(|target| target.relative_path)
        .ok_or_else(|| anyhow!("unknown target agent id {}", target_agent_id))
}

fn ensure_project_root(project_root: &str) -> Result<PathBuf> {
    let path = PathBuf::from(project_root);
    if !path.exists() {
        return Err(anyhow!("project root does not exist: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(anyhow!("project root is not a directory: {}", path.display()));
    }
    Ok(path)
}

fn create_injection_symlink(source: &Path, target: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::os::windows::fs::symlink_dir(source, target).map_err(|error| {
            if error.kind() == ErrorKind::PermissionDenied {
                anyhow!(
                    "Windows symlink permission denied for {}. Enable Developer Mode or run with elevated permission.",
                    target.display()
                )
            } else {
                anyhow!(error)
            }
        })?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::os::unix::fs::symlink(source, target)?;
    }

    Ok(())
}

fn build_result_item(
    skill_id: &str,
    skill_name: &str,
    target_path: &Path,
    reason: Option<String>,
) -> InjectTemplateItemResult {
    InjectTemplateItemResult {
        skill_id: skill_id.to_string(),
        skill_name: skill_name.to_string(),
        target_path: target_path.to_string_lossy().to_string(),
        reason,
    }
}

pub fn list_templates(path: &Path) -> Result<Vec<TemplateRecord>> {
    templates_repository::list_templates(path)
}

pub fn get_template(path: &Path, template_id: &str) -> Result<Option<TemplateRecord>> {
    templates_repository::get_template(path, template_id)
}

pub fn save_template(path: &Path, request: &SaveTemplateRequest) -> Result<TemplateRecord> {
    validate_template(request)?;
    templates_repository::save_template(path, request)
}

pub fn delete_template(path: &Path, template_id: &str) -> Result<()> {
    templates_repository::delete_template(path, template_id)
}

pub fn inject_template(state: &AppState, request: &InjectTemplateRequest) -> Result<InjectTemplateResult> {
    if request.template_id.trim().is_empty() {
        return Err(anyhow!("template_id is required"));
    }
    if request.project_root.trim().is_empty() {
        return Err(anyhow!("project_root is required"));
    }
    if request.install_mode != "symlink" && request.install_mode != "copy" {
        return Err(anyhow!("unsupported install mode {}", request.install_mode));
    }

    let template = templates_repository::get_template(&state.paths.db_file, &request.template_id)?
        .ok_or_else(|| anyhow!("template {} does not exist", request.template_id))?;
    if template.items.is_empty() {
        return Err(anyhow!("template must contain at least one skill before injection"));
    }

    let project_root = ensure_project_root(&request.project_root)?;
    let relative_target = match request.target_type.as_str() {
        "tag" => {
            let target_agent_id = request
                .target_agent_id
                .as_deref()
                .ok_or_else(|| anyhow!("target_agent_id is required when target_type=tag"))?;
            resolve_tag_relative_path(state, target_agent_id)?
        }
        "custom" => validate_custom_relative_path(
            request
                .custom_relative_path
                .as_deref()
                .ok_or_else(|| anyhow!("custom_relative_path is required when target_type=custom"))?,
        )?,
        other => return Err(anyhow!("unsupported target_type {}", other)),
    };

    let target_root = project_root.join(relative_target);
    fs::create_dir_all(&target_root)?;

    let mut installed = Vec::new();
    let mut skipped = Vec::new();
    let mut failed = Vec::new();

    for item in &template.items {
        if item.skill_ref_type != "repository_skill" {
            failed.push(build_result_item(
                &item.skill_ref,
                item.display_name.as_deref().unwrap_or(&item.skill_ref),
                &target_root.join(&item.skill_ref),
                Some(format!("unsupported template item type {}", item.skill_ref_type)),
            ));
            continue;
        }

        let skill_name = item.display_name.clone().unwrap_or_else(|| item.skill_ref.clone());
        let source = match skills_repository::load_skill_source(&state.paths.db_file, &item.skill_ref) {
            Ok(source) => source,
            Err(error) => {
                failed.push(build_result_item(
                    &item.skill_ref,
                    &skill_name,
                    &target_root.join(&item.skill_ref),
                    Some(format!("missing repository skill: {}", error)),
                ));
                continue;
            }
        };

        let source_path = PathBuf::from(&source.source_path);
        let target_path = target_root.join(&source.target_name);

        if target_path.exists() || fs::symlink_metadata(&target_path).is_ok() {
            skipped.push(build_result_item(
                &item.skill_ref,
                &skill_name,
                &target_path,
                Some("target already exists".into()),
            ));
            continue;
        }

        let install_result: Result<()> = match request.install_mode.as_str() {
            "symlink" => create_injection_symlink(&source_path, &target_path),
            "copy" => distribution::copy_dir_all(&source_path, &target_path),
            _ => unreachable!(),
        };

        match install_result {
            Ok(()) => {
                let project_id = distributions_repository::find_project_id_by_root(
                    &state.paths.db_file,
                    &request.project_root,
                )?;

                distributions_repository::save_distribution(
                    &state.paths.db_file,
                    &item.skill_ref,
                    "project",
                    request.target_agent_id.as_deref().unwrap_or("custom"),
                    project_id.as_deref(),
                    &target_path.to_string_lossy(),
                    &request.install_mode,
                    "active",
                )?;

                installed.push(build_result_item(&item.skill_ref, &skill_name, &target_path, None));
            }
            Err(error) => {
                failed.push(build_result_item(
                    &item.skill_ref,
                    &skill_name,
                    &target_path,
                    Some(error.to_string()),
                ));
            }
        }
    }

    Ok(InjectTemplateResult {
        installed,
        skipped,
        failed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            agent_registry::AgentRegistry,
            app_state::{AppPaths, AppState},
            types::{AppSettings, CustomSkillsTarget, DistributionRequest, InstallSkillRequest, SaveTemplateItemRequest},
        },
        repositories::{db::run_migrations, settings as settings_repository, skills as skills_repository},
    };
    use std::sync::Arc;
    use tempfile::tempdir;

    fn test_state(root: &Path) -> AppState {
        let app_root = root.join("app-data");
        let db_dir = app_root.join("db");
        let temp_dir = app_root.join("tmp");
        let canonical_store_dir = app_root.join("skills");

        fs::create_dir_all(&db_dir).unwrap();
        fs::create_dir_all(&temp_dir).unwrap();
        fs::create_dir_all(&canonical_store_dir).unwrap();

        AppState {
            paths: AppPaths {
                db_file: db_dir.join("skills-manager.db"),
                temp_dir,
                canonical_store_dir,
            },
            agent_registry: Arc::new(AgentRegistry::new()),
        }
    }

    fn seed_skill(state: &AppState, slug: &str, name: &str) -> String {
        let source_dir = state.paths.canonical_store_dir.join(slug);
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("SKILL.md"), format!("# {}", name)).unwrap();

        skills_repository::save_installed_skill(
            &state.paths.db_file,
            &InstallSkillRequest {
                provider: "github".into(),
                market_skill_id: slug.into(),
                source_type: "github-resolved-skill".into(),
                source_url: "https://example.com/demo".into(),
                repo_url: Some("https://example.com/demo".into()),
                download_url: None,
                package_ref: Some(format!("example/demo@{}", slug)),
                manifest_path: Some(format!("skills/{}/SKILL.md", slug)),
                skill_root: Some(format!("skills/{}", slug)),
                name: name.into(),
                slug: slug.into(),
                version: Some("main".into()),
                author: Some("tester".into()),
                requested_targets: Vec::<DistributionRequest>::new(),
            },
            &source_dir.to_string_lossy(),
            "safe",
            false,
        )
        .unwrap()
    }

    fn seed_template(state: &AppState, skill_ids: &[(&str, &str)]) -> String {
        templates_repository::save_template(
            &state.paths.db_file,
            &SaveTemplateRequest {
                id: None,
                name: "Vue Template".into(),
                description: Some("starter".into()),
                tags: vec!["vue".into()],
                items: skill_ids
                    .iter()
                    .enumerate()
                    .map(|(index, (skill_id, display_name))| SaveTemplateItemRequest {
                        skill_ref_type: "repository_skill".into(),
                        skill_ref: (*skill_id).into(),
                        display_name: Some((*display_name).into()),
                        order_index: Some(index as u32),
                    })
                    .collect(),
            },
        )
        .unwrap()
        .id
    }

    #[test]
    fn injects_template_into_tag_path_with_copy_mode() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        let skill_id = seed_skill(&state, "demo-skill", "Demo Skill");
        let template_id = seed_template(&state, &[(&skill_id, "Demo Skill")]);
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        let result = inject_template(
            &state,
            &InjectTemplateRequest {
                template_id,
                project_root: project_root.to_string_lossy().to_string(),
                target_type: "tag".into(),
                target_agent_id: Some("claude-code".into()),
                custom_relative_path: None,
                install_mode: "copy".into(),
            },
        )
        .unwrap();

        assert_eq!(result.installed.len(), 1);
        assert!(project_root
            .join(".claude/skills/demo-skill/SKILL.md")
            .exists());
    }

    #[test]
    fn skips_existing_targets_and_marks_missing_skills_failed() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        let skill_id = seed_skill(&state, "demo-skill", "Demo Skill");
        let template_id =
            seed_template(&state, &[(&skill_id, "Demo Skill"), ("missing-skill", "Missing Skill")]);
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(project_root.join(".claude/skills/demo-skill")).unwrap();

        let result = inject_template(
            &state,
            &InjectTemplateRequest {
                template_id,
                project_root: project_root.to_string_lossy().to_string(),
                target_type: "tag".into(),
                target_agent_id: Some("claude-code".into()),
                custom_relative_path: None,
                install_mode: "copy".into(),
            },
        )
        .unwrap();

        assert_eq!(result.installed.len(), 0);
        assert_eq!(result.skipped.len(), 1);
        assert_eq!(result.failed.len(), 1);
    }

    #[test]
    fn uses_custom_relative_path_and_custom_target_id() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        let skill_id = seed_skill(&state, "demo-skill", "Demo Skill");
        let template_id = seed_template(&state, &[(&skill_id, "Demo Skill")]);
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        settings_repository::save_settings(
            &state.paths.db_file,
            &AppSettings {
                language: "zh-CN".into(),
                theme_mode: "system".into(),
                visible_skills_target_ids: vec!["custom-demo".into()],
                custom_skills_targets: vec![CustomSkillsTarget {
                    id: "custom-demo".into(),
                    label: "Demo IDE".into(),
                    relative_path: ".demo/skills".into(),
                }],
            },
        )
        .unwrap();

        let tag_result = inject_template(
            &state,
            &InjectTemplateRequest {
                template_id: template_id.clone(),
                project_root: project_root.to_string_lossy().to_string(),
                target_type: "tag".into(),
                target_agent_id: Some("custom-demo".into()),
                custom_relative_path: None,
                install_mode: "copy".into(),
            },
        )
        .unwrap();
        assert_eq!(tag_result.installed.len(), 1);
        assert!(project_root.join(".demo/skills/demo-skill/SKILL.md").exists());

        let project_root_custom = dir.path().join("workspace-custom");
        fs::create_dir_all(&project_root_custom).unwrap();
        let custom_result = inject_template(
            &state,
            &InjectTemplateRequest {
                template_id,
                project_root: project_root_custom.to_string_lossy().to_string(),
                target_type: "custom".into(),
                target_agent_id: None,
                custom_relative_path: Some(".company/skills/vue".into()),
                install_mode: "copy".into(),
            },
        )
        .unwrap();
        assert_eq!(custom_result.installed.len(), 1);
        assert!(project_root_custom
            .join(".company/skills/vue/demo-skill/SKILL.md")
            .exists());
    }

    #[test]
    fn rejects_template_injection_when_template_has_no_skills() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        let template_id = seed_template(&state, &[]);
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        let error = inject_template(
            &state,
            &InjectTemplateRequest {
                template_id,
                project_root: project_root.to_string_lossy().to_string(),
                target_type: "tag".into(),
                target_agent_id: Some("claude-code".into()),
                custom_relative_path: None,
                install_mode: "copy".into(),
            },
        )
        .unwrap_err();

        assert!(error.to_string().contains("at least one skill"));
    }

    #[test]
    fn rejects_invalid_custom_relative_path() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        let skill_id = seed_skill(&state, "demo-skill", "Demo Skill");
        let template_id = seed_template(&state, &[(&skill_id, "Demo Skill")]);
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        let error = inject_template(
            &state,
            &InjectTemplateRequest {
                template_id,
                project_root: project_root.to_string_lossy().to_string(),
                target_type: "custom".into(),
                target_agent_id: None,
                custom_relative_path: Some("../escape".into()),
                install_mode: "copy".into(),
            },
        )
        .unwrap_err();

        assert!(error.to_string().contains("cannot contain parent directory"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn reports_symlink_permission_failures_without_fallback() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        let skill_id = seed_skill(&state, "demo-skill", "Demo Skill");
        let template_id = seed_template(&state, &[(&skill_id, "Demo Skill")]);
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        let result = inject_template(
            &state,
            &InjectTemplateRequest {
                template_id,
                project_root: project_root.to_string_lossy().to_string(),
                target_type: "tag".into(),
                target_agent_id: Some("claude-code".into()),
                custom_relative_path: None,
                install_mode: "symlink".into(),
            },
        )
        .unwrap();

        if result.failed.is_empty() {
            assert!(project_root
                .join(".claude/skills/demo-skill")
                .symlink_metadata()
                .unwrap()
                .file_type()
                .is_symlink());
        } else {
            assert!(result.failed[0]
                .reason
                .as_deref()
                .unwrap_or_default()
                .contains("Windows symlink permission denied"));
        }
    }
}
