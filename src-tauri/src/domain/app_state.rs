use super::agent_registry::AgentRegistry;
use anyhow::{Context, Result};
use std::{path::PathBuf, sync::Arc};
use tauri::AppHandle;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub db_file: PathBuf,
    pub temp_dir: PathBuf,
    pub canonical_store_dir: PathBuf,
}

impl AppPaths {
    fn resolve_portable_data_root() -> Result<PathBuf> {
        let exe_path =
            std::env::current_exe().context("failed to resolve current executable path")?;
        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("current executable path has no parent directory"))?;
        Ok(exe_dir.join("data"))
    }

    pub fn from_app(_app: &AppHandle) -> Result<Self> {
        let data_root =
            Self::resolve_portable_data_root().context("failed to resolve portable data root")?;
        let db_dir = data_root.join("db");
        let cache_dir = data_root.join("cache").join("market");
        let temp_dir = data_root.join("tmp").join("tasks");
        let canonical_store_dir = data_root.join("skills");
        let db_file = db_dir.join("skills-manager.db");

        std::fs::create_dir_all(&db_dir)?;
        std::fs::create_dir_all(&cache_dir)?;
        std::fs::create_dir_all(&temp_dir)?;
        std::fs::create_dir_all(&canonical_store_dir)?;

        Ok(Self {
            db_file,
            temp_dir,
            canonical_store_dir,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub paths: AppPaths,
    pub agent_registry: Arc<AgentRegistry>,
}

impl AppState {
    pub fn new(app: &AppHandle) -> Result<Self> {
        Ok(Self {
            paths: AppPaths::from_app(app)?,
            agent_registry: Arc::new(AgentRegistry::new()),
        })
    }
}
