mod adapters;
mod commands;
mod domain;
mod repositories;
mod security;
mod services;
mod tasks;
mod utils;

use commands::app::{bootstrap_app, get_settings, save_settings, scan_skills};
use domain::app_state::AppState;
use repositories::db::run_migrations;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let app_handle = app.handle();
            let state = AppState::new(&app_handle)?;
            run_migrations(&state.paths.db_file)?;
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bootstrap_app,
            get_settings,
            save_settings,
            scan_skills,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
