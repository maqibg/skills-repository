use tauri::{AppHandle, State};

use crate::{
    domain::{app_state::AppState, types::{AppSettings, ScanSkillsRequest, TaskHandle}},
    services::{bootstrap, scan, settings},
    tasks,
};

#[tauri::command]
pub fn bootstrap_app(state: State<'_, AppState>) -> Result<crate::domain::types::BootstrapPayload, String> {
    log::info!("bootstrap_app invoked");
    bootstrap::bootstrap_payload(&state, env!("CARGO_PKG_VERSION").to_string())
        .map(|payload| {
            log::info!("bootstrap_app resolved");
            payload
        })
        .map_err(|error| {
            log::error!("bootstrap_app failed: {}", error);
            error.to_string()
        })
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    log::info!("get_settings invoked");
    settings::get_settings(&state, bootstrap::normalize_language(&bootstrap::system_locale()))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: AppSettings) -> Result<AppSettings, String> {
    log::info!("save_settings invoked");
    settings::save_settings(&state, &settings).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn scan_skills(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ScanSkillsRequest,
) -> Result<TaskHandle, String> {
    log::info!("scan_skills invoked");
    let task = tasks::new_task_handle("scan");
    let app_handle = app.clone();
    let task_handle = task.clone();
    let state = state.inner().clone();

    tasks::emit_progress(
        &app,
        &task,
        "queued",
        "prepare",
        0,
        3,
        "Scan task queued",
    );

    tauri::async_runtime::spawn(async move {
        tasks::emit_progress(
            &app_handle,
            &task_handle,
            "running",
            "scan",
            1,
            3,
            "Scanning configured skill roots",
        );

        match scan::scan_skills(state.agent_registry.as_ref(), &request) {
            Ok(result) => {
                tasks::emit_progress(
                    &app_handle,
                    &task_handle,
                    "running",
                    "persist",
                    2,
                    3,
                    "Collected scan result payload",
                );
                tasks::emit_completed(
                    &app_handle,
                    &task_handle,
                    "cleanup",
                    "Scan completed",
                    result,
                );
            }
            Err(error) => {
                tasks::emit_failed(
                    &app_handle,
                    &task_handle,
                    "scan",
                    &error.to_string(),
                );
            }
        }
    });

    Ok(task)
}
