mod ai;
mod db;
mod markdown;
mod models;

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use db::Database;
use models::{
    AiCourseSummary, AiNoteInsight, AiSettingsInput, ApplyExamReviewActionsRequest, ChatScope,
    ChatThreadDetails, ChatThreadSummary, CourseConfigInput, CreateChatThreadRequest,
    DashboardData, ExamAttemptResult, ExamBuilderInput, ExamDetails, ExamSubmissionRequest,
    ExamWorkspaceSnapshot, FlashcardGenerationRequest, FlashcardGenerationResult, FormulaBrief,
    FormulaDetails, FormulaWorkspaceSnapshot, GenerateFormulaBriefRequest, NoteDetails,
    RevisionNoteRequest, RevisionNoteResult, ScanReport, SendChatMessageRequest, ValidationResult,
    WorkspaceSnapshot,
};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

#[derive(Clone)]
struct AppState {
    db_path: PathBuf,
    export_dir: PathBuf,
    active_ai_courses: Arc<Mutex<HashSet<String>>>,
    active_exam_courses: Arc<Mutex<HashSet<String>>>,
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
    let (report, auto_course_id) = blocking({
        let state = state.clone();
        move || {
            let database = Database::open(&state.db_path)?;
            let report = database.run_scan()?;
            let workspace = database.load_workspace()?;
            let auto_course_id = workspace.selected_course_id.clone().filter(|_| {
                workspace
                    .ai_settings
                    .as_ref()
                    .map(|settings| settings.enabled)
                    .unwrap_or(false)
            });

            if let Some(course_id) = auto_course_id.as_ref() {
                let _ = database.queue_ai_enrichment(course_id, false)?;
            }

            Ok((report, auto_course_id))
        }
    })
    .await?;

    if let Some(course_id) = auto_course_id {
        spawn_ai_enrichment_thread(state.clone(), course_id, false)?;
    }

    let workspace = blocking(move || {
        let database = Database::open(&state.db_path)?;
        let workspace = database.load_workspace()?;
        Ok(workspace)
    })
    .await?;

    Ok(RunScanResponse { workspace, report })
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
async fn start_ai_enrichment(
    course_id: String,
    force: Option<bool>,
    state: State<'_, AppState>,
) -> Result<AiCourseSummary, String> {
    let state = state.inner().clone();
    let force = force.unwrap_or(false);
    let course_id_for_thread = course_id.clone();

    let summary = blocking({
        let state = state.clone();
        let course_id = course_id.clone();
        move || {
            let database = Database::open(&state.db_path)?;
            database.queue_ai_enrichment(&course_id, force)
        }
    })
    .await?;

    spawn_ai_enrichment_thread(state, course_id_for_thread, force)?;

    Ok(summary)
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

#[tauri::command]
async fn get_exam_workspace(
    course_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Option<ExamWorkspaceSnapshot>, String> {
    let state = state.inner().clone();
    let workspace = blocking({
        let state = state.clone();
        let course_id = course_id.clone();
        move || {
            let database = Database::open(&state.db_path)?;
            database.get_exam_workspace(course_id)
        }
    })
    .await?;

    if let Some(course_id) = workspace
        .as_ref()
        .map(|snapshot| snapshot.course_id.clone())
    {
        let should_spawn = blocking({
            let state = state.clone();
            let course_id = course_id.clone();
            move || {
                let database = Database::open(&state.db_path)?;
                database.has_pending_exam_jobs(&course_id)
            }
        })
        .await?;

        if should_spawn {
            spawn_exam_generation_thread(state, course_id)?;
        }
    }

    Ok(workspace)
}

#[tauri::command]
async fn add_exam_source_notes(
    course_id: String,
    note_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<ExamWorkspaceSnapshot, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.add_exam_source_notes(&course_id, &note_ids)
    })
    .await
}

#[tauri::command]
async fn remove_exam_source_notes(
    course_id: String,
    note_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<ExamWorkspaceSnapshot, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.remove_exam_source_notes(&course_id, &note_ids)
    })
    .await
}

#[tauri::command]
async fn clear_exam_source_queue(
    course_id: String,
    state: State<'_, AppState>,
) -> Result<ExamWorkspaceSnapshot, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.clear_exam_source_queue(&course_id)
    })
    .await
}

#[tauri::command]
async fn queue_exams(
    request: ExamBuilderInput,
    state: State<'_, AppState>,
) -> Result<ExamWorkspaceSnapshot, String> {
    let state = state.inner().clone();
    let course_id = request.course_id.clone();
    let workspace = blocking({
        let state = state.clone();
        move || {
            let database = Database::open(&state.db_path)?;
            database.queue_exams(request)
        }
    })
    .await?;

    spawn_exam_generation_thread(state, course_id)?;
    Ok(workspace)
}

#[tauri::command]
async fn get_exam_details(
    exam_id: String,
    state: State<'_, AppState>,
) -> Result<ExamDetails, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.get_exam_details(&exam_id)
    })
    .await
}

#[tauri::command]
async fn submit_exam_attempt(
    request: ExamSubmissionRequest,
    state: State<'_, AppState>,
) -> Result<ExamAttemptResult, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.submit_exam_attempt(request)
    })
    .await
}

#[tauri::command]
async fn apply_exam_review_actions(
    request: ApplyExamReviewActionsRequest,
    state: State<'_, AppState>,
) -> Result<ExamWorkspaceSnapshot, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.apply_exam_review_actions(request)
    })
    .await
}

#[tauri::command]
async fn get_formula_workspace(
    course_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Option<FormulaWorkspaceSnapshot>, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.get_formula_workspace(course_id)
    })
    .await
}

#[tauri::command]
async fn get_formula_details(
    formula_id: String,
    course_id: String,
    state: State<'_, AppState>,
) -> Result<FormulaDetails, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.get_formula_details(&formula_id, &course_id)
    })
    .await
}

#[tauri::command]
async fn generate_formula_brief(
    request: GenerateFormulaBriefRequest,
    state: State<'_, AppState>,
) -> Result<FormulaBrief, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.generate_formula_brief(request)
    })
    .await
}

#[tauri::command]
async fn list_chat_threads(
    scope: ChatScope,
    course_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<ChatThreadSummary>, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.list_chat_threads(scope, course_id)
    })
    .await
}

#[tauri::command]
async fn create_chat_thread(
    request: CreateChatThreadRequest,
    state: State<'_, AppState>,
) -> Result<ChatThreadDetails, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.create_chat_thread(request)
    })
    .await
}

#[tauri::command]
async fn get_chat_thread(
    thread_id: String,
    state: State<'_, AppState>,
) -> Result<ChatThreadDetails, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.get_chat_thread(&thread_id)
    })
    .await
}

#[tauri::command]
async fn send_chat_message(
    request: SendChatMessageRequest,
    state: State<'_, AppState>,
) -> Result<ChatThreadDetails, String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.send_chat_message(request)
    })
    .await
}

#[tauri::command]
async fn delete_chat_thread(thread_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let state = state.inner().clone();
    blocking(move || {
        let database = Database::open(&state.db_path)?;
        database.delete_chat_thread(&thread_id)
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

fn spawn_ai_enrichment_thread(
    state: AppState,
    course_id: String,
    force: bool,
) -> Result<(), String> {
    let should_spawn = {
        let mut active = state
            .active_ai_courses
            .lock()
            .map_err(|_| "failed to lock AI course registry".to_string())?;
        active.insert(course_id.clone())
    };

    if !should_spawn {
        return Ok(());
    }

    let state_for_thread = state.clone();
    std::thread::spawn(move || {
        let result = (|| -> Result<()> {
            let database = Database::open(&state_for_thread.db_path)?;
            database.run_ai_enrichment(&course_id, force)
        })();

        if let Err(error) = result {
            if let Ok(database) = Database::open(&state_for_thread.db_path) {
                let _ = database.mark_ai_enrichment_failed(&course_id, &error.to_string());
            }
            eprintln!("AI enrichment failed for {course_id}: {error}");
        }

        if let Ok(mut active) = state_for_thread.active_ai_courses.lock() {
            active.remove(&course_id);
        }
    });

    Ok(())
}

fn spawn_exam_generation_thread(state: AppState, course_id: String) -> Result<(), String> {
    let should_spawn = {
        let mut active = state
            .active_exam_courses
            .lock()
            .map_err(|_| "failed to lock exam course registry".to_string())?;
        active.insert(course_id.clone())
    };

    if !should_spawn {
        return Ok(());
    }

    let state_for_thread = state.clone();
    std::thread::spawn(move || {
        let result = (|| -> Result<()> {
            let database = Database::open(&state_for_thread.db_path)?;
            database.run_exam_generation_queue(&course_id)
        })();

        if let Err(error) = result {
            if let Ok(database) = Database::open(&state_for_thread.db_path) {
                let _ = database.mark_exam_generation_failed(&course_id, &error.to_string());
            }
            eprintln!("Exam generation failed for {course_id}: {error}");
        }

        if let Ok(mut active) = state_for_thread.active_exam_courses.lock() {
            active.remove(&course_id);
        }
    });

    Ok(())
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
        active_ai_courses: Arc::new(Mutex::new(HashSet::new())),
        active_exam_courses: Arc::new(Mutex::new(HashSet::new())),
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
            start_ai_enrichment,
            save_ai_settings,
            validate_ai_settings,
            generate_flashcards,
            generate_revision_note,
            get_exam_workspace,
            add_exam_source_notes,
            remove_exam_source_notes,
            clear_exam_source_queue,
            queue_exams,
            get_exam_details,
            submit_exam_attempt,
            apply_exam_review_actions,
            get_formula_workspace,
            get_formula_details,
            generate_formula_brief,
            list_chat_threads,
            create_chat_thread,
            get_chat_thread,
            send_chat_message,
            delete_chat_thread
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
