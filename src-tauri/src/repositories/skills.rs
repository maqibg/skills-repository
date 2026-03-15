use anyhow::{anyhow, Context, Result};
use rusqlite::{params, Connection, Transaction};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

use crate::domain::types::{InstallSkillRequest, RepositorySkillDetail, RepositorySkillSummary};
use crate::path_utils::display_path;

use super::db::open_connection;

#[path = "skills/metadata.rs"]
mod metadata;
#[path = "skills/install.rs"]
mod install;

pub use install::{repository_skill_slug_exists, save_installed_skill};
pub use metadata::resolve_github_repo_url;
use metadata::{
    can_update_from_source, display_source_url, metadata_from_json, metadata_string_field,
    normalize_persisted_source_market, normalize_persisted_source_type, repo_url_from_metadata,
    risk_override_from_metadata,
};

pub fn load_skill_name(path: &Path, skill_id: &str) -> Result<String> {
    let conn = open_connection(path)?;
    conn.query_row(
        "SELECT name FROM skills WHERE id = ?1",
        params![skill_id],
        |row| row.get(0),
    )
    .map_err(Into::into)
}

pub fn save_operation_log(
    path: &Path,
    operation_type: &str,
    entity_type: &str,
    entity_id: Option<&str>,
    status: &str,
    summary: &str,
    detail_json: Option<serde_json::Value>,
) -> Result<String> {
    let conn = open_connection(path)?;
    save_operation_log_in_connection(
        &conn,
        operation_type,
        entity_type,
        entity_id,
        status,
        summary,
        detail_json,
    )
}

fn save_operation_log_in_connection(
    conn: &Connection,
    operation_type: &str,
    entity_type: &str,
    entity_id: Option<&str>,
    status: &str,
    summary: &str,
    detail_json: Option<serde_json::Value>,
) -> Result<String> {
    let log_id = Uuid::new_v4().to_string();
    let now = OffsetDateTime::now_utc().unix_timestamp();

    conn.execute(
        "
        INSERT INTO operation_logs (
            id,
            operation_type,
            entity_type,
            entity_id,
            status,
            summary,
            detail_json,
            created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ",
        params![
            log_id,
            operation_type,
            entity_type,
            entity_id,
            status,
            summary,
            detail_json.map(|value| value.to_string()),
            now,
        ],
    )?;

    Ok(log_id)
}

pub fn save_operation_log_in_tx(
    tx: &Transaction<'_>,
    operation_type: &str,
    entity_type: &str,
    entity_id: Option<&str>,
    status: &str,
    summary: &str,
    detail_json: Option<serde_json::Value>,
) -> Result<String> {
    let log_id = Uuid::new_v4().to_string();
    let now = OffsetDateTime::now_utc().unix_timestamp();

    tx.execute(
        "
        INSERT INTO operation_logs (
            id,
            operation_type,
            entity_type,
            entity_id,
            status,
            summary,
            detail_json,
            created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ",
        params![
            log_id,
            operation_type,
            entity_type,
            entity_id,
            status,
            summary,
            detail_json.map(|value| value.to_string()),
            now,
        ],
    )?;

    Ok(log_id)
}

pub struct SkillSource {
    pub source_path: String,
    pub target_name: String,
}

pub struct InstalledSkillSummary {
    pub skill_id: String,
    pub name: String,
    pub canonical_path: String,
    pub source_url: Option<String>,
    pub repo_url: Option<String>,
    pub version: Option<String>,
    pub risk_override_applied: bool,
}

#[derive(Debug, Clone)]
pub struct RepositorySkillUpdateTarget {
    pub skill_id: String,
    pub name: String,
    pub slug: String,
    pub canonical_path: String,
    pub source_type: String,
    pub repo_url: String,
    pub manifest_path: Option<String>,
    pub skill_root: Option<String>,
    pub version: Option<String>,
    pub copy_distribution_count: usize,
}

type RepositorySkillUpdateRow = (
    String,
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
);

fn build_repository_skill_update_target(
    row: RepositorySkillUpdateRow,
) -> Result<RepositorySkillUpdateTarget> {
    let (
        skill_id,
        slug,
        name,
        canonical_path,
        source_type,
        source_url,
        version,
        metadata_json,
        copy_distribution_count,
    ) = row;

    let repo_url = resolve_github_repo_url(
        repo_url_from_metadata(metadata_json.clone()),
        source_url.clone(),
    )
    .ok_or_else(|| anyhow!("skill does not have an updatable GitHub repository source"))?;

    Ok(RepositorySkillUpdateTarget {
        skill_id,
        name,
        slug,
        canonical_path,
        source_type,
        repo_url,
        manifest_path: metadata_string_field(metadata_json.clone(), "manifestPath"),
        skill_root: metadata_string_field(metadata_json.clone(), "skillRoot"),
        version,
        copy_distribution_count: copy_distribution_count.max(0) as usize,
    })
}

pub struct RepositorySkillRemovalPlan {
    pub skill_id: String,
    pub skill_name: String,
    pub canonical_path: String,
    pub distribution_paths: Vec<String>,
}

pub struct RepositoryStorageEntry {
    pub skill_id: String,
    pub slug: String,
    pub canonical_path: String,
}

pub fn load_skill_source(path: &Path, skill_id: &str) -> Result<SkillSource> {
    let conn = open_connection(path)?;
    conn.query_row(
        "
        SELECT
            COALESCE(s.canonical_path, (
                SELECT target_path
                FROM skill_distributions
                WHERE skill_id = s.id
                ORDER BY created_at ASC
                LIMIT 1
            )) AS source_path,
            COALESCE(s.slug, s.name) AS target_name
        FROM skills s
        WHERE s.id = ?1
        ",
        params![skill_id],
        |row| {
            Ok(SkillSource {
                source_path: row.get(0)?,
                target_name: row.get(1)?,
            })
        },
    )
    .map_err(Into::into)
}

pub fn update_skill_security_status(
    path: &Path,
    skill_id: &str,
    security_level: &str,
    blocked: bool,
    scanned_at: i64,
) -> Result<()> {
    let conn = open_connection(path)?;
    conn.execute(
        "
        UPDATE skills
        SET security_level = ?2,
            blocked = ?3,
            last_scanned_at = ?4,
            updated_at = ?4
        WHERE id = ?1
        ",
        params![skill_id, security_level, blocked as i64, scanned_at],
    )?;
    Ok(())
}

pub fn update_skill_risk_override_state(
    path: &Path,
    skill_id: &str,
    risk_override_applied: bool,
) -> Result<()> {
    let conn = open_connection(path)?;
    let metadata_json: Option<String> = conn.query_row(
        "SELECT metadata_json FROM skills WHERE id = ?1",
        params![skill_id],
        |row| row.get(0),
    )?;

    let mut metadata = metadata_json
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .unwrap_or_else(|| json!({}));
    if !metadata.is_object() {
        metadata = json!({});
    }

    if let Some(object) = metadata.as_object_mut() {
        object.insert(
            "riskOverrideApplied".to_string(),
            Value::Bool(risk_override_applied),
        );
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();
    conn.execute(
        "
        UPDATE skills
        SET metadata_json = ?2,
            updated_at = ?3
        WHERE id = ?1
        ",
        params![skill_id, metadata.to_string(), now],
    )?;

    Ok(())
}

pub fn list_installed_skills(path: &Path) -> Result<Vec<InstalledSkillSummary>> {
    let conn = open_connection(path)?;
    let mut stmt = conn.prepare(
        "
        SELECT id, name, canonical_path, source_url, version, metadata_json
        FROM skills
        WHERE canonical_path IS NOT NULL
        ORDER BY updated_at DESC
        ",
    )?;

    let rows = stmt.query_map([], |row| {
        let metadata_json: Option<String> = row.get(5)?;
        Ok(InstalledSkillSummary {
            skill_id: row.get(0)?,
            name: row.get(1)?,
            canonical_path: row.get(2)?,
            source_url: row.get(3)?,
            repo_url: repo_url_from_metadata(metadata_json.clone()),
            version: row.get(4)?,
            risk_override_applied: risk_override_from_metadata(metadata_json),
        })
    })?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

pub fn list_repository_storage_entries(
    path: &Path,
    canonical_store_dir: &Path,
) -> Result<Vec<RepositoryStorageEntry>> {
    let canonical_root = canonicalize_existing_path(canonical_store_dir)?;
    let conn = open_connection(path)?;
    let mut stmt = conn.prepare(
        "
        SELECT id, slug, canonical_path
        FROM skills
        WHERE canonical_path IS NOT NULL
        ORDER BY installed_at DESC, name ASC
        ",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;

    let mut entries = Vec::new();
    for row in rows {
        let (skill_id, slug, raw_path) = row?;
        let skill_path = PathBuf::from(&raw_path);
        if !skill_path.exists() {
            continue;
        }
        let canonical_skill_path = match canonicalize_existing_path(&skill_path) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if !canonical_skill_path.starts_with(&canonical_root) {
            continue;
        }

        entries.push(RepositoryStorageEntry {
            skill_id,
            slug,
            canonical_path: canonical_skill_path.to_string_lossy().to_string(),
        });
    }

    Ok(entries)
}

pub fn rewrite_repository_storage_paths_with_connection(
    conn: &Connection,
    previous_root: &Path,
    current_root: &Path,
) -> Result<usize> {
    let previous_root = canonicalize_existing_path(previous_root)?;
    let current_root = canonicalize_existing_path(current_root)?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let mut stmt = conn.prepare(
        "
        SELECT id, canonical_path
        FROM skills
        WHERE canonical_path IS NOT NULL
        ",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut updates = Vec::new();
    for row in rows {
        let (skill_id, canonical_path) = row?;
        let current_path = fs::canonicalize(PathBuf::from(&canonical_path))
            .unwrap_or_else(|_| PathBuf::from(&canonical_path));
        let relative = match current_path.strip_prefix(&previous_root) {
            Ok(value) => value.to_path_buf(),
            Err(_) => continue,
        };
        updates.push((
            skill_id,
            current_root.join(relative).to_string_lossy().to_string(),
        ));
    }
    drop(stmt);

    for (skill_id, next_path) in &updates {
        conn.execute(
            "UPDATE skills SET canonical_path = ?2, updated_at = ?3 WHERE id = ?1",
            params![skill_id, next_path, now],
        )?;
    }

    Ok(updates.len())
}

fn canonicalize_existing_path(path: &Path) -> Result<PathBuf> {
    fs::canonicalize(path).with_context(|| format!("failed to canonicalize {}", path.display()))
}

pub fn list_repository_skills(
    path: &Path,
    canonical_store_dir: &Path,
) -> Result<Vec<RepositorySkillSummary>> {
    let canonical_root = canonicalize_existing_path(canonical_store_dir)?;
    let conn = open_connection(path)?;
    let mut stmt = conn.prepare(
        "
        SELECT
            id,
            slug,
            name,
            description,
            source_type,
            source_market,
            source_url,
            installed_at,
            security_level,
            blocked,
            canonical_path,
            metadata_json
        FROM skills
        WHERE canonical_path IS NOT NULL
        ORDER BY installed_at DESC, name ASC
        ",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, i64>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, i64>(9)? != 0,
            row.get::<_, String>(10)?,
            row.get::<_, Option<String>>(11)?,
        ))
    })?;

    let mut skills = Vec::new();
    for row in rows {
        let (
            id,
            slug,
            name,
            description,
            source_type,
            source_market,
            source_url,
            installed_at,
            security_level,
            blocked,
            raw_path,
            metadata_json,
        ) = row?;
        let skill_path = PathBuf::from(&raw_path);
        if !skill_path.exists() {
            continue;
        }
        let canonical_skill_path = match canonicalize_existing_path(&skill_path) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if !canonical_skill_path.starts_with(&canonical_root) {
            continue;
        }

        let risk_override_applied = risk_override_from_metadata(metadata_json.clone());
        skills.push(RepositorySkillSummary {
            id,
            slug,
            name,
            description,
            source_type,
            source_market,
            installed_at,
            security_level,
            blocked,
            risk_override_applied,
            can_update: can_update_from_source(
                repo_url_from_metadata(metadata_json.clone()),
                source_url,
            ),
        });
    }

    Ok(skills)
}

pub fn get_repository_skill_detail(
    path: &Path,
    canonical_store_dir: &Path,
    skill_id: &str,
) -> Result<RepositorySkillDetail> {
    let canonical_root = canonicalize_existing_path(canonical_store_dir)?;
    let conn = open_connection(path)?;
    let row = conn.query_row(
        "
        SELECT
            id,
            slug,
            name,
            description,
            canonical_path,
            source_type,
            source_market,
            source_url,
            installed_at,
            security_level,
            blocked,
            metadata_json
        FROM skills
        WHERE id = ?1 AND canonical_path IS NOT NULL
        ",
        params![skill_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, i64>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, i64>(10)? != 0,
                row.get::<_, Option<String>>(11)?,
            ))
        },
    )?;

    let (
        id,
        slug,
        name,
        description,
        canonical_path,
        source_type,
        source_market,
        source_url,
        installed_at,
        security_level,
        blocked,
        metadata_json,
    ) = row;

    let skill_dir = PathBuf::from(&canonical_path);
    let canonical_skill_dir = canonicalize_existing_path(&skill_dir)?;
    if !canonical_skill_dir.starts_with(&canonical_root) {
        return Err(anyhow!(
            "skill {} is not inside canonical store: {}",
            skill_id,
            canonical_skill_dir.display()
        ));
    }

    let skill_markdown_path = canonical_skill_dir.join("SKILL.md");
    let skill_markdown = fs::read_to_string(&skill_markdown_path).with_context(|| {
        format!(
            "failed to read repository skill markdown {}",
            skill_markdown_path.display()
        )
    })?;
    let display_source_url = display_source_url(&source_type, source_url);
    let can_update = can_update_from_source(
        repo_url_from_metadata(metadata_json.clone()),
        display_source_url.clone(),
    );

    Ok(RepositorySkillDetail {
        id,
        slug,
        name,
        description,
        canonical_path: display_path(&canonical_skill_dir.to_string_lossy()),
        source_type,
        source_market,
        source_url: display_source_url,
        installed_at,
        security_level,
        blocked,
        risk_override_applied: risk_override_from_metadata(metadata_json),
        can_update,
        skill_markdown,
    })
}

pub fn load_repository_skill_update_target(
    path: &Path,
    skill_id: &str,
) -> Result<RepositorySkillUpdateTarget> {
    let conn = open_connection(path)?;
    let row = conn.query_row(
        "
        SELECT
            id,
            slug,
            name,
            canonical_path,
            source_type,
            source_url,
            version,
            metadata_json,
            (
                SELECT COUNT(*)
                FROM skill_distributions
                WHERE skill_id = skills.id AND install_mode = 'copy'
            ) AS copy_distribution_count
        FROM skills
        WHERE id = ?1 AND canonical_path IS NOT NULL
        ",
        params![skill_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, i64>(8)?,
            ))
        },
    )?;
    build_repository_skill_update_target(row)
}

pub fn list_repository_skill_update_targets(
    path: &Path,
) -> Result<Vec<RepositorySkillUpdateTarget>> {
    let conn = open_connection(path)?;
    let mut stmt = conn.prepare(
        "
        SELECT
            id,
            slug,
            name,
            canonical_path,
            source_type,
            source_url,
            version,
            metadata_json,
            COALESCE(copy_stats.copy_distribution_count, 0)
        FROM skills
        LEFT JOIN (
            SELECT
                skill_id,
                COUNT(*) AS copy_distribution_count
            FROM skill_distributions
            WHERE install_mode = 'copy'
            GROUP BY skill_id
        ) AS copy_stats ON copy_stats.skill_id = skills.id
        WHERE canonical_path IS NOT NULL
        ORDER BY updated_at DESC, name ASC
        ",
    )?;

    let mut targets = Vec::new();
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, Option<String>>(7)?,
            row.get::<_, i64>(8)?,
        ))
    })?;

    for row in rows {
        if let Ok(target) = build_repository_skill_update_target(row?) {
            targets.push(target);
        }
    }

    Ok(targets)
}

pub struct UpdateRepositorySkillRecordInput {
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub source_url: String,
    pub repo_url: String,
    pub download_url: Option<String>,
    pub package_ref: Option<String>,
    pub manifest_path: Option<String>,
    pub skill_root: Option<String>,
    pub security_level: String,
    pub blocked: bool,
    pub scanned_at: i64,
}

pub fn update_repository_skill_record_in_tx(
    tx: &Transaction<'_>,
    skill_id: &str,
    input: &UpdateRepositorySkillRecordInput,
) -> Result<()> {
    let updated_at = OffsetDateTime::now_utc().unix_timestamp();

    let metadata_json: Option<String> = tx.query_row(
        "SELECT metadata_json FROM skills WHERE id = ?1",
        params![skill_id],
        |row| row.get(0),
    )?;
    let mut metadata = metadata_from_json(metadata_json);
    metadata["repoUrl"] = Value::String(input.repo_url.clone());
    metadata["downloadUrl"] = input
        .download_url
        .clone()
        .map(Value::String)
        .unwrap_or(Value::Null);
    metadata["packageRef"] = input
        .package_ref
        .clone()
        .map(Value::String)
        .unwrap_or(Value::Null);
    metadata["manifestPath"] = input
        .manifest_path
        .clone()
        .map(Value::String)
        .unwrap_or(Value::Null);
    metadata["skillRoot"] = input
        .skill_root
        .clone()
        .map(Value::String)
        .unwrap_or(Value::Null);
    metadata["riskOverrideApplied"] = Value::Bool(false);

    tx.execute(
        "
        UPDATE skills
        SET
            description = ?2,
            source_url = ?3,
            version = ?4,
            author = ?5,
            security_level = ?6,
            blocked = ?7,
            updated_at = ?8,
            last_scanned_at = ?9,
            metadata_json = ?10
        WHERE id = ?1
        ",
        params![
            skill_id,
            input.description,
            input.source_url,
            input.version,
            input.author,
            input.security_level,
            input.blocked as i64,
            updated_at,
            input.scanned_at,
            metadata.to_string(),
        ],
    )?;

    Ok(())
}

pub fn load_repository_skill_removal_plan(
    path: &Path,
    canonical_store_dir: &Path,
    skill_id: &str,
) -> Result<RepositorySkillRemovalPlan> {
    let canonical_root = canonicalize_existing_path(canonical_store_dir)?;
    let conn = open_connection(path)?;
    let (skill_name, canonical_path): (String, String) = conn.query_row(
        "SELECT name, canonical_path FROM skills WHERE id = ?1 AND canonical_path IS NOT NULL",
        params![skill_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let canonical_skill_dir = canonicalize_existing_path(Path::new(&canonical_path))?;
    if !canonical_skill_dir.starts_with(&canonical_root) {
        return Err(anyhow!(
            "skill {} is not inside canonical store: {}",
            skill_id,
            canonical_skill_dir.display()
        ));
    }

    let mut stmt = conn.prepare(
        "
        SELECT target_path
        FROM skill_distributions
        WHERE skill_id = ?1
        ORDER BY created_at ASC
        ",
    )?;
    let rows = stmt.query_map(params![skill_id], |row| row.get::<_, String>(0))?;
    let distribution_paths = rows
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .map(|path| display_path(&path))
        .collect::<Vec<_>>();

    Ok(RepositorySkillRemovalPlan {
        skill_id: skill_id.to_string(),
        skill_name,
        canonical_path: display_path(&canonical_skill_dir.to_string_lossy()),
        distribution_paths,
    })
}

pub fn delete_repository_skill(path: &Path, skill_id: &str) -> Result<()> {
    let mut conn = open_connection(path)?;
    let tx = conn.transaction()?;

    tx.execute(
        "DELETE FROM skill_distributions WHERE skill_id = ?1",
        params![skill_id],
    )?;
    tx.execute(
        "DELETE FROM security_reports WHERE skill_id = ?1",
        params![skill_id],
    )?;
    tx.execute("DELETE FROM skills WHERE id = ?1", params![skill_id])?;

    tx.commit()?;
    Ok(())
}


#[cfg(test)]
#[path = "skills/tests.rs"]
mod tests;
