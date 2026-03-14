use anyhow::{anyhow, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    domain::{
        agent_registry::VISIBLE_SKILLS_TARGETS_VERSION,
        app_state::{AppPaths, AppState},
        types::{
            AppSettings, MigrateRepositoryStorageRequest, MigrateRepositoryStorageResult,
            RepositoryStorageInfo,
        },
    },
    path_utils::display_path,
    repositories::{
        db::open_connection,
        settings as settings_repo,
        skills as skills_repository,
    },
    services::fs_utils::{copy_dir_all, remove_dir_if_present},
};

fn insert_visible_target_once(
    visible_skills_target_ids: &mut Vec<String>,
    target_id: &str,
    after_target_id: Option<&str>,
) {
    if visible_skills_target_ids
        .iter()
        .any(|id| id == target_id)
    {
        return;
    }

    if let Some(after_target_id) = after_target_id {
        if let Some(index) = visible_skills_target_ids
            .iter()
            .position(|id| id == after_target_id)
        {
            visible_skills_target_ids.insert(index + 1, target_id.to_string());
            return;
        }
    }

    visible_skills_target_ids.push(target_id.to_string());
}

fn normalize_settings(mut settings: AppSettings) -> AppSettings {
    if settings.visible_skills_targets_version < VISIBLE_SKILLS_TARGETS_VERSION {
        insert_visible_target_once(
            &mut settings.visible_skills_target_ids,
            "codex",
            Some("universal"),
        );
        settings.visible_skills_targets_version = VISIBLE_SKILLS_TARGETS_VERSION;
    }

    settings
}

pub fn load_or_create_settings(state: &AppState, language: String) -> Result<AppSettings> {
    if let Some(settings) = settings_repo::load_settings(&state.paths.db_file)? {
        let normalized = normalize_settings(settings);
        let persisted = settings_repo::save_settings(&state.paths.db_file, &normalized)?;
        return Ok(persisted);
    }

    let settings = settings_repo::default_settings(language);
    settings_repo::save_settings(&state.paths.db_file, &settings)
}

pub fn save_settings(state: &AppState, settings: &AppSettings) -> Result<AppSettings> {
    let persisted = settings_repo::load_settings(&state.paths.db_file)?
        .unwrap_or_else(|| settings_repo::default_settings(settings.language.clone()));
    if settings.repository_storage_path != persisted.repository_storage_path {
        return Err(anyhow!(
            "repository storage path must be changed via migrate_repository_storage"
        ));
    }

    let normalized = normalize_settings(settings.clone());
    settings_repo::save_settings(&state.paths.db_file, &normalized)
}

pub fn resolve_repository_storage_dir(state: &AppState) -> Result<PathBuf> {
    let configured = settings_repo::load_settings(&state.paths.db_file)?
        .and_then(|settings| settings.repository_storage_path);

    let resolved = configured
        .map(PathBuf::from)
        .unwrap_or_else(|| state.paths.canonical_store_dir.clone());
    fs::create_dir_all(&resolved)?;
    fs::canonicalize(&resolved)
        .with_context(|| format!("failed to canonicalize {}", resolved.display()))
}

pub fn runtime_paths(state: &AppState) -> Result<AppPaths> {
    let mut paths = state.paths.clone();
    paths.canonical_store_dir = resolve_repository_storage_dir(state)?;
    Ok(paths)
}

fn normalize_repository_storage_info(info: RepositoryStorageInfo) -> RepositoryStorageInfo {
    RepositoryStorageInfo {
        default_path: display_path(&info.default_path),
        current_path: display_path(&info.current_path),
        is_custom: info.is_custom,
    }
}

pub fn repository_storage_info(state: &AppState) -> Result<RepositoryStorageInfo> {
    let current_path = resolve_repository_storage_dir(state)?;
    let default_path = fs::canonicalize(&state.paths.canonical_store_dir).with_context(|| {
        format!(
            "failed to canonicalize default repository storage {}",
            state.paths.canonical_store_dir.display()
        )
    })?;

    Ok(normalize_repository_storage_info(RepositoryStorageInfo {
        default_path: default_path.to_string_lossy().to_string(),
        current_path: current_path.to_string_lossy().to_string(),
        is_custom: current_path != default_path,
    }))
}

fn normalize_target_storage_path(raw: &str) -> Result<PathBuf> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("target path is required"));
    }

    let input = PathBuf::from(trimmed);
    if input.exists() {
        if !input.is_dir() {
            return Err(anyhow!(
                "repository storage target must be a directory: {}",
                input.display()
            ));
        }
        return fs::canonicalize(&input)
            .with_context(|| format!("failed to canonicalize {}", input.display()));
    }

    let parent = input
        .parent()
        .ok_or_else(|| anyhow!("target path must include an existing parent directory"))?;
    if !parent.exists() {
        return Err(anyhow!(
            "target parent directory does not exist: {}",
            parent.display()
        ));
    }
    let file_name = input
        .file_name()
        .ok_or_else(|| anyhow!("target path must end with a directory name"))?;

    let canonical_parent = fs::canonicalize(parent)
        .with_context(|| format!("failed to canonicalize {}", parent.display()))?;
    Ok(canonical_parent.join(file_name))
}

fn ensure_empty_target_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if !path.is_dir() {
        return Err(anyhow!(
            "repository storage target must be a directory: {}",
            path.display()
        ));
    }
    if fs::read_dir(path)?.next().transpose()?.is_some() {
        return Err(anyhow!(
            "repository storage target must be empty: {}",
            path.display()
        ));
    }
    Ok(())
}

fn validate_storage_target(current_path: &Path, target_path: &Path) -> Result<()> {
    if current_path == target_path {
        return Err(anyhow!("repository storage target must be different from current path"));
    }
    if target_path.starts_with(current_path) {
        return Err(anyhow!(
            "repository storage target cannot be inside current repository storage"
        ));
    }
    if current_path.starts_with(target_path) {
        return Err(anyhow!(
            "repository storage target cannot contain current repository storage"
        ));
    }
    ensure_empty_target_directory(target_path)
}

fn verify_migrated_entries(
    current_root: &Path,
    target_root: &Path,
    entries: &[skills_repository::RepositoryStorageEntry],
) -> Result<()> {
    for entry in entries {
        let current_path = PathBuf::from(&entry.canonical_path);
        let relative = current_path
            .strip_prefix(current_root)
            .with_context(|| format!("failed to rebase skill {}", entry.skill_id))?;
        let migrated_path = target_root.join(relative);
        if migrated_path.file_name().and_then(|value| value.to_str()) != Some(entry.slug.as_str()) {
            return Err(anyhow!(
                "migrated repository skill slug mismatch for {}",
                entry.skill_id
            ));
        }
        if !migrated_path.exists() {
            return Err(anyhow!(
                "migrated repository skill is missing: {}",
                migrated_path.display()
            ));
        }
        if !migrated_path.join("SKILL.md").exists() {
            return Err(anyhow!(
                "migrated repository skill missing SKILL.md: {}",
                migrated_path.display()
            ));
        }
    }
    Ok(())
}

fn persist_migrated_storage(
    db_file: &Path,
    next_settings: &AppSettings,
    previous_root: &Path,
    current_root: &Path,
) -> Result<usize> {
    let mut conn = open_connection(db_file)?;
    let tx = conn.transaction()?;
    let migrated_skill_count = skills_repository::rewrite_repository_storage_paths_with_connection(
        &tx,
        previous_root,
        current_root,
    )?;
    settings_repo::save_settings_with_connection(&tx, next_settings)?;
    tx.commit()?;

    Ok(migrated_skill_count)
}

fn rollback_migrated_storage(
    db_file: &Path,
    previous_settings: &AppSettings,
    current_settings: &AppSettings,
    previous_root: &Path,
    current_root: &Path,
) -> Result<()> {
    if previous_settings.repository_storage_path == current_settings.repository_storage_path {
        return Ok(());
    }

    persist_migrated_storage(
        db_file,
        previous_settings,
        current_root,
        previous_root,
    )?;
    Ok(())
}

pub fn migrate_repository_storage(
    state: &AppState,
    request: &MigrateRepositoryStorageRequest,
) -> Result<MigrateRepositoryStorageResult> {
    let previous_settings = settings_repo::load_settings(&state.paths.db_file)?
        .unwrap_or_else(|| settings_repo::default_settings("en-US".into()));
    let previous_root = resolve_repository_storage_dir(state)?;
    let target_root = normalize_target_storage_path(&request.target_path)?;
    validate_storage_target(&previous_root, &target_root)?;

    let entries =
        skills_repository::list_repository_storage_entries(&state.paths.db_file, &previous_root)?;

    fs::create_dir_all(&target_root)?;
    let copy_result = copy_dir_all(&previous_root, &target_root)
        .and_then(|_| verify_migrated_entries(&previous_root, &target_root, &entries));
    if let Err(error) = copy_result {
        let _ = remove_dir_if_present(&target_root);
        return Err(error);
    }

    let default_root = fs::canonicalize(&state.paths.canonical_store_dir).with_context(|| {
        format!(
            "failed to canonicalize default repository storage {}",
            state.paths.canonical_store_dir.display()
        )
    })?;
    let target_root = fs::canonicalize(&target_root)
        .with_context(|| format!("failed to canonicalize {}", target_root.display()))?;

    let mut next_settings = previous_settings.clone();
    next_settings.repository_storage_path = if target_root == default_root {
        None
    } else {
        Some(target_root.to_string_lossy().to_string())
    };

    let migrated_skill_count = match persist_migrated_storage(
        &state.paths.db_file,
        &next_settings,
        &previous_root,
        &target_root,
    ) {
        Ok(count) => count,
        Err(error) => {
            let _ = remove_dir_if_present(&target_root);
            return Err(error);
        }
    };

    if let Err(error) = remove_dir_if_present(&previous_root) {
        let rollback_error = rollback_migrated_storage(
            &state.paths.db_file,
            &previous_settings,
            &next_settings,
            &previous_root,
            &target_root,
        );
        let cleanup_error = remove_dir_if_present(&target_root).err();
        let mut message = format!("failed to remove previous repository storage: {}", error);
        if let Err(rollback_error) = rollback_error {
            message.push_str(&format!("; rollback failed: {}", rollback_error));
        }
        if let Some(cleanup_error) = cleanup_error {
            message.push_str(&format!("; cleanup failed: {}", cleanup_error));
        }
        return Err(anyhow!(message));
    }

    Ok(MigrateRepositoryStorageResult {
        previous_path: display_path(&previous_root.to_string_lossy()),
        current_path: display_path(&target_root.to_string_lossy()),
        migrated_skill_count,
        removed_old_path: true,
        cleanup_warning: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::types::{CustomSkillsTarget, DistributionRequest, InstallSkillRequest},
        repositories::{db::run_migrations, skills as skills_repository},
    };
    use std::{fs, sync::Arc};
    use tempfile::tempdir;

    fn create_state() -> AppState {
        let root = tempdir().unwrap().keep();
        let app_data_dir = root.join("app-data");
        let db_dir = app_data_dir.join("db");
        let temp_dir = app_data_dir.join("tmp");
        let canonical_store_dir = app_data_dir.join("skills");
        fs::create_dir_all(&db_dir).unwrap();
        fs::create_dir_all(&temp_dir).unwrap();
        fs::create_dir_all(&canonical_store_dir).unwrap();

        let db_file = db_dir.join("skills-manager.db");
        run_migrations(&db_file).unwrap();

        AppState {
            paths: AppPaths {
                db_file,
                temp_dir,
                canonical_store_dir,
            },
            agent_registry: Arc::new(crate::domain::agent_registry::AgentRegistry::new()),
        }
    }

    fn seed_repository_skill(state: &AppState, slug: &str) -> String {
        let skill_dir = state.paths.canonical_store_dir.join(slug);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), format!("# {}", slug)).unwrap();

        skills_repository::save_installed_skill(
            &state.paths.db_file,
            &InstallSkillRequest {
                provider: "github".into(),
                market_skill_id: slug.into(),
                source_type: "github-resolved-skill".into(),
                source_url: format!("https://example.com/{}", slug),
                repo_url: Some(format!("https://example.com/{}", slug)),
                download_url: None,
                package_ref: Some(format!("example/demo@skills/{}", slug)),
                manifest_path: Some(format!("skills/{}/SKILL.md", slug)),
                skill_root: Some(format!("skills/{}", slug)),
                name: slug.into(),
                slug: slug.into(),
                description: Some("repository skill".into()),
                version: None,
                author: None,
                requested_targets: Vec::<DistributionRequest>::new(),
            },
            &skill_dir.to_string_lossy(),
            "safe",
            false,
        )
        .unwrap()
    }

    #[test]
    fn keeps_repository_storage_path_empty_for_legacy_settings_json() {
        let legacy = serde_json::json!({
            "language": "zh-CN",
            "themeMode": "dark",
            "visibleSkillsTargetIds": ["universal"],
            "customSkillsTargets": [
                {
                    "id": "demo",
                    "label": "Demo",
                    "relativePath": ".demo/skills"
                }
            ]
        });

        let parsed: AppSettings = serde_json::from_value(legacy).unwrap();

        assert_eq!(parsed.language, "zh-CN");
        assert_eq!(parsed.theme_mode, "dark");
        assert_eq!(parsed.visible_skills_target_ids, vec!["universal"]);
        assert_eq!(parsed.visible_skills_targets_version, 0);
        assert_eq!(
            parsed.custom_skills_targets,
            vec![CustomSkillsTarget {
                id: "demo".into(),
                label: "Demo".into(),
                relative_path: ".demo/skills".into(),
            }]
        );
        assert_eq!(parsed.repository_storage_path, None);
    }

    #[test]
    fn hides_windows_verbatim_prefix_in_repository_storage_info() {
        let info = RepositoryStorageInfo {
            default_path: r"\\?\C:\Users\jiang\AppData\Roaming\app\skills".into(),
            current_path: r"\\?\UNC\server\share\skills".into(),
            is_custom: true,
        };

        let normalized = normalize_repository_storage_info(info);

        assert_eq!(
            normalized.default_path,
            r"C:\Users\jiang\AppData\Roaming\app\skills"
        );
        assert_eq!(normalized.current_path, r"\\server\share\skills");
    }

    #[test]
    fn migrates_repository_storage_and_rewrites_canonical_paths() {
        let state = create_state();
        let skill_id = seed_repository_skill(&state, "demo-skill");
        let target_dir = state.paths.temp_dir.join("repository-target");

        let result = migrate_repository_storage(
            &state,
            &MigrateRepositoryStorageRequest {
                target_path: target_dir.to_string_lossy().to_string(),
            },
        )
        .unwrap();

        assert_eq!(result.migrated_skill_count, 1);
        assert!(result.removed_old_path);
        assert_eq!(result.cleanup_warning, None);

        let detail = skills_repository::get_repository_skill_detail(
            &state.paths.db_file,
            PathBuf::from(&result.current_path).as_path(),
            &skill_id,
        )
        .unwrap();

        assert!(detail.canonical_path.starts_with(&result.current_path));
        assert!(PathBuf::from(&detail.canonical_path).join("SKILL.md").exists());
        assert!(!state.paths.canonical_store_dir.join("demo-skill").exists());
    }

    #[test]
    fn rejects_nested_repository_storage_targets() {
        let state = create_state();
        let nested_target = state.paths.canonical_store_dir.join("nested");

        let error = migrate_repository_storage(
            &state,
            &MigrateRepositoryStorageRequest {
                target_path: nested_target.to_string_lossy().to_string(),
            },
        )
        .unwrap_err();

        assert!(error.to_string().contains("cannot be inside current repository storage"));
    }

    #[test]
    fn load_or_create_settings_migrates_legacy_visible_targets() {
        let state = create_state();
        let legacy = serde_json::json!({
            "language": "zh-CN",
            "themeMode": "dark",
            "visibleSkillsTargetIds": ["universal", "qoder"],
            "customSkillsTargets": [],
            "repositoryStoragePath": null
        });

        let conn = open_connection(&state.paths.db_file).unwrap();
        conn.execute(
            "
            INSERT INTO settings (key, value_json, updated_at)
            VALUES (?1, ?2, 0)
            ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at
            ",
            rusqlite::params![settings_repo::SETTINGS_KEY, legacy.to_string()],
        )
        .unwrap();

        let loaded = load_or_create_settings(&state, "zh-CN".into()).unwrap();

        assert_eq!(
            loaded.visible_skills_target_ids,
            vec![
                "universal".to_string(),
                "codex".to_string(),
                "qoder".to_string()
            ]
        );
        assert_eq!(
            loaded.visible_skills_targets_version,
            VISIBLE_SKILLS_TARGETS_VERSION
        );

        let persisted = settings_repo::load_settings(&state.paths.db_file)
            .unwrap()
            .unwrap();
        assert_eq!(persisted.visible_skills_target_ids, loaded.visible_skills_target_ids);
        assert_eq!(
            persisted.visible_skills_targets_version,
            VISIBLE_SKILLS_TARGETS_VERSION
        );
    }
}
