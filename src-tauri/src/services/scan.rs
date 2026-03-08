use anyhow::Result;
use std::{collections::HashMap, path::{Path, PathBuf}};
use time::OffsetDateTime;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::domain::{
    agent_registry::AgentRegistry,
    types::{DuplicateGroup, ProjectRecord, ScanSkillsRequest, ScanSkillsResult, SkillRecord},
};

fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

fn canonical_string(path: &Path) -> String {
    std::fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string()
}

fn discover_skill_dirs(root: &Path) -> Vec<PathBuf> {
    if !root.exists() {
        return Vec::new();
    }

    WalkDir::new(root)
        .min_depth(1)
        .max_depth(4)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() && entry.file_name() == "SKILL.md")
        .filter_map(|entry| entry.path().parent().map(|path| path.to_path_buf()))
        .collect()
}

pub fn scan_skills(registry: &AgentRegistry, request: &ScanSkillsRequest) -> Result<ScanSkillsResult> {
    let mut skills = Vec::new();
    let mut projects = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();
    let now = OffsetDateTime::now_utc().unix_timestamp();

    if request.include_system {
        if let Some(home) = home_dir() {
            for agent in registry.agents() {
                for relative in &agent.global_paths {
                    let root = home.join(relative);
                    for skill_dir in discover_skill_dirs(&root) {
                        let path = canonical_string(&skill_dir);
                        if !seen_paths.insert(path.clone()) {
                            continue;
                        }
                        skills.push(SkillRecord {
                            id: Uuid::new_v4().to_string(),
                            name: skill_dir.file_name().unwrap_or_default().to_string_lossy().to_string(),
                            path,
                            agent: agent.label.clone(),
                            scope: "system".into(),
                            source: "discovered".into(),
                            managed: false,
                            project_root: None,
                            last_seen_at: now,
                        });
                    }
                }
            }
        }
    }

    if request.include_projects {
        for root in &request.project_roots {
            let project_root = PathBuf::from(root);
            if !project_root.exists() {
                continue;
            }

            projects.push(ProjectRecord {
                id: Uuid::new_v4().to_string(),
                name: project_root.file_name().unwrap_or_default().to_string_lossy().to_string(),
                root_path: canonical_string(&project_root),
            });

            for agent in registry.agents() {
                for relative in &agent.project_paths {
                    let candidate = project_root.join(relative);
                    for skill_dir in discover_skill_dirs(&candidate) {
                        let path = canonical_string(&skill_dir);
                        if !seen_paths.insert(path.clone()) {
                            continue;
                        }
                        skills.push(SkillRecord {
                            id: Uuid::new_v4().to_string(),
                            name: skill_dir.file_name().unwrap_or_default().to_string_lossy().to_string(),
                            path,
                            agent: agent.label.clone(),
                            scope: "project".into(),
                            source: "discovered".into(),
                            managed: false,
                            project_root: Some(canonical_string(&project_root)),
                            last_seen_at: now,
                        });
                    }
                }
            }
        }
    }

    for root in &request.custom_roots {
        let custom_root = PathBuf::from(root);
        for skill_dir in discover_skill_dirs(&custom_root) {
            let path = canonical_string(&skill_dir);
            if !seen_paths.insert(path.clone()) {
                continue;
            }
            skills.push(SkillRecord {
                id: Uuid::new_v4().to_string(),
                name: skill_dir.file_name().unwrap_or_default().to_string_lossy().to_string(),
                path,
                agent: "Custom".into(),
                scope: "custom".into(),
                source: "discovered".into(),
                managed: false,
                project_root: None,
                last_seen_at: now,
            });
        }
    }

    let mut duplicates_map: HashMap<String, Vec<String>> = HashMap::new();
    for skill in &skills {
        duplicates_map
            .entry(skill.name.clone())
            .or_default()
            .push(skill.path.clone());
    }

    let duplicates = duplicates_map
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(name, paths)| DuplicateGroup { name, paths })
        .collect();

    Ok(ScanSkillsResult {
        skills,
        distributions: Vec::new(),
        duplicates,
        projects,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn finds_project_level_skills() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join(".claude/skills/python-helper");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# test").unwrap();

        let registry = AgentRegistry::new();
        let result = scan_skills(
            &registry,
            &ScanSkillsRequest {
                include_system: false,
                include_projects: true,
                project_roots: vec![dir.path().to_string_lossy().to_string()],
                custom_roots: vec![],
            },
        )
        .unwrap();

        assert_eq!(result.skills.len(), 1);
        assert_eq!(result.skills[0].name, "python-helper");
        assert_eq!(result.skills[0].scope, "project");
    }
}
