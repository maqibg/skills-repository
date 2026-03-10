use anyhow::{anyhow, Result};
use rusqlite::{params, OptionalExtension, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use std::path::Path;

use crate::domain::types::{SaveTemplateRequest, TemplateItem, TemplateRecord};

use super::db::open_connection;

fn load_template_items(tx: &Transaction<'_>, template_id: &str) -> Result<Vec<TemplateItem>> {
    let mut stmt = tx.prepare(
        "
        SELECT id, skill_ref_type, skill_ref, display_name, required, order_index
        FROM template_items
        WHERE template_id = ?1
        ORDER BY order_index ASC
        ",
    )?;

    let rows = stmt.query_map(params![template_id], |row| {
        Ok(TemplateItem {
            id: row.get(0)?,
            skill_ref_type: row.get(1)?,
            skill_ref: row.get(2)?,
            display_name: row.get(3)?,
            required: row.get::<_, i64>(4)? != 0,
            order_index: row.get::<_, i64>(5)? as u32,
        })
    })?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn load_template_record(tx: &Transaction<'_>, template_id: &str) -> Result<Option<TemplateRecord>> {
    let record = tx
        .query_row(
            "
            SELECT
                id,
                name,
                description,
                tags_json,
                target_agents_json,
                scope,
                is_builtin,
                created_at,
                updated_at
            FROM templates
            WHERE id = ?1
            ",
            params![template_id],
            |row| {
                let tags_json: String = row.get(3)?;
                let target_agents_json: String = row.get(4)?;

                Ok(TemplateRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    tags: serde_json::from_str(&tags_json).unwrap_or_default(),
                    target_agents: serde_json::from_str(&target_agents_json).unwrap_or_default(),
                    scope: row.get(5)?,
                    is_builtin: row.get::<_, i64>(6)? != 0,
                    items: Vec::new(),
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
        .optional()?;

    let Some(mut record) = record else {
        return Ok(None);
    };
    record.items = load_template_items(tx, &record.id)?;
    Ok(Some(record))
}

pub fn list_templates(path: &Path) -> Result<Vec<TemplateRecord>> {
    let mut conn = open_connection(path)?;
    let tx = conn.transaction()?;
    let mut stmt = tx.prepare(
        "
        SELECT id
        FROM templates
        ORDER BY updated_at DESC, created_at DESC
        ",
    )?;

    let template_ids = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut records = Vec::with_capacity(template_ids.len());
    for template_id in template_ids {
        if let Some(record) = load_template_record(&tx, &template_id)? {
            records.push(record);
        }
    }

    drop(stmt);
    tx.commit()?;
    Ok(records)
}

pub fn get_template(path: &Path, template_id: &str) -> Result<Option<TemplateRecord>> {
    let mut conn = open_connection(path)?;
    let tx = conn.transaction()?;
    let record = load_template_record(&tx, template_id)?;
    tx.commit()?;
    Ok(record)
}

pub fn save_template(path: &Path, request: &SaveTemplateRequest) -> Result<TemplateRecord> {
    let mut conn = open_connection(path)?;
    let tx = conn.transaction()?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let template_id = request
        .id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let existing = tx
        .query_row(
            "
            SELECT created_at, is_builtin
            FROM templates
            WHERE id = ?1
            ",
            params![template_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)? != 0)),
        )
        .optional()?;

    let created_at = existing.map(|(created_at, _)| created_at).unwrap_or(now);
    let is_builtin = existing.map(|(_, is_builtin)| is_builtin).unwrap_or(false);

    tx.execute(
        "
        INSERT INTO templates (
            id,
            name,
            description,
            tags_json,
            target_agents_json,
            scope,
            is_builtin,
            created_at,
            updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            description = excluded.description,
            tags_json = excluded.tags_json,
            target_agents_json = excluded.target_agents_json,
            scope = excluded.scope,
            updated_at = excluded.updated_at
        ",
        params![
            template_id,
            request.name,
            request.description,
            serde_json::to_string(&request.tags)?,
            serde_json::to_string(&Vec::<String>::new())?,
            "user",
            is_builtin as i64,
            created_at,
            now,
        ],
    )?;

    tx.execute(
        "DELETE FROM template_items WHERE template_id = ?1",
        params![template_id],
    )?;

    for (index, item) in request.items.iter().enumerate() {
        let item_id = Uuid::new_v4().to_string();
        tx.execute(
            "
            INSERT INTO template_items (
                id,
                template_id,
                skill_ref_type,
                skill_ref,
                display_name,
                required,
                order_index
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            params![
                item_id,
                template_id,
                item.skill_ref_type,
                item.skill_ref,
                item.display_name,
                1_i64,
                item.order_index.unwrap_or(index as u32) as i64,
            ],
        )?;
    }

    let saved = load_template_record(&tx, &template_id)?
        .ok_or_else(|| anyhow!("failed to reload saved template"))?;
    tx.commit()?;
    Ok(saved)
}

pub fn delete_template(path: &Path, template_id: &str) -> Result<()> {
    let mut conn = open_connection(path)?;
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM template_items WHERE template_id = ?1",
        params![template_id],
    )?;
    let deleted = tx.execute("DELETE FROM templates WHERE id = ?1", params![template_id])?;
    if deleted == 0 {
        return Err(anyhow!("template {} does not exist", template_id));
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::db::run_migrations;
    use tempfile::tempdir;

    #[test]
    fn saves_lists_and_deletes_templates() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("templates.db");
        run_migrations(&db_path).unwrap();

        let saved = save_template(
            &db_path,
            &SaveTemplateRequest {
                id: None,
                name: "Rust Project".into(),
                description: Some("Rust starter".into()),
                tags: vec!["rust".into(), "backend".into()],
                items: Vec::new(),
            },
        )
        .unwrap();

        let listed = list_templates(&db_path).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "Rust Project");
        assert!(listed[0].items.is_empty());

        let loaded = get_template(&db_path, &saved.id).unwrap().unwrap();
        assert!(loaded.target_agents.is_empty());
        assert_eq!(loaded.scope, "user");

        delete_template(&db_path, &saved.id).unwrap();
        assert!(list_templates(&db_path).unwrap().is_empty());
    }

    #[test]
    fn saves_template_items_and_replaces_them_on_update() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("template-items.db");
        run_migrations(&db_path).unwrap();

        let saved = save_template(
            &db_path,
            &SaveTemplateRequest {
                id: None,
                name: "Vue Project".into(),
                description: Some("Vue starter".into()),
                tags: vec!["vue".into()],
                items: vec![
                    crate::domain::types::SaveTemplateItemRequest {
                        skill_ref_type: "repository_skill".into(),
                        skill_ref: "skill-a".into(),
                        display_name: Some("Skill A".into()),
                        order_index: Some(0),
                    },
                    crate::domain::types::SaveTemplateItemRequest {
                        skill_ref_type: "repository_skill".into(),
                        skill_ref: "skill-b".into(),
                        display_name: Some("Skill B".into()),
                        order_index: Some(1),
                    },
                ],
            },
        )
        .unwrap();

        assert_eq!(saved.items.len(), 2);
        assert_eq!(saved.items[0].skill_ref, "skill-a");
        assert_eq!(saved.items[1].skill_ref, "skill-b");

        let updated = save_template(
            &db_path,
            &SaveTemplateRequest {
                id: Some(saved.id.clone()),
                name: "Vue Project".into(),
                description: Some("Vue starter updated".into()),
                tags: vec!["vue".into(), "frontend".into()],
                items: vec![crate::domain::types::SaveTemplateItemRequest {
                    skill_ref_type: "repository_skill".into(),
                    skill_ref: "skill-c".into(),
                    display_name: Some("Skill C".into()),
                    order_index: Some(0),
                }],
            },
        )
        .unwrap();

        assert_eq!(updated.items.len(), 1);
        assert_eq!(updated.items[0].skill_ref, "skill-c");

        let loaded = get_template(&db_path, &saved.id).unwrap().unwrap();
        assert_eq!(loaded.items.len(), 1);
        assert_eq!(loaded.items[0].skill_ref, "skill-c");
    }
}
