use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::ai::{self, AiProviderSettings, FlashcardCard};
use crate::markdown::{normalize_key, note_title_candidates, parse_markdown};
use crate::models::{
    AiCourseSummary, AiNoteInsight, AiSettings, AiSettingsInput, ConceptMetric, Countdown,
    CourseConfig, CourseConfigInput, CoverageStats, DashboardData, FlashcardGenerationRequest,
    FlashcardGenerationResult, FlashcardSummary, FormulaMetric, GraphStats, NoteDetails,
    NoteSummary, RevisionNoteRequest, RevisionNoteResult, RevisionSummary, ScanReport, ScanStatus,
    ValidationResult, VaultConfig, WeakNote, WorkspaceSnapshot,
};

pub struct Database {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct StoredCourse {
    pub id: String,
    pub name: String,
    pub folder: String,
    pub exam_date: Option<String>,
    pub revision_folder: String,
    pub flashcards_folder: String,
}

#[derive(Debug, Clone)]
struct StoredAiSettings {
    base_url: String,
    model: String,
    api_key: String,
    enabled: bool,
    timeout_ms: u64,
}

#[derive(Debug, Clone)]
struct StoredNote {
    id: String,
    title: String,
    relative_path: String,
    content_hash: String,
    links: Vec<String>,
    prerequisites: Vec<String>,
    frontmatter_exam_date: Option<String>,
}

#[derive(Debug, Clone)]
struct ParsedConcept {
    name: String,
    normalized_name: String,
    source: String,
    support_score: f64,
}

#[derive(Debug, Clone)]
struct ParsedStorageNote {
    title: String,
    excerpt: String,
    headings: Vec<String>,
    links: Vec<String>,
    tags: Vec<String>,
    prerequisites: Vec<String>,
    concepts: Vec<ParsedConcept>,
    formulas: Vec<String>,
    frontmatter_raw: Option<String>,
    frontmatter_exam_date: Option<String>,
}

#[derive(Debug, Clone)]
struct StoredEdge {
    from_note_id: String,
    to_note_id: String,
    edge_type: String,
    weight: f64,
    rationale: String,
}

#[derive(Debug, Clone)]
struct StoredWeakSuggestion {
    note_id: String,
    related_note_id: Option<String>,
    score: f64,
    reason: String,
}

#[derive(Debug, Clone)]
struct StoredFlashcardSet {
    source_note_ids: Vec<String>,
    csv_path: Option<String>,
    card_count: usize,
    created_at: String,
}

#[derive(Debug, Clone)]
struct StoredRevisionRun {
    note_path: String,
    item_count: usize,
    created_at: String,
}

#[derive(Debug, Clone)]
struct StoredAiCourseRun {
    status: String,
    started_at: Option<String>,
    finished_at: Option<String>,
    updated_at: Option<String>,
    model: Option<String>,
    summary: Option<String>,
    revision_priorities: Vec<String>,
    weak_spots: Vec<String>,
    next_actions: Vec<String>,
    last_error: Option<String>,
}

#[derive(Debug, Clone)]
struct StoredAiNoteState {
    note_id: String,
    status: String,
    content_hash: String,
    last_error: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct AiStatusCounts {
    total_notes: usize,
    ready_notes: usize,
    pending_notes: usize,
    failed_notes: usize,
    stale_notes: usize,
    missing_notes: usize,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("failed to create database directory")?;
        }

        let conn = Connection::open(path).context("failed to open SQLite database")?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
            .context("failed to configure SQLite")?;

        let database = Self { conn };
        database.migrate()?;
        Ok(database)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS vault_config (
              id INTEGER PRIMARY KEY CHECK (id = 1),
              vault_path TEXT NOT NULL,
              connected_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS ai_settings (
              id INTEGER PRIMARY KEY CHECK (id = 1),
              base_url TEXT NOT NULL,
              model TEXT NOT NULL,
              api_key TEXT NOT NULL,
              enabled INTEGER NOT NULL,
              timeout_ms INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS app_state (
              id INTEGER PRIMARY KEY CHECK (id = 1),
              selected_course_id TEXT,
              last_scan_at TEXT,
              last_note_count INTEGER NOT NULL DEFAULT 0,
              last_changed_count INTEGER NOT NULL DEFAULT 0,
              last_removed_count INTEGER NOT NULL DEFAULT 0
            );
            INSERT OR IGNORE INTO app_state (id, selected_course_id, last_scan_at, last_note_count, last_changed_count, last_removed_count)
            VALUES (1, NULL, NULL, 0, 0, 0);
            CREATE TABLE IF NOT EXISTS course_configs (
              id TEXT PRIMARY KEY,
              name TEXT NOT NULL,
              folder TEXT NOT NULL,
              exam_date TEXT,
              revision_folder TEXT NOT NULL,
              flashcards_folder TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS note_records (
              id TEXT PRIMARY KEY,
              course_id TEXT NOT NULL,
              relative_path TEXT NOT NULL,
              title TEXT NOT NULL,
              content_hash TEXT NOT NULL,
              frontmatter TEXT,
              frontmatter_exam_date TEXT,
              excerpt TEXT NOT NULL,
              headings_json TEXT NOT NULL,
              links_json TEXT NOT NULL,
              tags_json TEXT NOT NULL,
              prerequisites_json TEXT NOT NULL,
              concept_count INTEGER NOT NULL DEFAULT 0,
              formula_count INTEGER NOT NULL DEFAULT 0,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              UNIQUE(course_id, relative_path),
              FOREIGN KEY (course_id) REFERENCES course_configs(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS concept_records (
              id TEXT PRIMARY KEY,
              note_id TEXT NOT NULL,
              course_id TEXT NOT NULL,
              name TEXT NOT NULL,
              normalized_name TEXT NOT NULL,
              source TEXT NOT NULL,
              support_score REAL NOT NULL DEFAULT 1.0,
              FOREIGN KEY (note_id) REFERENCES note_records(id) ON DELETE CASCADE,
              FOREIGN KEY (course_id) REFERENCES course_configs(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS formula_records (
              id TEXT PRIMARY KEY,
              note_id TEXT NOT NULL,
              course_id TEXT NOT NULL,
              latex TEXT NOT NULL,
              normalized_latex TEXT NOT NULL,
              source TEXT NOT NULL,
              FOREIGN KEY (note_id) REFERENCES note_records(id) ON DELETE CASCADE,
              FOREIGN KEY (course_id) REFERENCES course_configs(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS dependency_edges (
              id TEXT PRIMARY KEY,
              course_id TEXT NOT NULL,
              from_note_id TEXT NOT NULL,
              to_note_id TEXT NOT NULL,
              edge_type TEXT NOT NULL,
              weight REAL NOT NULL,
              rationale TEXT NOT NULL,
              FOREIGN KEY (course_id) REFERENCES course_configs(id) ON DELETE CASCADE,
              FOREIGN KEY (from_note_id) REFERENCES note_records(id) ON DELETE CASCADE,
              FOREIGN KEY (to_note_id) REFERENCES note_records(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS weak_link_suggestions (
              id TEXT PRIMARY KEY,
              course_id TEXT NOT NULL,
              note_id TEXT NOT NULL,
              related_note_id TEXT,
              score REAL NOT NULL,
              reason TEXT NOT NULL,
              FOREIGN KEY (course_id) REFERENCES course_configs(id) ON DELETE CASCADE,
              FOREIGN KEY (note_id) REFERENCES note_records(id) ON DELETE CASCADE,
              FOREIGN KEY (related_note_id) REFERENCES note_records(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS flashcard_sets (
              id TEXT PRIMARY KEY,
              course_id TEXT NOT NULL,
              source_note_ids_json TEXT NOT NULL,
              markdown_path TEXT NOT NULL,
              csv_path TEXT,
              card_count INTEGER NOT NULL,
              created_at TEXT NOT NULL,
              FOREIGN KEY (course_id) REFERENCES course_configs(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS revision_note_runs (
              id TEXT PRIMARY KEY,
              course_id TEXT NOT NULL,
              note_path TEXT NOT NULL,
              item_count INTEGER NOT NULL,
              created_at TEXT NOT NULL,
              FOREIGN KEY (course_id) REFERENCES course_configs(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS ai_note_insights (
              note_id TEXT PRIMARY KEY,
              content_hash TEXT NOT NULL,
              summary TEXT NOT NULL,
              takeaways_json TEXT NOT NULL,
              exam_questions_json TEXT NOT NULL,
              connection_opportunities_json TEXT NOT NULL,
              generated_at TEXT NOT NULL,
              model TEXT NOT NULL,
              FOREIGN KEY (note_id) REFERENCES note_records(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS ai_note_states (
              note_id TEXT PRIMARY KEY,
              course_id TEXT NOT NULL,
              content_hash TEXT NOT NULL,
              status TEXT NOT NULL,
              last_error TEXT,
              updated_at TEXT NOT NULL,
              generated_at TEXT,
              model TEXT,
              FOREIGN KEY (note_id) REFERENCES note_records(id) ON DELETE CASCADE,
              FOREIGN KEY (course_id) REFERENCES course_configs(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS ai_course_runs (
              course_id TEXT PRIMARY KEY,
              status TEXT NOT NULL,
              started_at TEXT,
              finished_at TEXT,
              updated_at TEXT NOT NULL,
              model TEXT,
              summary TEXT,
              revision_priorities_json TEXT NOT NULL DEFAULT '[]',
              weak_spots_json TEXT NOT NULL DEFAULT '[]',
              next_actions_json TEXT NOT NULL DEFAULT '[]',
              last_error TEXT,
              FOREIGN KEY (course_id) REFERENCES course_configs(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_notes_course ON note_records(course_id);
            CREATE INDEX IF NOT EXISTS idx_concepts_course ON concept_records(course_id);
            CREATE INDEX IF NOT EXISTS idx_formulas_course ON formula_records(course_id);
            CREATE INDEX IF NOT EXISTS idx_edges_course ON dependency_edges(course_id);
            CREATE INDEX IF NOT EXISTS idx_weak_course ON weak_link_suggestions(course_id);
            CREATE INDEX IF NOT EXISTS idx_ai_note_states_course ON ai_note_states(course_id);
            "#,
        )?;
        Ok(())
    }

    pub fn load_workspace(&self) -> Result<WorkspaceSnapshot> {
        let vault = self.get_vault_config()?;
        let ai_settings = self.get_ai_settings_public()?;
        let courses = self.get_courses_with_metrics()?;
        let selected_course_id = self
            .get_selected_course_id()?
            .or_else(|| courses.first().map(|course| course.id.clone()));
        let dashboard = if vault.is_some() && selected_course_id.is_some() {
            self.get_dashboard(selected_course_id.clone())?
        } else {
            None
        };
        let scan_status = self.get_scan_status()?;

        Ok(WorkspaceSnapshot {
            vault,
            ai_settings,
            courses,
            selected_course_id,
            dashboard,
            scan_status,
        })
    }
}

impl Database {
    pub fn connect_vault(&self, vault_path: &str) -> Result<()> {
        let path = Path::new(vault_path);
        if !path.exists() {
            bail!("vault path does not exist");
        }
        if !path.is_dir() {
            bail!("vault path must be a directory");
        }

        self.conn.execute("DELETE FROM vault_config", [])?;
        self.conn.execute("DELETE FROM course_configs", [])?;
        self.conn.execute("DELETE FROM flashcard_sets", [])?;
        self.conn.execute("DELETE FROM revision_note_runs", [])?;
        self.reset_app_state()?;

        self.conn.execute(
            "INSERT INTO vault_config (id, vault_path, connected_at) VALUES (1, ?1, ?2)",
            params![path.to_string_lossy().to_string(), now_string()],
        )?;
        self.seed_courses_from_vault(path)?;

        Ok(())
    }

    pub fn disconnect_vault(&self) -> Result<()> {
        self.conn.execute("DELETE FROM vault_config", [])?;
        self.conn.execute("DELETE FROM course_configs", [])?;
        self.conn.execute("DELETE FROM flashcard_sets", [])?;
        self.conn.execute("DELETE FROM revision_note_runs", [])?;
        self.reset_app_state()?;
        Ok(())
    }

    pub fn save_ai_settings(&self, input: AiSettingsInput) -> Result<()> {
        let existing = self.get_ai_settings_internal()?;
        let timeout_ms = input.timeout_ms.unwrap_or(12_000).max(2_000);
        let api_key = input
            .api_key
            .filter(|value| !value.trim().is_empty())
            .or_else(|| existing.clone().map(|settings| settings.api_key))
            .unwrap_or_default();

        self.conn.execute(
            r#"
            INSERT INTO ai_settings (id, base_url, model, api_key, enabled, timeout_ms)
            VALUES (1, ?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
              base_url = excluded.base_url,
              model = excluded.model,
              api_key = excluded.api_key,
              enabled = excluded.enabled,
              timeout_ms = excluded.timeout_ms
            "#,
            params![
                input.base_url.trim(),
                input.model.trim(),
                api_key,
                bool_to_int(input.enabled),
                timeout_ms as i64,
            ],
        )?;

        Ok(())
    }

    pub fn validate_ai_settings(&self, input: AiSettingsInput) -> Result<ValidationResult> {
        let existing = self.get_ai_settings_internal()?;
        let settings = AiProviderSettings {
            base_url: input.base_url.trim().to_string(),
            model: input.model.trim().to_string(),
            api_key: input
                .api_key
                .filter(|value| !value.trim().is_empty())
                .or_else(|| existing.and_then(|value| resolve_api_key(&value.api_key)))
                .unwrap_or_default(),
            enabled: input.enabled,
            timeout_ms: input.timeout_ms.unwrap_or(12_000),
        };

        match ai::validate_settings(&settings) {
            Ok(message) => Ok(ValidationResult { ok: true, message }),
            Err(error) => Ok(ValidationResult {
                ok: false,
                message: error.to_string(),
            }),
        }
    }

    pub fn save_course_config(&self, input: CourseConfigInput) -> Result<String> {
        let vault = self
            .get_vault_config()?
            .ok_or_else(|| anyhow!("connect a vault before creating courses"))?;
        if input.name.trim().is_empty() {
            bail!("course name is required");
        }
        if input.folder.trim().is_empty() {
            bail!("course folder is required");
        }

        let normalized_folder = normalize_relative_path(input.folder.trim());
        let absolute_folder = Path::new(&vault.vault_path).join(&normalized_folder);
        if !absolute_folder.exists() {
            bail!("course folder does not exist inside the selected vault");
        }

        let id = input
            .id
            .or_else(|| {
                self.find_course_id_by_folder(&normalized_folder)
                    .ok()
                    .flatten()
            })
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let now = now_string();
        self.conn.execute(
            r#"
            INSERT INTO course_configs (id, name, folder, exam_date, revision_folder, flashcards_folder, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
            ON CONFLICT(id) DO UPDATE SET
              name = excluded.name,
              folder = excluded.folder,
              exam_date = excluded.exam_date,
              revision_folder = excluded.revision_folder,
              flashcards_folder = excluded.flashcards_folder,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                input.name.trim(),
                normalized_folder,
                nullable_string(input.exam_date.as_deref()),
                input
                    .revision_folder
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or("Revision"),
                input
                    .flashcards_folder
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or("Flashcards"),
                now,
            ],
        )?;

        self.set_selected_course(Some(&id))?;
        Ok(id)
    }

    pub fn delete_course(&self, course_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM course_configs WHERE id = ?1",
            params![course_id],
        )?;

        let replacement = self
            .list_courses()?
            .into_iter()
            .find(|course| course.id != course_id)
            .map(|course| course.id);
        self.set_selected_course(replacement.as_deref())?;
        Ok(())
    }

    fn seed_courses_from_vault(&self, vault_path: &Path) -> Result<()> {
        let mut course_ids = Vec::new();
        let mut folders = fs::read_dir(vault_path)
            .with_context(|| format!("failed to read {}", vault_path.display()))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || name.eq_ignore_ascii_case("attachments") {
                    return None;
                }
                if !directory_contains_markdown(&entry.path()) {
                    return None;
                }
                Some((name, entry.path()))
            })
            .collect::<Vec<_>>();
        folders.sort_by(|left, right| left.0.cmp(&right.0));

        for (folder_name, _) in folders {
            let id = build_seed_course_id(&folder_name);
            let now = now_string();
            self.conn.execute(
                r#"
                INSERT INTO course_configs (id, name, folder, exam_date, revision_folder, flashcards_folder, created_at, updated_at)
                VALUES (?1, ?2, ?3, NULL, 'Revision', 'Flashcards', ?4, ?4)
                ON CONFLICT(id) DO UPDATE SET
                  name = excluded.name,
                  folder = excluded.folder,
                  updated_at = excluded.updated_at
                "#,
                params![id, folder_name, normalize_relative_path(&folder_name), now],
            )?;
            course_ids.push(build_seed_course_id(&folder_name));
        }

        self.set_selected_course(course_ids.first().map(String::as_str))?;
        Ok(())
    }

    pub fn get_dashboard(&self, course_id: Option<String>) -> Result<Option<DashboardData>> {
        let vault = self
            .get_vault_config()?
            .ok_or_else(|| anyhow!("no vault connected"))?;
        let courses = self.list_courses()?;
        if courses.is_empty() {
            return Ok(None);
        }

        let selected = course_id
            .or_else(|| self.get_selected_course_id().ok().flatten())
            .and_then(|id| {
                courses
                    .iter()
                    .find(|course| course.id == id)
                    .map(|course| course.id.clone())
            })
            .unwrap_or_else(|| courses[0].id.clone());

        self.set_selected_course(Some(&selected))?;
        let course = courses
            .into_iter()
            .find(|course| course.id == selected)
            .ok_or_else(|| anyhow!("selected course does not exist"))?;

        let notes = self.list_notes(&course.id)?;
        let concepts = self.list_concepts(&course.id)?;
        let formulas = self.list_formulas(&course.id)?;
        let edges = self.list_edges(&course.id)?;
        let weak_rows = self.list_weak_suggestions(&course.id)?;
        let flashcard_sets = self.list_flashcard_sets(&course.id)?;
        let revision_runs = self.list_revision_runs(&course.id)?;
        let ai_settings = self.get_ai_settings_public()?;
        let ai_states = self.list_ai_note_states(&course.id)?;
        let ai_run = self.get_ai_course_run(&course.id)?;

        Ok(Some(DashboardData {
            generated_at: now_string(),
            vault_path: vault.vault_path,
            selected_course_id: Some(course.id.clone()),
            countdown: build_countdown(&course, &notes),
            coverage: build_coverage(&notes, &concepts, &formulas, &flashcard_sets),
            graph: build_graph_stats(&notes, &edges),
            weak_notes: build_weak_notes(&notes, &weak_rows),
            top_concepts: build_top_concepts(&concepts),
            formulas: build_top_formulas(&formulas),
            flashcards: build_flashcard_summary(&flashcard_sets),
            revision: build_revision_summary(&revision_runs),
            notes: build_note_summaries(&notes, &edges, &concepts, &formulas, &ai_states),
            ai: build_ai_course_summary(
                notes.len(),
                ai_settings.as_ref(),
                &notes,
                &ai_states,
                ai_run.as_ref(),
            ),
        }))
    }

    pub fn get_note_details(&self, note_id: &str) -> Result<NoteDetails> {
        let note = self
            .conn
            .query_row(
                r#"
                SELECT id, title, relative_path, excerpt, headings_json, links_json, tags_json, prerequisites_json, content_hash
                FROM note_records
                WHERE id = ?1
                "#,
                params![note_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        from_json_vec::<String>(&row.get::<_, String>(4)?),
                        from_json_vec::<String>(&row.get::<_, String>(5)?),
                        from_json_vec::<String>(&row.get::<_, String>(6)?),
                        from_json_vec::<String>(&row.get::<_, String>(7)?),
                        row.get::<_, String>(8)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| anyhow!("note not found"))?;

        let mut concepts = self
            .conn
            .prepare("SELECT name FROM concept_records WHERE note_id = ?1 ORDER BY support_score DESC, name ASC")?
            .query_map(params![note_id], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        concepts.dedup();

        let mut formulas = self
            .conn
            .prepare("SELECT latex FROM formula_records WHERE note_id = ?1 ORDER BY latex ASC")?
            .query_map(params![note_id], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        formulas.dedup();

        let suggestions = self
            .conn
            .prepare(
                r#"
                SELECT COALESCE(related.title, weak.reason)
                FROM weak_link_suggestions AS weak
                LEFT JOIN note_records AS related ON related.id = weak.related_note_id
                WHERE weak.note_id = ?1
                ORDER BY weak.score DESC
                "#,
            )?
            .query_map(params![note_id], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut links = note.5;
        for prerequisite in note.7 {
            if !links.iter().any(|value| value == &prerequisite) {
                links.push(prerequisite);
            }
        }

        let ai_state = self.resolve_note_ai_state(note_id, &note.8)?;
        let ai_insight = self.get_cached_note_ai_insight(note_id, &note.8)?;

        Ok(NoteDetails {
            id: note.0,
            title: note.1,
            relative_path: note.2,
            excerpt: note.3,
            headings: note.4,
            links,
            tags: note.6,
            concepts,
            formulas,
            suggestions,
            ai_status: ai_state.0,
            ai_error: ai_state.1,
            ai_insight,
        })
    }

    pub fn generate_note_ai_insight(&self, note_id: &str) -> Result<AiNoteInsight> {
        let settings = self.ai_settings_for_runtime()?.ok_or_else(|| {
            anyhow!("enable AI in Setup and save reachable provider settings first")
        })?;
        let course_id = self.note_course_id(note_id)?;

        let note = self
            .conn
            .query_row(
                r#"
                SELECT id, title, excerpt, headings_json, links_json, content_hash
                FROM note_records
                WHERE id = ?1
                "#,
                params![note_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        from_json_vec::<String>(&row.get::<_, String>(3)?),
                        from_json_vec::<String>(&row.get::<_, String>(4)?),
                        row.get::<_, String>(5)?,
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| anyhow!("note not found"))?;

        let concepts = self
            .conn
            .prepare(
                "SELECT name FROM concept_records WHERE note_id = ?1 ORDER BY support_score DESC, name ASC",
            )?
            .query_map(params![note_id], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let formulas = self
            .conn
            .prepare("SELECT latex FROM formula_records WHERE note_id = ?1 ORDER BY latex ASC")?
            .query_map(params![note_id], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        self.upsert_ai_note_state(
            note_id,
            &course_id,
            &note.5,
            "running",
            None,
            None,
            Some(&settings.model),
        )?;

        let payload = ai::generate_note_insight(
            &settings, &note.1, &note.2, &note.3, &concepts, &formulas, &note.4,
        )
        .map_err(|error| {
            let _ = self.upsert_ai_note_state(
                note_id,
                &course_id,
                &note.5,
                "failed",
                Some(&truncate_error(&error.to_string())),
                None,
                Some(&settings.model),
            );
            error
        })?;

        let generated_at = now_string();
        self.upsert_ai_note_state(
            note_id,
            &course_id,
            &note.5,
            "complete",
            None,
            Some(&generated_at),
            Some(&settings.model),
        )?;
        self.conn.execute(
            r#"
            INSERT INTO ai_note_insights (
              note_id, content_hash, summary, takeaways_json, exam_questions_json,
              connection_opportunities_json, generated_at, model
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(note_id) DO UPDATE SET
              content_hash = excluded.content_hash,
              summary = excluded.summary,
              takeaways_json = excluded.takeaways_json,
              exam_questions_json = excluded.exam_questions_json,
              connection_opportunities_json = excluded.connection_opportunities_json,
              generated_at = excluded.generated_at,
              model = excluded.model
            "#,
            params![
                note.0,
                note.5,
                payload.summary,
                to_json(&payload.takeaways),
                to_json(&payload.exam_questions),
                to_json(&payload.connection_opportunities),
                generated_at,
                settings.model,
            ],
        )?;

        self.get_cached_note_ai_insight(note_id, &note.5)?
            .ok_or_else(|| anyhow!("AI insight was generated but could not be loaded"))
    }

    pub fn queue_ai_enrichment(&self, course_id: &str, force: bool) -> Result<AiCourseSummary> {
        self.ai_settings_for_runtime()?.ok_or_else(|| {
            anyhow!("enable AI in Setup and save reachable provider settings first")
        })?;
        let course = self
            .find_course(course_id)?
            .ok_or_else(|| anyhow!("course not found"))?;
        let notes_for_summary = self.list_notes(&course.id)?;
        let ai_settings = self.get_ai_settings_public()?;
        let existing_run = self.get_ai_course_run(course_id)?;
        let existing_states = self.list_ai_note_states(course_id)?;

        if existing_run.as_ref().map(|run| run.status.as_str()) == Some("running") {
            return Ok(build_ai_course_summary(
                notes_for_summary.len(),
                ai_settings.as_ref(),
                &notes_for_summary,
                &existing_states,
                existing_run.as_ref(),
            ));
        }

        let notes = self.list_ai_candidate_notes(course_id, force)?;
        let started_at = now_string();

        for note in &notes {
            self.upsert_ai_note_state(&note.0, course_id, &note.1, "queued", None, None, None)?;
        }

        self.conn.execute(
            r#"
            INSERT INTO ai_course_runs (
              course_id, status, started_at, finished_at, updated_at, model, summary,
              revision_priorities_json, weak_spots_json, next_actions_json, last_error
            )
            VALUES (?1, 'running', ?2, NULL, ?2, NULL, NULL, '[]', '[]', '[]', NULL)
            ON CONFLICT(course_id) DO UPDATE SET
              status = 'running',
              started_at = excluded.started_at,
              finished_at = NULL,
              updated_at = excluded.updated_at,
              model = NULL,
              summary = NULL,
              revision_priorities_json = '[]',
              weak_spots_json = '[]',
              next_actions_json = '[]',
              last_error = NULL
            "#,
            params![course_id, started_at],
        )?;

        let ai_states = self.list_ai_note_states(course_id)?;
        let ai_run = self.get_ai_course_run(course_id)?;
        Ok(build_ai_course_summary(
            notes_for_summary.len(),
            ai_settings.as_ref(),
            &notes_for_summary,
            &ai_states,
            ai_run.as_ref(),
        ))
    }

    pub fn run_ai_enrichment(&self, course_id: &str, force: bool) -> Result<()> {
        let settings = self.ai_settings_for_runtime()?.ok_or_else(|| {
            anyhow!("enable AI in Setup and save reachable provider settings first")
        })?;
        let course = self
            .find_course(course_id)?
            .ok_or_else(|| anyhow!("course not found"))?;
        let now = now_string();
        let notes = self.list_ai_candidate_notes(course_id, force)?;
        let mut failed_note_count = 0usize;

        for (note_id, content_hash, title, excerpt, headings, links, concepts, formulas) in &notes {
            self.upsert_ai_note_state(
                note_id,
                course_id,
                content_hash,
                "running",
                None,
                None,
                Some(&settings.model),
            )?;

            let result = ai::generate_note_insight(
                &settings, title, excerpt, headings, concepts, formulas, links,
            );

            match result {
                Ok(payload) => {
                    let generated_at = now_string();
                    self.conn.execute(
                        r#"
                        INSERT INTO ai_note_insights (
                          note_id, content_hash, summary, takeaways_json, exam_questions_json,
                          connection_opportunities_json, generated_at, model
                        )
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                        ON CONFLICT(note_id) DO UPDATE SET
                          content_hash = excluded.content_hash,
                          summary = excluded.summary,
                          takeaways_json = excluded.takeaways_json,
                          exam_questions_json = excluded.exam_questions_json,
                          connection_opportunities_json = excluded.connection_opportunities_json,
                          generated_at = excluded.generated_at,
                          model = excluded.model
                        "#,
                        params![
                            note_id,
                            content_hash,
                            payload.summary,
                            to_json(&payload.takeaways),
                            to_json(&payload.exam_questions),
                            to_json(&payload.connection_opportunities),
                            generated_at,
                            settings.model,
                        ],
                    )?;
                    self.upsert_ai_note_state(
                        note_id,
                        course_id,
                        content_hash,
                        "complete",
                        None,
                        Some(&generated_at),
                        Some(&settings.model),
                    )?;
                }
                Err(error) => {
                    failed_note_count += 1;
                    self.upsert_ai_note_state(
                        note_id,
                        course_id,
                        content_hash,
                        "failed",
                        Some(&truncate_error(&error.to_string())),
                        None,
                        Some(&settings.model),
                    )?;
                }
            }
        }

        let course_brief = self.generate_course_brief(&course, &settings);
        let finished_at = now_string();

        match course_brief {
            Ok(brief) => {
                let status = if failed_note_count > 0 {
                    "failed"
                } else {
                    "complete"
                };
                let last_error = if failed_note_count > 0 {
                    Some(format!(
                        "{} note brief{} failed. Open Notes to retry the failed items.",
                        failed_note_count,
                        if failed_note_count == 1 { "" } else { "s" }
                    ))
                } else {
                    None
                };
                self.conn.execute(
                    r#"
                    INSERT INTO ai_course_runs (
                      course_id, status, started_at, finished_at, updated_at, model, summary,
                      revision_priorities_json, weak_spots_json, next_actions_json, last_error
                    )
                    VALUES (?1, ?2, ?3, ?4, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                    ON CONFLICT(course_id) DO UPDATE SET
                      status = excluded.status,
                      finished_at = excluded.finished_at,
                      updated_at = excluded.updated_at,
                      model = excluded.model,
                      summary = excluded.summary,
                      revision_priorities_json = excluded.revision_priorities_json,
                      weak_spots_json = excluded.weak_spots_json,
                      next_actions_json = excluded.next_actions_json,
                      last_error = excluded.last_error
                    "#,
                    params![
                        course_id,
                        status,
                        now,
                        finished_at,
                        settings.model,
                        brief.summary,
                        to_json(&brief.revision_priorities),
                        to_json(&brief.weak_spots),
                        to_json(&brief.next_actions),
                        last_error,
                    ],
                )?;
            }
            Err(error) => {
                let message = truncate_error(&error.to_string());
                self.conn.execute(
                    r#"
                    INSERT INTO ai_course_runs (
                      course_id, status, started_at, finished_at, updated_at, model, summary,
                      revision_priorities_json, weak_spots_json, next_actions_json, last_error
                    )
                    VALUES (?1, 'failed', ?2, ?3, ?3, ?4, NULL, '[]', '[]', '[]', ?5)
                    ON CONFLICT(course_id) DO UPDATE SET
                      status = 'failed',
                      finished_at = excluded.finished_at,
                      updated_at = excluded.updated_at,
                      model = excluded.model,
                      summary = NULL,
                      revision_priorities_json = '[]',
                      weak_spots_json = '[]',
                      next_actions_json = '[]',
                      last_error = excluded.last_error
                    "#,
                    params![course_id, now, finished_at, settings.model, message],
                )?;
            }
        }

        Ok(())
    }

    pub fn mark_ai_enrichment_failed(&self, course_id: &str, message: &str) -> Result<()> {
        let timestamp = now_string();
        self.conn.execute(
            r#"
            INSERT INTO ai_course_runs (
              course_id, status, started_at, finished_at, updated_at, model, summary,
              revision_priorities_json, weak_spots_json, next_actions_json, last_error
            )
            VALUES (?1, 'failed', ?2, ?2, ?2, NULL, NULL, '[]', '[]', '[]', ?3)
            ON CONFLICT(course_id) DO UPDATE SET
              status = 'failed',
              finished_at = excluded.finished_at,
              updated_at = excluded.updated_at,
              summary = NULL,
              revision_priorities_json = '[]',
              weak_spots_json = '[]',
              next_actions_json = '[]',
              last_error = excluded.last_error
            "#,
            params![course_id, timestamp, truncate_error(message)],
        )?;
        Ok(())
    }
}

impl Database {
    pub fn run_scan(&self) -> Result<ScanReport> {
        let vault = self
            .get_vault_config()?
            .ok_or_else(|| anyhow!("no vault connected"))?;
        let courses = self.list_courses()?;
        if courses.is_empty() {
            bail!("create at least one course before scanning");
        }

        let selected_course_id = self
            .get_selected_course_id()?
            .or_else(|| courses.first().map(|course| course.id.clone()));
        let courses = if let Some(selected_course_id) = selected_course_id {
            courses
                .into_iter()
                .filter(|course| course.id == selected_course_id)
                .collect::<Vec<_>>()
        } else {
            courses
        };

        if courses.is_empty() {
            bail!("select a course before scanning");
        }

        let mut scanned_notes = 0usize;
        let mut changed_notes = 0usize;
        let mut unchanged_notes = 0usize;
        let mut removed_notes = 0usize;
        let mut generated_edges = 0usize;
        let mut generated_weak_links = 0usize;

        for course in &courses {
            let folder = Path::new(&vault.vault_path).join(&course.folder);
            if !folder.exists() {
                continue;
            }

            let existing_hashes = self.existing_note_hashes(&course.id)?;
            let mut seen_ids = HashSet::new();
            let mut files = WalkDir::new(&folder)
                .into_iter()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().is_file())
                .filter(|entry| {
                    entry
                        .path()
                        .extension()
                        .and_then(|value| value.to_str())
                        .map(|value| value.eq_ignore_ascii_case("md"))
                        .unwrap_or(false)
                })
                .map(|entry| entry.path().to_path_buf())
                .collect::<Vec<_>>();
            files.sort();

            for path in files {
                scanned_notes += 1;
                let relative_path = normalize_relative_path(
                    path.strip_prefix(&vault.vault_path)
                        .context("failed to compute vault-relative path")?
                        .to_string_lossy()
                        .as_ref(),
                );
                let note_id = build_note_id(&course.id, &relative_path);
                seen_ids.insert(note_id.clone());

                let content = fs::read_to_string(&path)
                    .with_context(|| format!("failed to read {}", path.display()))?;
                let content_hash = hash_content(&content);

                if existing_hashes
                    .get(&note_id)
                    .map(|existing| existing == &content_hash)
                    .unwrap_or(false)
                {
                    unchanged_notes += 1;
                    continue;
                }

                let file_stem = path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("Untitled");
                let parsed = self.prepare_note_for_storage(file_stem, &content)?;
                self.upsert_note(&course.id, &note_id, &relative_path, &content_hash, parsed)?;
                changed_notes += 1;
            }

            for stale_note_id in existing_hashes.keys() {
                if !seen_ids.contains(stale_note_id) {
                    self.delete_note(stale_note_id)?;
                    removed_notes += 1;
                }
            }

            let (edge_count, weak_count) = self.rebuild_course_analytics(course)?;
            generated_edges += edge_count;
            generated_weak_links += weak_count;
        }

        let scanned_at = now_string();
        let note_count = self.total_note_count()?;
        self.conn.execute(
            "UPDATE app_state SET last_scan_at = ?1, last_note_count = ?2, last_changed_count = ?3, last_removed_count = ?4 WHERE id = 1",
            params![scanned_at, note_count as i64, changed_notes as i64, removed_notes as i64],
        )?;

        Ok(ScanReport {
            scanned_notes,
            changed_notes,
            unchanged_notes,
            removed_notes,
            generated_edges,
            generated_weak_links,
            scanned_at,
        })
    }

    pub fn generate_flashcards(
        &self,
        request: FlashcardGenerationRequest,
        export_dir: &Path,
    ) -> Result<FlashcardGenerationResult> {
        let vault = self
            .get_vault_config()?
            .ok_or_else(|| anyhow!("no vault connected"))?;
        let course = self
            .find_course(&request.course_id)?
            .ok_or_else(|| anyhow!("course not found"))?;
        if request.note_ids.is_empty() {
            bail!("select at least one note to generate flashcards");
        }

        let note_ids = request.note_ids;
        let notes = note_ids
            .iter()
            .map(|note_id| self.get_note_details(note_id))
            .collect::<Result<Vec<_>>>()?;

        let ai_settings = self.ai_settings_for_runtime()?;
        let mut cards = if let Some(settings) = ai_settings.as_ref() {
            let payload = notes
                .iter()
                .map(|note| {
                    format!(
                        "Title: {}\nExcerpt: {}\nConcepts: {}\nFormulas: {}\n",
                        note.title,
                        note.excerpt,
                        note.concepts.join(", "),
                        note.formulas.join(" | ")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n---\n");
            ai::generate_flashcards(settings, &course.name, &payload).unwrap_or_default()
        } else {
            Vec::new()
        };

        cards.extend(build_local_flashcards(&notes));
        dedupe_cards(&mut cards);
        cards.truncate(16);
        if cards.is_empty() {
            bail!("no flashcards could be generated from the selected notes");
        }

        let generated_at = now_string();
        let stamp = file_stamp();
        let flashcards_folder = request
            .flashcards_folder
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| course.flashcards_folder.clone());
        let markdown_dir =
            Path::new(&vault.vault_path).join(normalize_relative_path(&flashcards_folder));
        fs::create_dir_all(&markdown_dir).context("failed to create flashcard folder")?;
        let markdown_path =
            markdown_dir.join(format!("{}-flashcards-{}.md", slugify(&course.name), stamp));
        fs::write(
            &markdown_path,
            render_flashcard_markdown(&course.name, &generated_at, &cards),
        )
        .context("failed to write flashcard markdown file")?;

        let csv_path = if request.export_csv.unwrap_or(true) {
            fs::create_dir_all(export_dir).context("failed to create export directory")?;
            let csv_path = export_dir.join(format!(
                "{}-flashcards-{}.csv",
                slugify(&course.name),
                stamp
            ));
            let mut writer =
                csv::Writer::from_path(&csv_path).context("failed to create flashcard CSV")?;
            writer
                .write_record(["Front", "Back"])
                .context("failed to write CSV header")?;
            for card in &cards {
                writer
                    .write_record([card.question.as_str(), card.answer.as_str()])
                    .context("failed to write CSV row")?;
            }
            writer.flush().context("failed to flush CSV export")?;
            Some(csv_path)
        } else {
            None
        };

        self.conn.execute(
            r#"
            INSERT INTO flashcard_sets (id, course_id, source_note_ids_json, markdown_path, csv_path, card_count, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                Uuid::new_v4().to_string(),
                course.id,
                to_json(&note_ids),
                markdown_path.to_string_lossy().to_string(),
                csv_path.as_ref().map(|path| path.to_string_lossy().to_string()),
                cards.len() as i64,
                generated_at,
            ],
        )?;

        Ok(FlashcardGenerationResult {
            markdown_path: markdown_path.to_string_lossy().to_string(),
            csv_path: csv_path.map(|path| path.to_string_lossy().to_string()),
            card_count: cards.len(),
            generated_at,
        })
    }

    pub fn generate_revision_note(
        &self,
        request: RevisionNoteRequest,
    ) -> Result<RevisionNoteResult> {
        let vault = self
            .get_vault_config()?
            .ok_or_else(|| anyhow!("no vault connected"))?;
        let course = self
            .find_course(&request.course_id)?
            .ok_or_else(|| anyhow!("course not found"))?;
        let dashboard = self
            .get_dashboard(Some(course.id.clone()))?
            .ok_or_else(|| anyhow!("no dashboard data available"))?;

        let generated_at = now_string();
        let stamp = file_stamp();
        let revision_folder = request
            .revision_folder
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| course.revision_folder.clone());
        let revision_dir =
            Path::new(&vault.vault_path).join(normalize_relative_path(&revision_folder));
        fs::create_dir_all(&revision_dir).context("failed to create revision folder")?;
        let note_path =
            revision_dir.join(format!("{}-revision-{}.md", slugify(&course.name), stamp));
        fs::write(
            &note_path,
            render_revision_markdown(&course.name, &dashboard),
        )
        .context("failed to write revision note")?;

        let item_count =
            dashboard.notes.iter().take(8).count() + dashboard.weak_notes.iter().take(5).count();
        self.conn.execute(
            r#"
            INSERT INTO revision_note_runs (id, course_id, note_path, item_count, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                Uuid::new_v4().to_string(),
                course.id,
                note_path.to_string_lossy().to_string(),
                item_count as i64,
                generated_at,
            ],
        )?;

        Ok(RevisionNoteResult {
            note_path: note_path.to_string_lossy().to_string(),
            generated_at,
            item_count,
        })
    }
}

impl Database {
    fn get_vault_config(&self) -> Result<Option<VaultConfig>> {
        self.conn
            .query_row(
                "SELECT vault_path, connected_at FROM vault_config WHERE id = 1",
                [],
                |row| {
                    Ok(VaultConfig {
                        vault_path: row.get(0)?,
                        connected_at: row.get(1)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    fn get_ai_settings_internal(&self) -> Result<Option<StoredAiSettings>> {
        self.conn
            .query_row(
                "SELECT base_url, model, api_key, enabled, timeout_ms FROM ai_settings WHERE id = 1",
                [],
                |row| {
                    Ok(StoredAiSettings {
                        base_url: row.get(0)?,
                        model: row.get(1)?,
                        api_key: row.get(2)?,
                        enabled: row.get::<_, i64>(3)? == 1,
                        timeout_ms: row.get::<_, i64>(4)? as u64,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    fn get_ai_settings_public(&self) -> Result<Option<AiSettings>> {
        Ok(self.get_ai_settings_internal()?.map(|settings| AiSettings {
            base_url: settings.base_url,
            model: settings.model,
            enabled: settings.enabled,
            timeout_ms: settings.timeout_ms,
            has_api_key: resolve_api_key(&settings.api_key).is_some(),
        }))
    }

    fn ai_settings_for_runtime(&self) -> Result<Option<AiProviderSettings>> {
        Ok(self.get_ai_settings_internal()?.and_then(|settings| {
            if settings.enabled {
                Some(AiProviderSettings {
                    base_url: settings.base_url,
                    model: settings.model,
                    api_key: resolve_api_key(&settings.api_key).unwrap_or_default(),
                    enabled: settings.enabled,
                    timeout_ms: settings.timeout_ms,
                })
            } else {
                None
            }
        }))
    }

    fn get_cached_note_ai_insight(
        &self,
        note_id: &str,
        content_hash: &str,
    ) -> Result<Option<AiNoteInsight>> {
        self.conn
            .query_row(
                r#"
                SELECT summary, takeaways_json, exam_questions_json, connection_opportunities_json, generated_at, model
                FROM ai_note_insights
                WHERE note_id = ?1 AND content_hash = ?2
                "#,
                params![note_id, content_hash],
                |row| {
                    Ok(AiNoteInsight {
                        note_id: note_id.to_string(),
                        summary: row.get(0)?,
                        takeaways: from_json_vec::<String>(&row.get::<_, String>(1)?),
                        exam_questions: from_json_vec::<String>(&row.get::<_, String>(2)?),
                        connection_opportunities: from_json_vec::<String>(&row.get::<_, String>(3)?),
                        generated_at: row.get(4)?,
                        model: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    fn resolve_note_ai_state(
        &self,
        note_id: &str,
        content_hash: &str,
    ) -> Result<(String, Option<String>)> {
        let state = self
            .conn
            .query_row(
                r#"
                SELECT status, content_hash, last_error
                FROM ai_note_states
                WHERE note_id = ?1
                "#,
                params![note_id],
                |row| {
                    Ok(StoredAiNoteState {
                        note_id: note_id.to_string(),
                        status: row.get(0)?,
                        content_hash: row.get(1)?,
                        last_error: row.get(2)?,
                    })
                },
            )
            .optional()?;

        Ok(match state {
            None => ("missing".to_string(), None),
            Some(state) if state.content_hash != content_hash => ("stale".to_string(), None),
            Some(state) => (state.status, state.last_error),
        })
    }

    fn list_ai_note_states(&self, course_id: &str) -> Result<HashMap<String, StoredAiNoteState>> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT note_id, status, content_hash, last_error, updated_at
            FROM ai_note_states
            WHERE course_id = ?1
            "#,
        )?;
        let rows = statement
            .query_map(params![course_id], |row| {
                Ok(StoredAiNoteState {
                    note_id: row.get(0)?,
                    status: row.get(1)?,
                    content_hash: row.get(2)?,
                    last_error: row.get(3)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows
            .into_iter()
            .map(|state| (state.note_id.clone(), state))
            .collect())
    }

    fn get_ai_course_run(&self, course_id: &str) -> Result<Option<StoredAiCourseRun>> {
        self.conn
            .query_row(
                r#"
                SELECT status, started_at, finished_at, updated_at, model, summary,
                       revision_priorities_json, weak_spots_json, next_actions_json, last_error
                FROM ai_course_runs
                WHERE course_id = ?1
                "#,
                params![course_id],
                |row| {
                    Ok(StoredAiCourseRun {
                        status: row.get(0)?,
                        started_at: row.get(1)?,
                        finished_at: row.get(2)?,
                        updated_at: row.get(3)?,
                        model: row.get(4)?,
                        summary: row.get(5)?,
                        revision_priorities: from_json_vec::<String>(&row.get::<_, String>(6)?),
                        weak_spots: from_json_vec::<String>(&row.get::<_, String>(7)?),
                        next_actions: from_json_vec::<String>(&row.get::<_, String>(8)?),
                        last_error: row.get(9)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    fn note_course_id(&self, note_id: &str) -> Result<String> {
        self.conn
            .query_row(
                "SELECT course_id FROM note_records WHERE id = ?1",
                params![note_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or_else(|| anyhow!("note not found"))
    }

    fn upsert_ai_note_state(
        &self,
        note_id: &str,
        course_id: &str,
        content_hash: &str,
        status: &str,
        last_error: Option<&str>,
        generated_at: Option<&str>,
        model: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO ai_note_states (
              note_id, course_id, content_hash, status, last_error, updated_at, generated_at, model
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(note_id) DO UPDATE SET
              course_id = excluded.course_id,
              content_hash = excluded.content_hash,
              status = excluded.status,
              last_error = excluded.last_error,
              updated_at = excluded.updated_at,
              generated_at = excluded.generated_at,
              model = excluded.model
            "#,
            params![
                note_id,
                course_id,
                content_hash,
                status,
                last_error,
                now_string(),
                generated_at,
                model,
            ],
        )?;
        Ok(())
    }

    fn list_ai_candidate_notes(
        &self,
        course_id: &str,
        force: bool,
    ) -> Result<
        Vec<(
            String,
            String,
            String,
            String,
            Vec<String>,
            Vec<String>,
            Vec<String>,
            Vec<String>,
        )>,
    > {
        let ai_states = self.list_ai_note_states(course_id)?;
        let mut statement = self.conn.prepare(
            r#"
            SELECT id, content_hash, title, excerpt, headings_json, links_json
            FROM note_records
            WHERE course_id = ?1
            ORDER BY title ASC
            "#,
        )?;

        let notes = statement
            .query_map(params![course_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    from_json_vec::<String>(&row.get::<_, String>(4)?),
                    from_json_vec::<String>(&row.get::<_, String>(5)?),
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut candidates = Vec::new();
        for (note_id, content_hash, title, excerpt, headings, links) in notes {
            let should_process = force
                || match ai_states.get(&note_id) {
                    None => true,
                    Some(state) if state.content_hash != content_hash => true,
                    Some(state) if state.status == "failed" => true,
                    Some(state) if state.status == "queued" || state.status == "running" => true,
                    Some(state) => state.status != "complete",
                };

            if !should_process {
                continue;
            }

            let concepts = self
                .conn
                .prepare(
                    "SELECT name FROM concept_records WHERE note_id = ?1 ORDER BY support_score DESC, name ASC",
                )?
                .query_map(params![&note_id], |row| row.get::<_, String>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let formulas = self
                .conn
                .prepare("SELECT latex FROM formula_records WHERE note_id = ?1 ORDER BY latex ASC")?
                .query_map(params![&note_id], |row| row.get::<_, String>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            candidates.push((
                note_id,
                content_hash,
                title,
                excerpt,
                headings,
                links,
                concepts,
                formulas,
            ));
        }

        Ok(candidates)
    }

    fn generate_course_brief(
        &self,
        course: &StoredCourse,
        settings: &AiProviderSettings,
    ) -> Result<ai::AiCourseBriefPayload> {
        let notes = self.list_notes(&course.id)?;
        let weak_rows = self.list_weak_suggestions(&course.id)?;
        let concepts = self.list_concepts(&course.id)?;
        let mut note_payload = Vec::new();

        for note in notes.iter().take(24) {
            if let Some(insight) = self.get_cached_note_ai_insight(&note.id, &note.content_hash)? {
                note_payload.push(format!(
                    "Title: {}\nSummary: {}\nTakeaways: {}\nQuestions: {}",
                    note.title,
                    insight.summary,
                    insight.takeaways.join(" | "),
                    insight.exam_questions.join(" | ")
                ));
            }
        }

        if note_payload.is_empty() {
            bail!("AI note insights are not ready yet");
        }

        ai::generate_course_brief(
            settings,
            &course.name,
            &build_top_concepts(&concepts)
                .into_iter()
                .map(|concept| concept.name)
                .collect::<Vec<_>>(),
            &build_weak_notes(&notes, &weak_rows)
                .into_iter()
                .map(|note| note.title)
                .collect::<Vec<_>>(),
            &note_payload.join("\n---\n"),
        )
    }

    fn list_courses(&self) -> Result<Vec<StoredCourse>> {
        let mut statement = self.conn.prepare(
            "SELECT id, name, folder, exam_date, revision_folder, flashcards_folder FROM course_configs ORDER BY name ASC",
        )?;
        let rows = statement
            .query_map([], |row| {
                Ok(StoredCourse {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    folder: row.get(2)?,
                    exam_date: row.get(3)?,
                    revision_folder: row.get(4)?,
                    flashcards_folder: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn find_course(&self, course_id: &str) -> Result<Option<StoredCourse>> {
        self.conn
            .query_row(
                "SELECT id, name, folder, exam_date, revision_folder, flashcards_folder FROM course_configs WHERE id = ?1",
                params![course_id],
                |row| {
                    Ok(StoredCourse {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        folder: row.get(2)?,
                        exam_date: row.get(3)?,
                        revision_folder: row.get(4)?,
                        flashcards_folder: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    fn find_course_id_by_folder(&self, folder: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT id FROM course_configs WHERE folder = ?1",
                params![folder],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(Into::into)
    }

    fn get_courses_with_metrics(&self) -> Result<Vec<CourseConfig>> {
        self.list_courses()?
            .into_iter()
            .map(|course| {
                let coverage = self.compute_coverage_for_course(&course.id)?;
                let note_count: usize = self.conn.query_row(
                    "SELECT COUNT(*) FROM note_records WHERE course_id = ?1",
                    params![course.id],
                    |row| row.get::<_, i64>(0).map(|value| value as usize),
                )?;
                let concept_count: usize = self.conn.query_row(
                    "SELECT COUNT(DISTINCT normalized_name) FROM concept_records WHERE course_id = ?1",
                    params![course.id],
                    |row| row.get::<_, i64>(0).map(|value| value as usize),
                )?;
                let formula_count: usize = self.conn.query_row(
                    "SELECT COUNT(DISTINCT normalized_latex) FROM formula_records WHERE course_id = ?1",
                    params![course.id],
                    |row| row.get::<_, i64>(0).map(|value| value as usize),
                )?;
                let weak_note_count: usize = self.conn.query_row(
                    "SELECT COUNT(DISTINCT note_id) FROM weak_link_suggestions WHERE course_id = ?1",
                    params![course.id],
                    |row| row.get::<_, i64>(0).map(|value| value as usize),
                )?;

                Ok(CourseConfig {
                    id: course.id,
                    name: course.name,
                    folder: course.folder,
                    exam_date: course.exam_date,
                    revision_folder: course.revision_folder,
                    flashcards_folder: course.flashcards_folder,
                    note_count,
                    concept_count,
                    formula_count,
                    coverage: coverage.percentage,
                    weak_note_count,
                })
            })
            .collect()
    }

    fn get_selected_course_id(&self) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT selected_course_id FROM app_state WHERE id = 1",
                [],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()
            .map(|value| value.flatten())
            .map_err(Into::into)
    }

    fn set_selected_course(&self, course_id: Option<&str>) -> Result<()> {
        self.conn.execute(
            "UPDATE app_state SET selected_course_id = ?1 WHERE id = 1",
            params![course_id],
        )?;
        Ok(())
    }

    fn get_scan_status(&self) -> Result<Option<ScanStatus>> {
        let status = self.conn.query_row(
            "SELECT last_scan_at, last_note_count, last_changed_count, last_removed_count FROM app_state WHERE id = 1",
            [],
            |row| {
                Ok(ScanStatus {
                    last_scan_at: row.get(0)?,
                    note_count: row.get::<_, i64>(1)? as usize,
                    changed_count: row.get::<_, i64>(2)? as usize,
                    removed_count: row.get::<_, i64>(3)? as usize,
                })
            },
        )?;

        if status.last_scan_at.is_none() && status.note_count == 0 {
            return Ok(None);
        }
        Ok(Some(status))
    }

    fn reset_app_state(&self) -> Result<()> {
        self.conn.execute(
            "UPDATE app_state SET selected_course_id = NULL, last_scan_at = NULL, last_note_count = 0, last_changed_count = 0, last_removed_count = 0 WHERE id = 1",
            [],
        )?;
        Ok(())
    }

    fn existing_note_hashes(&self, course_id: &str) -> Result<HashMap<String, String>> {
        let mut statement = self
            .conn
            .prepare("SELECT id, content_hash FROM note_records WHERE course_id = ?1")?;
        let rows = statement
            .query_map(params![course_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows.into_iter().collect())
    }

    fn prepare_note_for_storage(
        &self,
        file_stem: &str,
        content: &str,
    ) -> Result<ParsedStorageNote> {
        let parsed = parse_markdown(file_stem, content);
        let concepts = parsed
            .concepts
            .iter()
            .map(|concept| ParsedConcept {
                name: concept.clone(),
                normalized_name: normalize_key(concept),
                source: "local".to_string(),
                support_score: 1.0,
            })
            .collect::<Vec<_>>();

        Ok(ParsedStorageNote {
            title: parsed.title,
            excerpt: parsed.excerpt,
            headings: parsed.headings,
            links: parsed.links,
            tags: parsed.tags,
            prerequisites: Vec::new(),
            concepts,
            formulas: parsed.formulas,
            frontmatter_raw: parsed.frontmatter.raw,
            frontmatter_exam_date: parsed.frontmatter.exam_date,
        })
    }

    fn upsert_note(
        &self,
        course_id: &str,
        note_id: &str,
        relative_path: &str,
        content_hash: &str,
        parsed: ParsedStorageNote,
    ) -> Result<()> {
        let now = now_string();
        self.conn.execute(
            r#"
            INSERT INTO note_records (
              id, course_id, relative_path, title, content_hash, frontmatter, frontmatter_exam_date,
              excerpt, headings_json, links_json, tags_json, prerequisites_json, concept_count,
              formula_count, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?15)
            ON CONFLICT(id) DO UPDATE SET
              relative_path = excluded.relative_path,
              title = excluded.title,
              content_hash = excluded.content_hash,
              frontmatter = excluded.frontmatter,
              frontmatter_exam_date = excluded.frontmatter_exam_date,
              excerpt = excluded.excerpt,
              headings_json = excluded.headings_json,
              links_json = excluded.links_json,
              tags_json = excluded.tags_json,
              prerequisites_json = excluded.prerequisites_json,
              concept_count = excluded.concept_count,
              formula_count = excluded.formula_count,
              updated_at = excluded.updated_at
            "#,
            params![
                note_id,
                course_id,
                relative_path,
                parsed.title,
                content_hash,
                parsed.frontmatter_raw,
                parsed.frontmatter_exam_date,
                parsed.excerpt,
                to_json(&parsed.headings),
                to_json(&parsed.links),
                to_json(&parsed.tags),
                to_json(&parsed.prerequisites),
                parsed.concepts.len() as i64,
                parsed.formulas.len() as i64,
                now,
            ],
        )?;

        self.conn.execute(
            "DELETE FROM concept_records WHERE note_id = ?1",
            params![note_id],
        )?;
        self.conn.execute(
            "DELETE FROM formula_records WHERE note_id = ?1",
            params![note_id],
        )?;

        let mut seen_concepts = HashSet::new();
        for concept in parsed.concepts {
            if !seen_concepts.insert(concept.normalized_name.clone()) {
                continue;
            }
            self.conn.execute(
                r#"
                INSERT INTO concept_records (id, note_id, course_id, name, normalized_name, source, support_score)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![
                    build_concept_id(note_id, &concept.normalized_name),
                    note_id,
                    course_id,
                    concept.name,
                    concept.normalized_name,
                    concept.source,
                    concept.support_score,
                ],
            )?;
        }

        let mut seen_formulas = HashSet::new();
        for formula in parsed.formulas {
            let normalized_formula = normalize_key(&formula);
            if !seen_formulas.insert(normalized_formula.clone()) {
                continue;
            }
            self.conn.execute(
                r#"
                INSERT INTO formula_records (id, note_id, course_id, latex, normalized_latex, source)
                VALUES (?1, ?2, ?3, ?4, ?5, 'local')
                "#,
                params![
                    build_formula_id(note_id, &formula),
                    note_id,
                    course_id,
                    formula,
                    normalized_formula,
                ],
            )?;
        }

        Ok(())
    }

    fn delete_note(&self, note_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM note_records WHERE id = ?1", params![note_id])?;
        Ok(())
    }

    fn rebuild_course_analytics(&self, course: &StoredCourse) -> Result<(usize, usize)> {
        self.conn.execute(
            "DELETE FROM dependency_edges WHERE course_id = ?1",
            params![course.id],
        )?;
        self.conn.execute(
            "DELETE FROM weak_link_suggestions WHERE course_id = ?1",
            params![course.id],
        )?;

        let notes = self.list_notes(&course.id)?;
        if notes.is_empty() {
            return Ok((0, 0));
        }

        let concepts = self.list_concepts(&course.id)?;
        let formulas = self.list_formulas(&course.id)?;
        let note_by_id = notes
            .iter()
            .map(|note| (note.id.clone(), note.clone()))
            .collect::<HashMap<_, _>>();
        let mut title_index = HashMap::<String, String>::new();
        for note in &notes {
            for key in note_title_candidates(&note.title, &note.relative_path).into_keys() {
                title_index.insert(key, note.id.clone());
            }
        }

        let mut edges = BTreeMap::<(String, String, String), StoredEdge>::new();
        let mut explicit_incident = HashMap::<String, usize>::new();

        for note in &notes {
            for link in &note.links {
                let target_key = normalize_key(link);
                if let Some(target_id) = title_index.get(&target_key) {
                    if target_id != &note.id {
                        let edge = StoredEdge {
                            from_note_id: note.id.clone(),
                            to_note_id: target_id.clone(),
                            edge_type: "wikilink".to_string(),
                            weight: 1.0,
                            rationale: link.clone(),
                        };
                        explicit_incident
                            .entry(note.id.clone())
                            .and_modify(|value| *value += 1)
                            .or_insert(1);
                        explicit_incident
                            .entry(target_id.clone())
                            .and_modify(|value| *value += 1)
                            .or_insert(1);
                        edges.insert(
                            (
                                edge.from_note_id.clone(),
                                edge.to_note_id.clone(),
                                edge.edge_type.clone(),
                            ),
                            edge,
                        );
                    }
                }
            }

            for prerequisite in &note.prerequisites {
                let key = normalize_key(prerequisite);
                if let Some(target_id) = title_index.get(&key) {
                    if target_id != &note.id {
                        let edge = StoredEdge {
                            from_note_id: target_id.clone(),
                            to_note_id: note.id.clone(),
                            edge_type: "ai-prerequisite".to_string(),
                            weight: 1.2,
                            rationale: prerequisite.clone(),
                        };
                        explicit_incident
                            .entry(note.id.clone())
                            .and_modify(|value| *value += 1)
                            .or_insert(1);
                        explicit_incident
                            .entry(target_id.clone())
                            .and_modify(|value| *value += 1)
                            .or_insert(1);
                        edges.insert(
                            (
                                edge.from_note_id.clone(),
                                edge.to_note_id.clone(),
                                edge.edge_type.clone(),
                            ),
                            edge,
                        );
                    }
                }
            }
        }

        let mut relation_scores = HashMap::<(String, String), f64>::new();
        accumulate_relation_scores(&mut relation_scores, &group_concepts_by_key(&concepts), 1.0);
        accumulate_relation_scores(&mut relation_scores, &group_formulas_by_key(&formulas), 1.5);

        for ((left, right), score) in &relation_scores {
            let edge = StoredEdge {
                from_note_id: left.clone(),
                to_note_id: right.clone(),
                edge_type: "concept-overlap".to_string(),
                weight: *score,
                rationale: "Shared concepts or formulas".to_string(),
            };
            edges.insert(
                (
                    edge.from_note_id.clone(),
                    edge.to_note_id.clone(),
                    edge.edge_type.clone(),
                ),
                edge,
            );
        }

        for edge in edges.values() {
            self.conn.execute(
                r#"
                INSERT INTO dependency_edges (id, course_id, from_note_id, to_note_id, edge_type, weight, rationale)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![
                    build_edge_id(&course.id, &edge.from_note_id, &edge.to_note_id, &edge.edge_type),
                    course.id,
                    edge.from_note_id,
                    edge.to_note_id,
                    edge.edge_type,
                    edge.weight,
                    edge.rationale,
                ],
            )?;
        }

        let weak_rows = build_weak_rows(&notes, &note_by_id, &relation_scores, &explicit_incident);
        for row in &weak_rows {
            self.conn.execute(
                r#"
                INSERT INTO weak_link_suggestions (id, course_id, note_id, related_note_id, score, reason)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
                params![
                    build_weak_id(&course.id, &row.note_id, row.related_note_id.as_deref()),
                    course.id,
                    row.note_id,
                    row.related_note_id,
                    row.score,
                    row.reason,
                ],
            )?;
        }

        Ok((edges.len(), weak_rows.len()))
    }

    fn list_notes(&self, course_id: &str) -> Result<Vec<StoredNote>> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT id, title, relative_path, content_hash, links_json, prerequisites_json, frontmatter_exam_date
            FROM note_records
            WHERE course_id = ?1
            ORDER BY title ASC
            "#,
        )?;
        let rows = statement
            .query_map(params![course_id], |row| {
                Ok(StoredNote {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    relative_path: row.get(2)?,
                    content_hash: row.get(3)?,
                    links: from_json_vec::<String>(&row.get::<_, String>(4)?),
                    prerequisites: from_json_vec::<String>(&row.get::<_, String>(5)?),
                    frontmatter_exam_date: row.get(6)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn list_concepts(&self, course_id: &str) -> Result<Vec<(String, String, String, f64)>> {
        let mut statement = self.conn.prepare(
            "SELECT note_id, name, normalized_name, support_score FROM concept_records WHERE course_id = ?1 ORDER BY support_score DESC",
        )?;
        let rows = statement
            .query_map(params![course_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, f64>(3)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn list_formulas(&self, course_id: &str) -> Result<Vec<(String, String, String)>> {
        let mut statement = self.conn.prepare(
            "SELECT note_id, latex, normalized_latex FROM formula_records WHERE course_id = ?1 ORDER BY latex ASC",
        )?;
        let rows = statement
            .query_map(params![course_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn list_edges(&self, course_id: &str) -> Result<Vec<StoredEdge>> {
        let mut statement = self.conn.prepare(
            "SELECT from_note_id, to_note_id, edge_type, weight, rationale FROM dependency_edges WHERE course_id = ?1",
        )?;
        let rows = statement
            .query_map(params![course_id], |row| {
                Ok(StoredEdge {
                    from_note_id: row.get(0)?,
                    to_note_id: row.get(1)?,
                    edge_type: row.get(2)?,
                    weight: row.get(3)?,
                    rationale: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn list_weak_suggestions(&self, course_id: &str) -> Result<Vec<StoredWeakSuggestion>> {
        let mut statement = self.conn.prepare(
            "SELECT note_id, related_note_id, score, reason FROM weak_link_suggestions WHERE course_id = ?1 ORDER BY score DESC",
        )?;
        let rows = statement
            .query_map(params![course_id], |row| {
                Ok(StoredWeakSuggestion {
                    note_id: row.get(0)?,
                    related_note_id: row.get(1)?,
                    score: row.get(2)?,
                    reason: row.get(3)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn list_flashcard_sets(&self, course_id: &str) -> Result<Vec<StoredFlashcardSet>> {
        let mut statement = self.conn.prepare(
            "SELECT source_note_ids_json, markdown_path, csv_path, card_count, created_at FROM flashcard_sets WHERE course_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = statement
            .query_map(params![course_id], |row| {
                Ok(StoredFlashcardSet {
                    source_note_ids: from_json_vec::<String>(&row.get::<_, String>(0)?),
                    csv_path: row.get(2)?,
                    card_count: row.get::<_, i64>(3)? as usize,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn list_revision_runs(&self, course_id: &str) -> Result<Vec<StoredRevisionRun>> {
        let mut statement = self.conn.prepare(
            "SELECT note_path, item_count, created_at FROM revision_note_runs WHERE course_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = statement
            .query_map(params![course_id], |row| {
                Ok(StoredRevisionRun {
                    note_path: row.get(0)?,
                    item_count: row.get::<_, i64>(1)? as usize,
                    created_at: row.get(2)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn total_note_count(&self) -> Result<usize> {
        self.conn
            .query_row("SELECT COUNT(*) FROM note_records", [], |row| {
                row.get::<_, i64>(0).map(|value| value as usize)
            })
            .map_err(Into::into)
    }

    fn compute_coverage_for_course(&self, course_id: &str) -> Result<CoverageStats> {
        let notes = self.list_notes(course_id)?;
        let concepts = self.list_concepts(course_id)?;
        let formulas = self.list_formulas(course_id)?;
        let flashcards = self.list_flashcard_sets(course_id)?;
        Ok(build_coverage(&notes, &concepts, &formulas, &flashcards))
    }
}

fn build_countdown(course: &StoredCourse, notes: &[StoredNote]) -> Countdown {
    let exam_date = course.exam_date.clone().or_else(|| {
        notes
            .iter()
            .find_map(|note| note.frontmatter_exam_date.clone())
    });

    if let Some(exam_date) = exam_date {
        if let Some(days_remaining) = parse_days_remaining(&exam_date) {
            let label = if days_remaining < 0 {
                format!("Exam passed {} days ago", days_remaining.abs())
            } else if days_remaining == 0 {
                "Exam is today".to_string()
            } else {
                format!("{days_remaining} days until exam")
            };

            return Countdown {
                exam_date: Some(exam_date),
                days_remaining: Some(days_remaining),
                label,
            };
        }

        return Countdown {
            exam_date: Some(exam_date),
            days_remaining: None,
            label: "Exam date stored".to_string(),
        };
    }

    Countdown {
        exam_date: None,
        days_remaining: None,
        label: "No exam date configured".to_string(),
    }
}

fn build_coverage(
    notes: &[StoredNote],
    concepts: &[(String, String, String, f64)],
    formulas: &[(String, String, String)],
    flashcard_sets: &[StoredFlashcardSet],
) -> CoverageStats {
    let formula_notes = formulas
        .iter()
        .map(|(note_id, _, _)| note_id.clone())
        .collect::<HashSet<_>>();
    let flashcard_notes = flashcard_sets
        .iter()
        .flat_map(|set| set.source_note_ids.iter().cloned())
        .collect::<HashSet<_>>();
    let link_notes = notes
        .iter()
        .filter(|note| !note.links.is_empty() || !note.prerequisites.is_empty())
        .map(|note| note.id.clone())
        .collect::<HashSet<_>>();

    let strong_notes = notes
        .iter()
        .filter(|note| {
            formula_notes.contains(&note.id)
                || flashcard_notes.contains(&note.id)
                || link_notes.contains(&note.id)
        })
        .map(|note| note.id.clone())
        .collect::<HashSet<_>>();

    let mut total = HashSet::new();
    let mut covered = HashSet::new();
    for (note_id, _, normalized, _) in concepts {
        total.insert(normalized.clone());
        if strong_notes.contains(note_id) {
            covered.insert(normalized.clone());
        }
    }

    let total_count = total.len();
    let covered_count = covered.len();
    let percentage = if total_count == 0 {
        0.0
    } else {
        (covered_count as f64 / total_count as f64) * 100.0
    };

    CoverageStats {
        total_concepts: total_count,
        covered_concepts: covered_count,
        percentage: round_percentage(percentage),
    }
}

fn build_graph_stats(notes: &[StoredNote], edges: &[StoredEdge]) -> GraphStats {
    let mut incident = HashMap::<String, usize>::new();
    let mut strong_links = 0usize;
    let mut inferred_links = 0usize;

    for edge in edges {
        *incident.entry(edge.from_note_id.clone()).or_insert(0) += 1;
        *incident.entry(edge.to_note_id.clone()).or_insert(0) += 1;
        if edge.edge_type == "concept-overlap" {
            inferred_links += 1;
        } else {
            strong_links += 1;
        }
    }

    let isolated_notes = notes
        .iter()
        .filter(|note| incident.get(&note.id).copied().unwrap_or(0) == 0)
        .count();

    GraphStats {
        note_count: notes.len(),
        edge_count: edges.len(),
        strong_links,
        inferred_links,
        isolated_notes,
    }
}

fn build_weak_notes(notes: &[StoredNote], rows: &[StoredWeakSuggestion]) -> Vec<WeakNote> {
    let mut grouped = BTreeMap::<String, Vec<&StoredWeakSuggestion>>::new();
    for row in rows {
        grouped.entry(row.note_id.clone()).or_default().push(row);
    }

    let note_lookup = notes
        .iter()
        .map(|note| (note.id.clone(), note))
        .collect::<HashMap<_, _>>();

    grouped
        .into_iter()
        .filter_map(|(note_id, entries)| {
            let note = note_lookup.get(&note_id)?;
            let score = entries.iter().map(|entry| entry.score).fold(0.0, f64::max);
            let suggestions = entries
                .iter()
                .map(|entry| entry.reason.clone())
                .collect::<Vec<_>>();
            Some(WeakNote {
                note_id,
                title: note.title.clone(),
                relative_path: note.relative_path.clone(),
                score: (score * 100.0).round() / 100.0,
                suggestions,
            })
        })
        .take(10)
        .collect()
}

fn build_top_concepts(concepts: &[(String, String, String, f64)]) -> Vec<ConceptMetric> {
    let mut grouped = HashMap::<String, (String, HashSet<String>, f64)>::new();
    for (note_id, name, normalized_name, support_score) in concepts {
        let entry = grouped
            .entry(normalized_name.clone())
            .or_insert_with(|| (name.clone(), HashSet::new(), 0.0));
        entry.1.insert(note_id.clone());
        entry.2 += *support_score;
    }

    let mut metrics = grouped
        .into_iter()
        .map(|(_, (name, note_ids, support_score))| ConceptMetric {
            name,
            note_count: note_ids.len(),
            support_score: (support_score * 100.0).round() / 100.0,
        })
        .collect::<Vec<_>>();
    metrics.sort_by(|left, right| {
        right
            .support_score
            .total_cmp(&left.support_score)
            .then_with(|| right.note_count.cmp(&left.note_count))
            .then_with(|| left.name.cmp(&right.name))
    });
    metrics.truncate(8);
    metrics
}

fn build_top_formulas(formulas: &[(String, String, String)]) -> Vec<FormulaMetric> {
    let mut grouped = HashMap::<String, (String, HashSet<String>)>::new();
    for (note_id, latex, normalized) in formulas {
        let entry = grouped
            .entry(normalized.clone())
            .or_insert_with(|| (latex.clone(), HashSet::new()));
        entry.1.insert(note_id.clone());
    }

    let mut metrics = grouped
        .into_iter()
        .map(|(_, (latex, note_ids))| FormulaMetric {
            latex,
            note_count: note_ids.len(),
        })
        .collect::<Vec<_>>();
    metrics.sort_by(|left, right| {
        right
            .note_count
            .cmp(&left.note_count)
            .then_with(|| left.latex.cmp(&right.latex))
    });
    metrics.truncate(6);
    metrics
}

fn build_flashcard_summary(flashcard_sets: &[StoredFlashcardSet]) -> FlashcardSummary {
    let total_cards = flashcard_sets.iter().map(|set| set.card_count).sum();
    let latest = flashcard_sets.first();
    FlashcardSummary {
        set_count: flashcard_sets.len(),
        total_cards,
        last_generated_at: latest.map(|set| set.created_at.clone()),
        export_path: latest.and_then(|set| set.csv_path.clone()),
    }
}

fn build_revision_summary(runs: &[StoredRevisionRun]) -> RevisionSummary {
    if let Some(run) = runs.first() {
        return RevisionSummary {
            last_generated_at: Some(run.created_at.clone()),
            note_path: Some(run.note_path.clone()),
            item_count: run.item_count,
        };
    }

    RevisionSummary {
        last_generated_at: None,
        note_path: None,
        item_count: 0,
    }
}

fn build_note_summaries(
    notes: &[StoredNote],
    edges: &[StoredEdge],
    concepts: &[(String, String, String, f64)],
    formulas: &[(String, String, String)],
    ai_states: &HashMap<String, StoredAiNoteState>,
) -> Vec<NoteSummary> {
    let mut incident = HashMap::<String, usize>::new();
    for edge in edges {
        *incident.entry(edge.from_note_id.clone()).or_insert(0) += 1;
        *incident.entry(edge.to_note_id.clone()).or_insert(0) += 1;
    }

    let mut concept_count = HashMap::<String, usize>::new();
    for (note_id, _, _, _) in concepts {
        *concept_count.entry(note_id.clone()).or_insert(0) += 1;
    }

    let mut formula_count = HashMap::<String, usize>::new();
    for (note_id, _, _) in formulas {
        *formula_count.entry(note_id.clone()).or_insert(0) += 1;
    }

    let mut summaries = notes
        .iter()
        .map(|note| {
            let link_count = incident.get(&note.id).copied().unwrap_or(0);
            let concept_total = concept_count.get(&note.id).copied().unwrap_or(0);
            let formula_total = formula_count.get(&note.id).copied().unwrap_or(0);
            let strength =
                link_count as f64 * 0.6 + concept_total as f64 * 0.3 + formula_total as f64 * 1.2;
            NoteSummary {
                id: note.id.clone(),
                title: note.title.clone(),
                relative_path: note.relative_path.clone(),
                link_count,
                concept_count: concept_total,
                formula_count: formula_total,
                strength: (strength * 100.0).round() / 100.0,
                ai_status: current_ai_status(note, ai_states),
            }
        })
        .collect::<Vec<_>>();

    summaries.sort_by(|left, right| {
        right
            .strength
            .total_cmp(&left.strength)
            .then_with(|| left.title.cmp(&right.title))
    });
    summaries
}

fn current_ai_status(note: &StoredNote, ai_states: &HashMap<String, StoredAiNoteState>) -> String {
    match ai_states.get(&note.id) {
        None => "missing".to_string(),
        Some(state) if state.content_hash != note.content_hash => "stale".to_string(),
        Some(state) => state.status.clone(),
    }
}

fn build_ai_course_summary(
    total_notes: usize,
    ai_settings: Option<&AiSettings>,
    notes: &[StoredNote],
    ai_states: &HashMap<String, StoredAiNoteState>,
    ai_run: Option<&StoredAiCourseRun>,
) -> AiCourseSummary {
    let counts = compute_ai_status_counts(notes, ai_states);
    let default_status = if let Some(settings) = ai_settings {
        if settings.enabled {
            if counts.pending_notes > 0 {
                "running"
            } else if counts.ready_notes == counts.total_notes && counts.total_notes > 0 {
                "complete"
            } else if counts.failed_notes > 0 && counts.ready_notes == 0 {
                "failed"
            } else if counts.ready_notes > 0 {
                "partial"
            } else {
                "idle"
            }
        } else {
            "disabled"
        }
    } else {
        "disabled"
    };
    let status = match ai_run.map(|run| run.status.as_str()) {
        Some("running") => "running".to_string(),
        Some("failed") if counts.ready_notes == 0 => "failed".to_string(),
        _ => default_status.to_string(),
    };

    AiCourseSummary {
        status,
        total_notes: total_notes.max(counts.total_notes),
        ready_notes: counts.ready_notes,
        pending_notes: counts.pending_notes,
        failed_notes: counts.failed_notes,
        stale_notes: counts.stale_notes,
        missing_notes: counts.missing_notes,
        started_at: ai_run.and_then(|run| run.started_at.clone()),
        finished_at: ai_run.and_then(|run| run.finished_at.clone()),
        updated_at: ai_run.and_then(|run| run.updated_at.clone()),
        model: ai_run.and_then(|run| run.model.clone()),
        summary: ai_run.and_then(|run| run.summary.clone()),
        revision_priorities: ai_run
            .map(|run| run.revision_priorities.clone())
            .unwrap_or_default(),
        weak_spots: ai_run.map(|run| run.weak_spots.clone()).unwrap_or_default(),
        next_actions: ai_run
            .map(|run| run.next_actions.clone())
            .unwrap_or_default(),
        last_error: ai_run.and_then(|run| run.last_error.clone()),
    }
}

fn compute_ai_status_counts(
    notes: &[StoredNote],
    ai_states: &HashMap<String, StoredAiNoteState>,
) -> AiStatusCounts {
    let mut counts = AiStatusCounts {
        total_notes: notes.len(),
        ..AiStatusCounts::default()
    };

    for note in notes {
        match current_ai_status(note, ai_states).as_str() {
            "complete" => counts.ready_notes += 1,
            "queued" | "running" => counts.pending_notes += 1,
            "failed" => counts.failed_notes += 1,
            "stale" => {
                counts.stale_notes += 1;
                counts.pending_notes += 1;
            }
            _ => counts.missing_notes += 1,
        }
    }

    counts
}

fn truncate_error(message: &str) -> String {
    let limit = 320usize;
    if message.chars().count() <= limit {
        return message.to_string();
    }

    message.chars().take(limit).collect::<String>()
}

fn group_concepts_by_key(
    concepts: &[(String, String, String, f64)],
) -> HashMap<String, HashSet<String>> {
    let mut grouped = HashMap::<String, HashSet<String>>::new();
    for (note_id, _, normalized, _) in concepts {
        grouped
            .entry(normalized.clone())
            .or_default()
            .insert(note_id.clone());
    }
    grouped
}

fn group_formulas_by_key(
    formulas: &[(String, String, String)],
) -> HashMap<String, HashSet<String>> {
    let mut grouped = HashMap::<String, HashSet<String>>::new();
    for (note_id, _, normalized) in formulas {
        grouped
            .entry(normalized.clone())
            .or_default()
            .insert(note_id.clone());
    }
    grouped
}

fn accumulate_relation_scores(
    relation_scores: &mut HashMap<(String, String), f64>,
    grouped: &HashMap<String, HashSet<String>>,
    weight: f64,
) {
    for note_ids in grouped.values() {
        let values = note_ids.iter().cloned().collect::<Vec<_>>();
        if values.len() < 2 {
            continue;
        }

        for left_index in 0..values.len() {
            for right_index in (left_index + 1)..values.len() {
                let pair = ordered_pair(&values[left_index], &values[right_index]);
                *relation_scores.entry(pair).or_insert(0.0) += weight;
            }
        }
    }
}

fn build_weak_rows(
    notes: &[StoredNote],
    note_lookup: &HashMap<String, StoredNote>,
    relation_scores: &HashMap<(String, String), f64>,
    explicit_incident: &HashMap<String, usize>,
) -> Vec<StoredWeakSuggestion> {
    let mut relation_map = HashMap::<String, Vec<(String, f64)>>::new();
    for ((left, right), score) in relation_scores {
        relation_map
            .entry(left.clone())
            .or_default()
            .push((right.clone(), *score));
        relation_map
            .entry(right.clone())
            .or_default()
            .push((left.clone(), *score));
    }

    let mut rows = Vec::new();
    for note in notes {
        let degree = explicit_incident.get(&note.id).copied().unwrap_or(0) as f64;
        let mut related = relation_map.remove(&note.id).unwrap_or_default();
        related.sort_by(|left, right| right.1.total_cmp(&left.1));
        let overlap_total: f64 = related.iter().take(3).map(|(_, score)| *score).sum();
        let weakness = 1.0 - (degree * 0.32 + overlap_total * 0.14).min(1.0);

        if weakness < 0.48 {
            continue;
        }

        if related.is_empty() {
            rows.push(StoredWeakSuggestion {
                note_id: note.id.clone(),
                related_note_id: None,
                score: weakness,
                reason: "Sparse note with few graph connections.".to_string(),
            });
            continue;
        }

        for (related_note_id, score) in related.into_iter().take(3) {
            let title = note_lookup
                .get(&related_note_id)
                .map(|candidate| candidate.title.clone())
                .unwrap_or_else(|| "Related note".to_string());
            rows.push(StoredWeakSuggestion {
                note_id: note.id.clone(),
                related_note_id: Some(related_note_id),
                score: weakness + score * 0.05,
                reason: format!("Consider linking to {title}"),
            });
        }
    }

    rows
}

fn parse_days_remaining(raw: &str) -> Option<i64> {
    if let Ok(date) = NaiveDate::parse_from_str(raw, "%Y-%m-%d") {
        return Some((date - Utc::now().date_naive()).num_days());
    }
    if let Ok(date_time) = DateTime::parse_from_rfc3339(raw) {
        return Some((date_time.date_naive() - Utc::now().date_naive()).num_days());
    }
    None
}

fn build_local_flashcards(notes: &[NoteDetails]) -> Vec<FlashcardCard> {
    let mut cards = Vec::new();
    for note in notes {
        for concept in note.concepts.iter().take(4) {
            let answer = if note.excerpt.is_empty() {
                format!("Review {} in {}", concept, note.title)
            } else {
                note.excerpt.clone()
            };
            cards.push(FlashcardCard {
                question: format!("What should you recall about {}?", concept),
                answer,
            });
        }

        for formula in note.formulas.iter().take(2) {
            cards.push(FlashcardCard {
                question: format!("Which formula is central to {}?", note.title),
                answer: formula.clone(),
            });
        }
    }
    cards
}

fn dedupe_cards(cards: &mut Vec<FlashcardCard>) {
    let mut seen = HashSet::new();
    cards.retain(|card| {
        let key = format!(
            "{}::{}",
            normalize_key(&card.question),
            normalize_key(&card.answer)
        );
        seen.insert(key)
    });
}

fn render_flashcard_markdown(
    course_name: &str,
    generated_at: &str,
    cards: &[FlashcardCard],
) -> String {
    let mut markdown = format!("# Flashcards: {course_name}\n\nGenerated: {generated_at}\n\n");
    for (index, card) in cards.iter().enumerate() {
        markdown.push_str(&format!(
            "## Card {}\n\n**Q** {}\n\n**A** {}\n\n",
            index + 1,
            card.question,
            card.answer
        ));
    }
    markdown
}

fn render_revision_markdown(course_name: &str, dashboard: &DashboardData) -> String {
    let mut markdown = format!(
        "# Today's Revision: {course_name}\n\n## Countdown\n\n{}\n\n## Coverage\n\n- {} of {} concepts covered ({:.1}%)\n- {} notes indexed\n- {} graph edges mapped\n\n## Priority Queue\n\n",
        dashboard.countdown.label,
        dashboard.coverage.covered_concepts,
        dashboard.coverage.total_concepts,
        dashboard.coverage.percentage,
        dashboard.graph.note_count,
        dashboard.graph.edge_count,
    );

    for note in dashboard.notes.iter().take(8) {
        markdown.push_str(&format!(
            "- {} ({}) | links: {} | concepts: {} | formulas: {}\n",
            note.title, note.relative_path, note.link_count, note.concept_count, note.formula_count
        ));
    }

    markdown.push_str("\n## Weak Topics\n\n");
    for weak_note in dashboard.weak_notes.iter().take(5) {
        markdown.push_str(&format!(
            "- {} -> {}\n",
            weak_note.title,
            weak_note.suggestions.join("; ")
        ));
    }

    markdown.push_str("\n## Top Concepts\n\n");
    for concept in dashboard.top_concepts.iter().take(6) {
        markdown.push_str(&format!(
            "- {} ({:.2} support across {} notes)\n",
            concept.name, concept.support_score, concept.note_count
        ));
    }

    markdown.push_str("\n## Formula Focus\n\n");
    for formula in dashboard.formulas.iter().take(4) {
        markdown.push_str(&format!(
            "- `{}` ({} notes)\n",
            formula.latex, formula.note_count
        ));
    }

    markdown.push_str("\n## Flashcards Ready\n\n");
    markdown.push_str(&format!(
        "- {} cards across {} sets\n",
        dashboard.flashcards.total_cards, dashboard.flashcards.set_count
    ));
    markdown
}

fn to_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "[]".to_string())
}

fn from_json_vec<T>(value: &str) -> Vec<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    serde_json::from_str(value).unwrap_or_default()
}

fn bool_to_int(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn now_string() -> String {
    Utc::now().to_rfc3339()
}

fn file_stamp() -> String {
    Utc::now().format("%Y%m%d-%H%M%S").to_string()
}

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn normalize_relative_path(value: &str) -> String {
    value.replace('\\', "/").trim_start_matches('/').to_string()
}

fn directory_contains_markdown(path: &Path) -> bool {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .any(|entry| {
            entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .and_then(|value| value.to_str())
                    .map(|value| value.eq_ignore_ascii_case("md"))
                    .unwrap_or(false)
        })
}

fn nullable_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn resolve_api_key(stored: &str) -> Option<String> {
    let trimmed = stored.trim();
    if !trimmed.is_empty() {
        return Some(trimmed.to_string());
    }

    env::var("OPENROUTER_API_KEY")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            env::var("OPENAI_API_KEY")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
}

fn slugify(value: &str) -> String {
    let normalized = normalize_key(value);
    normalized.replace(' ', "-")
}

fn ordered_pair(left: &str, right: &str) -> (String, String) {
    if left <= right {
        (left.to_string(), right.to_string())
    } else {
        (right.to_string(), left.to_string())
    }
}

fn build_note_id(course_id: &str, relative_path: &str) -> String {
    format!(
        "note::{course_id}::{}",
        normalize_key(relative_path).replace(' ', "_")
    )
}

fn build_seed_course_id(folder: &str) -> String {
    format!("course::{}", normalize_key(folder).replace(' ', "_"))
}

fn build_concept_id(note_id: &str, normalized_name: &str) -> String {
    format!("concept::{note_id}::{}", normalized_name.replace(' ', "_"))
}

fn build_formula_id(note_id: &str, formula: &str) -> String {
    format!(
        "formula::{note_id}::{}",
        normalize_key(formula).replace(' ', "_")
    )
}

fn build_edge_id(course_id: &str, from_note_id: &str, to_note_id: &str, edge_type: &str) -> String {
    format!(
        "edge::{course_id}::{}::{}::{edge_type}",
        from_note_id.replace(':', "_"),
        to_note_id.replace(':', "_")
    )
}

fn build_weak_id(course_id: &str, note_id: &str, related_note_id: Option<&str>) -> String {
    format!(
        "weak::{course_id}::{}::{}",
        note_id.replace(':', "_"),
        related_note_id.unwrap_or("none").replace(':', "_")
    )
}

fn round_percentage(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::Database;
    use crate::models::{CourseConfigInput, FlashcardGenerationRequest, RevisionNoteRequest};

    #[test]
    fn scan_detects_changes_and_outputs_files() {
        let temp = tempdir().expect("temp dir");
        let vault_dir = temp.path().join("vault");
        let course_dir = vault_dir.join("math");
        fs::create_dir_all(&course_dir).expect("course dir");
        fs::write(
            course_dir.join("limits.md"),
            "# Limits\n\n**Epsilon delta** proof.\nSee [[continuity]].\n$\\lim_{x\\to a} f(x)$",
        )
        .expect("write note");
        fs::write(
            course_dir.join("continuity.md"),
            "# Continuity\n\nContinuity: function without jumps.",
        )
        .expect("write note");

        let db_path = temp.path().join("exam-os.sqlite");
        let database = Database::open(&db_path).expect("database");
        database
            .connect_vault(vault_dir.to_string_lossy().as_ref())
            .expect("connect vault");
        let course_id = database
            .save_course_config(CourseConfigInput {
                id: None,
                name: "Math".to_string(),
                folder: "math".to_string(),
                exam_date: Some("2026-07-01".to_string()),
                revision_folder: None,
                flashcards_folder: None,
            })
            .expect("save course");

        let first_scan = database.run_scan().expect("first scan");
        assert_eq!(first_scan.changed_notes, 2);

        fs::write(
            course_dir.join("continuity.md"),
            "# Continuity\n\nContinuity: function without jumps.\n\n[[limits]]",
        )
        .expect("update note");

        let second_scan = database.run_scan().expect("second scan");
        assert_eq!(second_scan.changed_notes, 1);

        let export_dir = temp.path().join("exports");
        let dashboard = database
            .get_dashboard(Some(course_id.clone()))
            .expect("dashboard")
            .expect("dashboard present");
        let note_ids = dashboard
            .notes
            .iter()
            .map(|note| note.id.clone())
            .collect::<Vec<_>>();
        let flashcards = database
            .generate_flashcards(
                FlashcardGenerationRequest {
                    course_id: course_id.clone(),
                    note_ids,
                    flashcards_folder: None,
                    export_csv: Some(true),
                },
                &export_dir,
            )
            .expect("flashcards");
        assert!(PathBuf::from(&flashcards.markdown_path).exists());
        assert!(PathBuf::from(flashcards.csv_path.expect("csv path")).exists());

        let revision = database
            .generate_revision_note(RevisionNoteRequest {
                course_id,
                revision_folder: None,
            })
            .expect("revision");
        assert!(PathBuf::from(revision.note_path).exists());
    }
}
