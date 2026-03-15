use super::*;
use crate::{
    domain::types::{DistributionRequest, SecurityReport},
    repositories::{
        db::run_migrations, distributions as distributions_repository,
        security as security_repository,
    },
};
use tempfile::tempdir;

    fn seed_skill(
        root: &Path,
        source_type: &str,
        source_market: Option<&str>,
    ) -> (PathBuf, String) {
        let app_data_dir = root.join("app-data");
        let db_dir = app_data_dir.join("db");
        let canonical_store_dir = app_data_dir.join("skills");
        fs::create_dir_all(&db_dir).unwrap();
        fs::create_dir_all(&canonical_store_dir).unwrap();
        let db_path = db_dir.join("skills-manager.db");
        run_migrations(&db_path).unwrap();

        let skill_dir = canonical_store_dir.join("demo-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# demo skill").unwrap();

        let request = InstallSkillRequest {
            provider: source_market.unwrap_or("local").to_string(),
            market_skill_id: "demo".into(),
            source_type: "catalog".into(),
            source_url: "https://example.com/demo".into(),
            repo_url: Some("https://example.com/demo".into()),
            download_url: None,
            package_ref: Some("example/demo@skills/demo-skill".into()),
            manifest_path: Some("skills/demo-skill/SKILL.md".into()),
            skill_root: Some("skills/demo-skill".into()),
            name: "Demo Skill".into(),
            slug: "demo-skill".into(),
            description: Some("Improve confusing UX copy and labels.".into()),
            version: Some("main".into()),
            author: Some("tester".into()),
            requested_targets: Vec::<DistributionRequest>::new(),
        };
        let skill_id = save_installed_skill(
            &db_path,
            &request,
            &skill_dir.to_string_lossy(),
            "safe",
            false,
        )
        .unwrap();

        let conn = open_connection(&db_path).unwrap();
        conn.execute(
            "UPDATE skills SET source_type = ?2, source_market = ?3 WHERE id = ?1",
            params![skill_id, source_type, source_market],
        )
        .unwrap();

        (db_path, skill_id)
    }

    #[test]
    fn lists_only_existing_skills_in_canonical_store() {
        let dir = tempdir().unwrap();
        let (db_path, skill_id) = seed_skill(dir.path(), "market", Some("github"));
        let canonical_store_dir = dir.path().join("app-data").join("skills");

        let skills = list_repository_skills(&db_path, &canonical_store_dir).unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].id, skill_id);
        assert_eq!(skills[0].source_market.as_deref(), Some("github"));
        assert_eq!(
            skills[0].description.as_deref(),
            Some("Improve confusing UX copy and labels.")
        );
    }

    #[test]
    fn loads_repository_skill_detail_with_markdown() {
        let dir = tempdir().unwrap();
        let (db_path, skill_id) = seed_skill(dir.path(), "market", Some("github"));
        let canonical_store_dir = dir.path().join("app-data").join("skills");

        let detail =
            get_repository_skill_detail(&db_path, &canonical_store_dir, &skill_id).unwrap();

        assert_eq!(detail.id, skill_id);
        assert!(detail.skill_markdown.contains("demo skill"));
        assert_eq!(detail.source_market.as_deref(), Some("github"));
        assert_eq!(
            detail.description.as_deref(),
            Some("Improve confusing UX copy and labels.")
        );
    }

    #[test]
    fn marks_only_github_backed_skills_as_updatable() {
        let dir = tempdir().unwrap();
        let (db_path, github_skill_id) = seed_skill(dir.path(), "market", Some("github"));
        let (_db_path, local_skill_id) = seed_skill(dir.path(), "local", None);
        let canonical_store_dir = dir.path().join("app-data").join("skills");
        let conn = open_connection(&db_path).unwrap();
        conn.execute(
            "UPDATE skills SET source_url = ?2, metadata_json = ?3 WHERE id = ?1",
            params![
                github_skill_id,
                "https://github.com/demo/repo/tree/main/skills/demo-skill",
                json!({
                    "repoUrl": "https://github.com/demo/repo",
                    "manifestPath": "skills/demo-skill/SKILL.md",
                    "skillRoot": "skills/demo-skill",
                })
                .to_string(),
            ],
        )
        .unwrap();
        conn.execute(
            "UPDATE skills SET source_url = ?2, metadata_json = ?3 WHERE id = ?1",
            params![
                local_skill_id,
                "E:/skills/demo-skill",
                json!({
                    "repoUrl": Value::Null,
                })
                .to_string(),
            ],
        )
        .unwrap();

        let skills = list_repository_skills(&db_path, &canonical_store_dir).unwrap();
        let github_skill = skills
            .iter()
            .find(|item| item.id == github_skill_id)
            .unwrap();
        let local_skill = skills
            .iter()
            .find(|item| item.id == local_skill_id)
            .unwrap();
        assert!(github_skill.can_update);
        assert!(!local_skill.can_update);

        let github_detail =
            get_repository_skill_detail(&db_path, &canonical_store_dir, &github_skill_id).unwrap();
        let local_detail =
            get_repository_skill_detail(&db_path, &canonical_store_dir, &local_skill_id).unwrap();
        assert!(github_detail.can_update);
        assert!(!local_detail.can_update);
    }

    #[test]
    fn deletes_skill_distributions_and_reports_from_database() {
        let dir = tempdir().unwrap();
        let (db_path, skill_id) = seed_skill(dir.path(), "market", Some("github"));
        let target_path = dir.path().join("global").join("demo-skill");
        fs::create_dir_all(target_path.parent().unwrap()).unwrap();
        fs::create_dir_all(&target_path).unwrap();
        fs::write(target_path.join("SKILL.md"), "# distributed").unwrap();

        distributions_repository::save_distribution(
            &db_path,
            &skill_id,
            "global",
            "Codex",
            &target_path.to_string_lossy(),
            "copy",
            "active",
        )
        .unwrap();

        security_repository::save_security_report(
            &db_path,
            &SecurityReport {
                id: "report-1".into(),
                skill_id: Some(skill_id.clone()),
                skill_name: Some("Demo Skill".into()),
                source_path: Some(target_path.to_string_lossy().to_string()),
                scan_scope: "canonical_store".into(),
                level: "safe".into(),
                score: 100,
                blocked: false,
                issues: Vec::new(),
                recommendations: Vec::new(),
                scanned_files: vec![target_path.join("SKILL.md").to_string_lossy().to_string()],
                category_breakdown: Vec::new(),
                blocking_reasons: Vec::new(),
                engine_version: "phase2-rules-v1".into(),
                scanned_at: 100,
            },
        )
        .unwrap();

        delete_repository_skill(&db_path, &skill_id).unwrap();
        let conn = open_connection(&db_path).unwrap();
        let skill_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM skills WHERE id = ?1",
                params![skill_id],
                |row| row.get(0),
            )
            .unwrap();
        let distribution_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM skill_distributions", [], |row| {
                row.get(0)
            })
            .unwrap();
        let report_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM security_reports", [], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(skill_count, 0);
        assert_eq!(distribution_count, 0);
        assert_eq!(report_count, 0);
    }