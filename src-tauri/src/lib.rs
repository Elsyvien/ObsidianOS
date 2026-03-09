mod ai;
mod db;
mod markdown;
mod models;

use std::path::PathBuf;

use anyhow::{Context, Result};
use db::Database;
use models::{
    AiNoteInsight, AiSettingsInput, CourseConfigInput, DashboardData,
    FlashcardGenerationRequest, FlashcardGenerationResult, NoteDetails, RevisionNoteRequest,
    RevisionNoteResult, ScanReport, ValidationResult, WorkspaceSnapshot,
};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

#[derive(Clone)]
struct AppState {
    db_path: PathBuf,
    export_dir: PathBuf,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunScanResponse {
    workspace: WorkspaceSnapshot,
    report: ScanReport,
}

#[tauri::command]
async fn load_workspace(state: State<'_, AppState>) -> Result<WorkspaceSnapshot, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.load_workspace()
    })
    .await
}

#[tauri::command]
async fn connect_vault(
    vault_path: String,
    state: State<'_, AppState>,
) -> Result<WorkspaceSnapshot, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.connect_vault(&vault_path)?;
        database.load_workspace()
    })
    .await
}

#[tauri::command]
async fn disconnect_vault(state: State<'_, AppState>) -> Result<WorkspaceSnapshot, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.disconnect_vault()?;
        database.load_workspace()
    })
    .await
}

#[tauri::command]
async fn save_course_config(
    input: CourseConfigInput,
    state: State<'_, AppState>,
) -> Result<WorkspaceSnapshot, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.save_course_config(input)?;
        database.load_workspace()
    })
    .await
}

#[tauri::command]
async fn delete_course(
    course_id: String,
    state: State<'_, AppState>,
) -> Result<WorkspaceSnapshot, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.delete_course(&course_id)?;
        database.load_workspace()
    })
    .await
}

#[tauri::command]
async fn run_scan(state: State<'_, AppState>) -> Result<RunScanResponse, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        let report = database.run_scan()?;
        let workspace = database.load_workspace()?;
        Ok(RunScanResponse { workspace, report })
    })
    .await
}

#[tauri::command]
async fn get_dashboard(
    course_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Option<DashboardData>, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.get_dashboard(course_id)
    })
    .await
}

#[tauri::command]
async fn get_note_details(
    note_id: String,
    state: State<'_, AppState>,
) -> Result<NoteDetails, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.get_note_details(&note_id)
    })
    .await
}

#[tauri::command]
async fn generate_note_ai_insight(
    note_id: String,
    state: State<'_, AppState>,
) -> Result<AiNoteInsight, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.generate_note_ai_insight(&note_id)
    })
    .await
}

#[tauri::command]
async fn save_ai_settings(
    input: AiSettingsInput,
    state: State<'_, AppState>,
) -> Result<WorkspaceSnapshot, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.save_ai_settings(input)?;
        database.load_workspace()
    })
    .await
}

#[tauri::command]
async fn validate_ai_settings(
    input: AiSettingsInput,
    state: State<'_, AppState>,
) -> Result<ValidationResult, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.validate_ai_settings(input)
    })
    .await
}

#[tauri::command]
async fn generate_flashcards(
    request: FlashcardGenerationRequest,
    state: State<'_, AppState>,
) -> Result<FlashcardGenerationResult, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.generate_flashcards(request, &state.export_dir)
    })
    .await
}

#[tauri::command]
async fn generate_revision_note(
    request: RevisionNoteRequest,
    state: State<'_, AppState>,
) -> Result<RevisionNoteResult, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.generate_revision_note(request)
    })
    .await
}

async fn blocking<T, F>(operation: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || operation().map_err(|error| error.to_string()))
        .await
        .map_err(|error| error.to_string())?
}

fn build_state(app: &AppHandle) -> Result<AppState> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .context("failed to resolve app data directory")?;
    let export_dir = app_data_dir.join("exports");
    Ok(AppState {
        db_path: app_data_dir.join("obsidian-exam-os.sqlite"),
        export_dir,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = build_state(app.handle())?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_workspace,
            connect_vault,
            disconnect_vault,
            save_course_config,
            delete_course,
            run_scan,
            get_dashboard,
            get_note_details,
            generate_note_ai_insight,
            save_ai_settings,
            validate_ai_settings,
            generate_flashcards,
            generate_revision_note
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
