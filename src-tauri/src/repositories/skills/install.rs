use super::*;

pub fn save_installed_skill(
    path: &Path,
    request: &InstallSkillRequest,
    canonical_path: &str,
    security_level: &str,
    blocked: bool,
) -> Result<String> {
    let conn = open_connection(path)?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let skill_id = Uuid::new_v4().to_string();
    let persisted_source_type = normalize_persisted_source_type(&request.source_type);
    let persisted_source_market = normalize_persisted_source_market(request, persisted_source_type);

    conn.execute(
        "
        INSERT INTO skills (
            id,
            slug,
            name,
            description,
            source_type,
            source_market,
            source_url,
            version,
            author,
            canonical_path,
            file_hash,
            size_bytes,
            management_mode,
            security_level,
            blocked,
            installed_at,
            updated_at,
            last_scanned_at,
            metadata_json
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, NULL, 0, 'managed', ?11, ?12, ?13, ?14, ?15, ?16)
        ",
        params![
            skill_id,
            request.slug,
            request.name,
            request.description,
            persisted_source_type,
            persisted_source_market,
            request.source_url,
            request.version,
            request.author,
            canonical_path,
            security_level,
            blocked as i64,
            now,
            now,
            now,
            json!({
                "sourceType": request.source_type,
                "repoUrl": request.repo_url,
                "downloadUrl": request.download_url,
                "packageRef": request.package_ref,
                "manifestPath": request.manifest_path,
                "skillRoot": request.skill_root,
                "requestedTargets": request.requested_targets,
            })
            .to_string(),
        ],
    )?;

    Ok(skill_id)
}

pub fn repository_skill_slug_exists(path: &Path, slug: &str) -> Result<bool> {
    let conn = open_connection(path)?;
    let exists = conn.query_row(
        "
        SELECT EXISTS(
            SELECT 1
            FROM skills
            WHERE slug = ?1 AND canonical_path IS NOT NULL
        )
        ",
        params![slug],
        |row| row.get::<_, i64>(0),
    )?;

    Ok(exists != 0)
}