use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::ai::{self, AiProviderSettings, ChatContextChunkInput, FlashcardCard};
use crate::markdown::{normalize_key, note_title_candidates, parse_markdown};
use crate::models::{
    AiCourseSummary, AiNoteInsight, AiSettings, AiSettingsInput, ApplyExamReviewActionsRequest,
    ChatCitation, ChatMessage, ChatMessageRole, ChatScope, ChatThreadDetails, ChatThreadSummary,
    ConceptMetric, Countdown, CourseConfig, CourseConfigInput, CourseStatisticsRow, CoverageStats,
    CreateChatThreadRequest, DashboardData, ExamAnswerValue, ExamAttemptResult, ExamAttemptSummary,
    ExamBuilderInput, ExamDefaults, ExamDetails, ExamDifficulty, ExamPreset, ExamQuestion,
    ExamQuestionResult, ExamQuestionType, ExamReviewSuggestion, ExamSourceNote, ExamStatus,
    ExamSubmissionRequest, ExamSummary, ExamVerdict, ExamWorkspaceSnapshot, ExamWorkspaceSummary,
    FlashcardGenerationRequest, FlashcardGenerationResult, FlashcardSummary, FormulaBrief,
    FormulaDetails, FormulaLinkedNote, FormulaMetric, FormulaSummary, FormulaWorkspaceSnapshot,
    FormulaWorkspaceSummary, GenerateFormulaBriefRequest, GitCommitItem, GitCourseActivityRow,
    GitNoteActivityRow, GitSummary, GitTimelinePoint, GraphStats, NoteChunkPreview, NoteDetails,
    NoteMasteryState, NoteSummary, RevisionNoteRequest, RevisionNoteResult, RevisionSummary,
    ScanReport, ScanStatus, SendChatMessageRequest, StatisticsAiSection,
    StatisticsCountBucket, StatisticsExamPoint, StatisticsExamsSection, StatisticsHighlight,
    StatisticsKnowledgeSection, StatisticsKnowledgeSummary, StatisticsNoteRow,
    StatisticsNotesSection, StatisticsNotesSummary, StatisticsOutputsSection,
    StatisticsOutputsSummary, StatisticsOverview, StatisticsOverviewSection, StatisticsResponse,
    StatisticsScope, StatisticsSnapshotPoint, StatisticsValuePoint, StatisticsVaultActivitySection,
    StatisticsGitSection, StatisticsAiSummary, StatisticsExamsSummary, ValidationResult,
    VaultActivityBucket, VaultActivitySummary, VaultConfig, WeakNote, WorkspaceSnapshot,
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
    source_modified_at: Option<String>,
    excerpt: String,
    headings: Vec<String>,
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
    chunks: Vec<ParsedNoteChunk>,
}

#[derive(Debug, Clone)]
struct ParsedNoteChunk {
    heading_path: String,
    text: String,
    ordinal: usize,
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

#[derive(Debug, Clone)]
struct StoredExamRecord {
    id: String,
    course_id: String,
    title: String,
    preset: ExamPreset,
    status: ExamStatus,
    difficulty: ExamDifficulty,
    question_count: usize,
    source_note_count: usize,
    multiple_choice_count: usize,
    short_answer_count: usize,
    time_limit_minutes: usize,
    created_at: String,
    updated_at: String,
    generated_at: Option<String>,
    instructions: String,
    summary: String,
    source_note_ids: Vec<String>,
    last_error: Option<String>,
    _model: Option<String>,
}

#[derive(Debug, Clone)]
struct StoredExamAttempt {
    course_id: String,
}

#[derive(Debug, Clone)]
struct StoredExamGenerationJob {
    exam: StoredExamRecord,
    notes: Vec<ai::ExamGenerationNoteInput>,
}

#[derive(Debug, Clone)]
struct StoredNoteMastery {
    note_id: String,
    mastery_state: NoteMasteryState,
    last_accuracy: Option<f64>,
}

#[derive(Debug, Clone)]
struct StoredNoteChunk {
    chunk_id: String,
    note_id: String,
    course_id: String,
    note_title: String,
    relative_path: String,
    heading_path: String,
    text: String,
    ordinal: usize,
}

#[derive(Debug, Clone)]
struct StoredFormulaAggregate {
    id: String,
    latex: String,
    normalized_latex: String,
    note_count: usize,
    source_note_ids: Vec<String>,
    source_note_titles: Vec<String>,
    source_hash: String,
}

#[derive(Debug, Clone)]
struct StoredChatThread {
    id: String,
    scope: ChatScope,
    course_id: Option<String>,
    title: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone)]
struct CourseStatisticsBundle {
    overview: StatisticsOverview,
    flashcards: FlashcardSummary,
    revision: RevisionSummary,
    notes: Vec<StatisticsNoteRow>,
    concepts: Vec<ConceptMetric>,
    formulas: Vec<FormulaMetric>,
}

#[derive(Debug, Clone)]
struct GitCommitRecord {
    sha: String,
    committed_at: String,
    author_name: String,
    summary: String,
    paths: Vec<String>,
}

#[derive(Debug, Clone)]
struct GitAnalytics {
    summary: GitSummary,
    commit_timeline: Vec<GitTimelinePoint>,
    churn_timeline: Vec<GitTimelinePoint>,
    course_activity: Vec<GitCourseActivityRow>,
    top_notes: Vec<GitNoteActivityRow>,
    recent_commits: Vec<GitCommitItem>,
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
              source_modified_at TEXT,
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
            CREATE TABLE IF NOT EXISTS note_chunks (
              chunk_id TEXT PRIMARY KEY,
              note_id TEXT NOT NULL,
              course_id TEXT NOT NULL,
              heading_path TEXT NOT NULL,
              text TEXT NOT NULL,
              ordinal INTEGER NOT NULL,
              content_hash TEXT NOT NULL,
              FOREIGN KEY (note_id) REFERENCES note_records(id) ON DELETE CASCADE,
              FOREIGN KEY (course_id) REFERENCES course_configs(id) ON DELETE CASCADE
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS note_chunks_fts USING fts5(
              chunk_id UNINDEXED,
              note_id UNINDEXED,
              course_id UNINDEXED,
              heading_path,
              text
            );
            CREATE TABLE IF NOT EXISTS formula_briefs (
              formula_id TEXT NOT NULL,
              course_id TEXT NOT NULL,
              source_hash TEXT NOT NULL,
              coach_json TEXT NOT NULL,
              practice_json TEXT NOT NULL,
              derivation_json TEXT NOT NULL,
              generated_at TEXT NOT NULL,
              model TEXT NOT NULL,
              PRIMARY KEY (formula_id, source_hash),
              FOREIGN KEY (course_id) REFERENCES course_configs(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS chat_threads (
              id TEXT PRIMARY KEY,
              scope TEXT NOT NULL,
              course_id TEXT,
              title TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              FOREIGN KEY (course_id) REFERENCES course_configs(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS chat_messages (
              id TEXT PRIMARY KEY,
              thread_id TEXT NOT NULL,
              role TEXT NOT NULL,
              content TEXT NOT NULL,
              used_fallback INTEGER,
              fallback_reason TEXT,
              created_at TEXT NOT NULL,
              FOREIGN KEY (thread_id) REFERENCES chat_threads(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS chat_citations (
              id TEXT PRIMARY KEY,
              message_id TEXT NOT NULL,
              note_id TEXT NOT NULL,
              chunk_id TEXT,
              note_title TEXT NOT NULL,
              relative_path TEXT NOT NULL,
              heading_path TEXT,
              excerpt TEXT NOT NULL,
              course_id TEXT NOT NULL,
              course_name TEXT NOT NULL,
              relevance REAL NOT NULL DEFAULT 0,
              position INTEGER NOT NULL,
              FOREIGN KEY (message_id) REFERENCES chat_messages(id) ON DELETE CASCADE,
              FOREIGN KEY (note_id) REFERENCES note_records(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS exam_source_queue (
              course_id TEXT NOT NULL,
              note_id TEXT NOT NULL,
              queued_at TEXT NOT NULL,
              PRIMARY KEY (course_id, note_id),
              FOREIGN KEY (course_id) REFERENCES course_configs(id) ON DELETE CASCADE,
              FOREIGN KEY (note_id) REFERENCES note_records(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS note_mastery_states (
              note_id TEXT PRIMARY KEY,
              course_id TEXT NOT NULL,
              mastery_state TEXT NOT NULL,
              last_accuracy REAL,
              updated_at TEXT NOT NULL,
              FOREIGN KEY (course_id) REFERENCES course_configs(id) ON DELETE CASCADE,
              FOREIGN KEY (note_id) REFERENCES note_records(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS exams (
              id TEXT PRIMARY KEY,
              course_id TEXT NOT NULL,
              title TEXT NOT NULL,
              preset TEXT NOT NULL,
              status TEXT NOT NULL,
              difficulty TEXT NOT NULL,
              question_count INTEGER NOT NULL,
              source_note_count INTEGER NOT NULL,
              multiple_choice_count INTEGER NOT NULL,
              short_answer_count INTEGER NOT NULL,
              time_limit_minutes INTEGER NOT NULL,
              source_note_ids_json TEXT NOT NULL DEFAULT '[]',
              instructions TEXT NOT NULL DEFAULT '',
              summary TEXT NOT NULL DEFAULT '',
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              generated_at TEXT,
              last_error TEXT,
              model TEXT,
              FOREIGN KEY (course_id) REFERENCES course_configs(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS exam_questions (
              id TEXT PRIMARY KEY,
              exam_id TEXT NOT NULL,
              position INTEGER NOT NULL,
              question_type TEXT NOT NULL,
              prompt TEXT NOT NULL,
              options_json TEXT NOT NULL,
              correct_answer TEXT NOT NULL,
              explanation TEXT NOT NULL,
              source_note_id TEXT NOT NULL,
              source_note_title TEXT NOT NULL,
              created_at TEXT NOT NULL,
              FOREIGN KEY (exam_id) REFERENCES exams(id) ON DELETE CASCADE,
              FOREIGN KEY (source_note_id) REFERENCES note_records(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS exam_attempts (
              id TEXT PRIMARY KEY,
              exam_id TEXT NOT NULL,
              course_id TEXT NOT NULL,
              submitted_at TEXT NOT NULL,
              score_percent REAL NOT NULL,
              correct_count INTEGER NOT NULL,
              partial_count INTEGER NOT NULL,
              incorrect_count INTEGER NOT NULL,
              overall_feedback TEXT NOT NULL,
              FOREIGN KEY (exam_id) REFERENCES exams(id) ON DELETE CASCADE,
              FOREIGN KEY (course_id) REFERENCES course_configs(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS exam_attempt_question_results (
              id TEXT PRIMARY KEY,
              attempt_id TEXT NOT NULL,
              question_id TEXT NOT NULL,
              position INTEGER NOT NULL,
              question_type TEXT NOT NULL,
              prompt TEXT NOT NULL,
              options_json TEXT NOT NULL,
              source_note_id TEXT NOT NULL,
              source_note_title TEXT NOT NULL,
              user_answer_json TEXT NOT NULL,
              verdict TEXT NOT NULL,
              is_correct INTEGER NOT NULL,
              expected_answer TEXT NOT NULL,
              explanation TEXT NOT NULL,
              feedback TEXT NOT NULL,
              FOREIGN KEY (attempt_id) REFERENCES exam_attempts(id) ON DELETE CASCADE,
              FOREIGN KEY (question_id) REFERENCES exam_questions(id) ON DELETE CASCADE,
              FOREIGN KEY (source_note_id) REFERENCES note_records(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS stats_snapshots (
              id TEXT PRIMARY KEY,
              scope TEXT NOT NULL,
              course_id TEXT,
              captured_at TEXT NOT NULL,
              note_count INTEGER NOT NULL,
              total_concepts INTEGER NOT NULL,
              covered_concepts INTEGER NOT NULL,
              coverage_percentage REAL NOT NULL,
              edge_count INTEGER NOT NULL,
              strong_links INTEGER NOT NULL,
              inferred_links INTEGER NOT NULL,
              isolated_notes INTEGER NOT NULL,
              weak_note_count INTEGER NOT NULL,
              formula_count INTEGER NOT NULL,
              notes_with_formulas INTEGER NOT NULL DEFAULT 0,
              average_note_strength REAL NOT NULL DEFAULT 0,
              flashcard_set_count INTEGER NOT NULL DEFAULT 0,
              flashcard_total_cards INTEGER NOT NULL DEFAULT 0,
              revision_run_count INTEGER NOT NULL DEFAULT 0,
              latest_revision_item_count INTEGER NOT NULL DEFAULT 0,
              ai_ready_notes INTEGER NOT NULL DEFAULT 0,
              ai_pending_notes INTEGER NOT NULL DEFAULT 0,
              ai_failed_notes INTEGER NOT NULL DEFAULT 0,
              ai_stale_notes INTEGER NOT NULL DEFAULT 0,
              ai_missing_notes INTEGER NOT NULL DEFAULT 0,
              exam_attempt_count INTEGER NOT NULL,
              latest_exam_score REAL,
              average_exam_score REAL,
              FOREIGN KEY (course_id) REFERENCES course_configs(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_notes_course ON note_records(course_id);
            CREATE INDEX IF NOT EXISTS idx_concepts_course ON concept_records(course_id);
            CREATE INDEX IF NOT EXISTS idx_formulas_course ON formula_records(course_id);
            CREATE INDEX IF NOT EXISTS idx_edges_course ON dependency_edges(course_id);
            CREATE INDEX IF NOT EXISTS idx_weak_course ON weak_link_suggestions(course_id);
            CREATE INDEX IF NOT EXISTS idx_ai_note_states_course ON ai_note_states(course_id);
            CREATE INDEX IF NOT EXISTS idx_note_chunks_note ON note_chunks(note_id, ordinal);
            CREATE INDEX IF NOT EXISTS idx_note_chunks_course ON note_chunks(course_id, note_id, ordinal);
            CREATE INDEX IF NOT EXISTS idx_formula_briefs_course ON formula_briefs(course_id, formula_id, generated_at DESC);
            CREATE INDEX IF NOT EXISTS idx_chat_threads_scope ON chat_threads(scope, updated_at DESC);
            CREATE INDEX IF NOT EXISTS idx_chat_messages_thread ON chat_messages(thread_id, created_at ASC);
            CREATE INDEX IF NOT EXISTS idx_chat_citations_message ON chat_citations(message_id, position ASC);
            CREATE INDEX IF NOT EXISTS idx_exam_source_queue_course ON exam_source_queue(course_id, queued_at DESC);
            CREATE INDEX IF NOT EXISTS idx_note_mastery_course ON note_mastery_states(course_id, mastery_state);
            CREATE INDEX IF NOT EXISTS idx_exams_course_status ON exams(course_id, status, created_at DESC);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_exam_questions_order ON exam_questions(exam_id, position);
            CREATE INDEX IF NOT EXISTS idx_exam_attempts_exam ON exam_attempts(exam_id, submitted_at DESC);
            CREATE INDEX IF NOT EXISTS idx_exam_attempts_course ON exam_attempts(course_id, submitted_at DESC);
            CREATE INDEX IF NOT EXISTS idx_exam_attempt_results_attempt ON exam_attempt_question_results(attempt_id, position);
            CREATE INDEX IF NOT EXISTS idx_stats_snapshots_scope ON stats_snapshots(scope, course_id, captured_at ASC);
            "#,
        )?;
        self.ensure_column_exists("note_records", "source_modified_at", "TEXT")?;
        self.ensure_column_exists(
            "stats_snapshots",
            "notes_with_formulas",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        self.ensure_column_exists(
            "stats_snapshots",
            "average_note_strength",
            "REAL NOT NULL DEFAULT 0",
        )?;
        self.ensure_column_exists(
            "stats_snapshots",
            "flashcard_set_count",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        self.ensure_column_exists(
            "stats_snapshots",
            "flashcard_total_cards",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        self.ensure_column_exists(
            "stats_snapshots",
            "revision_run_count",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        self.ensure_column_exists(
            "stats_snapshots",
            "latest_revision_item_count",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        self.ensure_column_exists(
            "stats_snapshots",
            "ai_ready_notes",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        self.ensure_column_exists(
            "stats_snapshots",
            "ai_pending_notes",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        self.ensure_column_exists(
            "stats_snapshots",
            "ai_failed_notes",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        self.ensure_column_exists(
            "stats_snapshots",
            "ai_stale_notes",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        self.ensure_column_exists(
            "stats_snapshots",
            "ai_missing_notes",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        self.ensure_column_exists("chat_citations", "course_id", "TEXT NOT NULL DEFAULT ''")?;
        self.ensure_column_exists("chat_citations", "course_name", "TEXT NOT NULL DEFAULT ''")?;
        self.ensure_column_exists("chat_citations", "relevance", "REAL NOT NULL DEFAULT 0")?;
        Ok(())
    }

    fn ensure_column_exists(&self, table: &str, column: &str, definition: &str) -> Result<()> {
        let pragma = format!("PRAGMA table_info({table})");
        let mut statement = self.conn.prepare(&pragma)?;
        let columns = statement
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if columns.iter().any(|existing| existing == column) {
            return Ok(());
        }

        let alter = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
        self.conn.execute(&alter, [])?;
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
        self.conn.execute("DELETE FROM chat_threads", [])?;
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
        self.conn.execute("DELETE FROM chat_threads", [])?;
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
            exams: self.get_exam_workspace_summary(&course.id)?,
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

    pub fn get_statistics(
        &self,
        scope: StatisticsScope,
        course_id: Option<String>,
    ) -> Result<Option<StatisticsResponse>> {
        let vault = self
            .get_vault_config()?
            .ok_or_else(|| anyhow!("no vault connected"))?;
        let courses = self.list_courses()?;
        if courses.is_empty() {
            return Ok(None);
        }

        let (selected_course, course_bundles) = match scope {
            StatisticsScope::Course => {
                let selected_id = course_id
                    .or_else(|| self.get_selected_course_id().ok().flatten())
                    .and_then(|id| {
                        courses
                            .iter()
                            .find(|course| course.id == id)
                            .map(|course| course.id.clone())
                    })
                    .unwrap_or_else(|| courses[0].id.clone());
                let course = courses
                    .iter()
                    .find(|entry| entry.id == selected_id)
                    .ok_or_else(|| anyhow!("selected course does not exist"))?;

                self.set_selected_course(Some(&selected_id))?;
                (
                    Some(course.clone()),
                    vec![self.build_course_statistics_bundle(course)?],
                )
            }
            StatisticsScope::Vault => {
                let bundles = courses
                    .iter()
                    .map(|course| self.build_course_statistics_bundle(course))
                    .collect::<Result<Vec<_>>>()?;
                (None, bundles)
            }
        };

        let history = self.list_statistics_history(scope, selected_course.as_ref().map(|course| course.id.as_str()))?;
        let exam_history =
            self.list_statistics_exam_points(selected_course.as_ref().map(|course| course.id.as_str()))?;
        let activity_buckets =
            self.build_activity_buckets(selected_course.as_ref().map(|course| course.id.as_str()))?;
        let course_rows = if scope == StatisticsScope::Vault {
            self.build_course_statistics_rows(&courses)?
        } else {
            Vec::new()
        };
        let overview_summary = if scope == StatisticsScope::Vault {
            self.build_vault_statistics_overview(&courses)?
        } else {
            course_bundles[0].overview.clone()
        };

        let top_concepts = if scope == StatisticsScope::Vault {
            let raw = courses
                .iter()
                .flat_map(|course| self.list_concepts(&course.id).unwrap_or_default())
                .collect::<Vec<_>>();
            build_top_concepts(&raw)
        } else {
            course_bundles[0].concepts.clone()
        };
        let top_formulas = if scope == StatisticsScope::Vault {
            let raw = courses
                .iter()
                .flat_map(|course| self.list_formulas(&course.id).unwrap_or_default())
                .collect::<Vec<_>>();
            build_top_formulas(&raw)
        } else {
            course_bundles[0].formulas.clone()
        };
        let note_rows = course_bundles
            .iter()
            .flat_map(|bundle| bundle.notes.clone())
            .collect::<Vec<_>>();
        let recent_notes = self.build_recent_notes(&note_rows);
        let attempt_history = build_attempt_history(&exam_history);
        let verdict_mix =
            self.build_exam_verdict_mix(selected_course.as_ref().map(|course| course.id.as_str()))?;
        let mastery_distribution =
            self.build_mastery_distribution(selected_course.as_ref().map(|course| course.id.as_str()))?;
        let latest_flashcards = aggregate_flashcard_summary(&course_bundles);
        let latest_revision = aggregate_revision_summary(&course_bundles);
        let git_result =
            self.try_build_git_analytics(&vault.vault_path, &courses, scope, selected_course.as_ref())?;
        let (git_available, git_error, git_data) = match git_result {
            Ok(value) => (value.is_some(), None, value),
            Err(error) => (false, Some(error), None),
        };
        let highlights = build_overview_highlights(&course_rows, git_data.as_ref());
        let overview = StatisticsOverviewSection {
            summary: overview_summary.clone(),
            history: history.clone(),
            course_rows: course_rows.clone(),
            highlights,
        };
        let knowledge = StatisticsKnowledgeSection {
            summary: StatisticsKnowledgeSummary {
                total_concepts: overview_summary.total_concepts,
                covered_concepts: overview_summary.covered_concepts,
                coverage_percentage: overview_summary.coverage_percentage,
                formula_count: overview_summary.formula_count,
                notes_with_formulas: overview_summary.notes_with_formulas,
            },
            history: history.clone(),
            top_concepts,
            top_formulas,
            formula_density_buckets: build_formula_density_buckets(&note_rows),
            course_rows: course_rows.clone(),
        };
        let notes = StatisticsNotesSection {
            summary: StatisticsNotesSummary {
                note_count: overview_summary.note_count,
                average_note_strength: overview_summary.average_note_strength,
                weak_note_count: overview_summary.weak_note_count,
                isolated_notes: overview_summary.isolated_notes,
                stale_note_count: note_rows.iter().filter(|note| is_stale_modified_at(note.modified_at.as_deref())).count(),
            },
            history: history.clone(),
            strength_buckets: build_strength_buckets(&note_rows),
            activity_buckets: activity_buckets.clone(),
            weakest_notes: sort_note_rows_by_strength(&note_rows, false),
            most_connected_notes: sort_note_rows_by_links(&note_rows),
            stalest_notes: sort_note_rows_by_modified_at(&note_rows),
            most_changed_notes: git_data
                .as_ref()
                .map(|git| git.top_notes.clone())
                .unwrap_or_default(),
        };
        let exams = StatisticsExamsSection {
            summary: StatisticsExamsSummary {
                attempt_count: overview_summary.exam_attempt_count,
                latest_score: overview_summary.latest_exam_score,
                average_score: overview_summary.average_exam_score,
                review_count: self.count_note_mastery_state(
                    selected_course.as_ref().map(|course| course.id.as_str()),
                    NoteMasteryState::Review,
                )?,
                mastered_count: self.count_note_mastery_state(
                    selected_course.as_ref().map(|course| course.id.as_str()),
                    NoteMasteryState::Mastered,
                )?,
            },
            score_history: exam_history.clone(),
            attempt_history,
            verdict_mix,
            mastery_distribution,
            recent_exams: recent_exam_points(&exam_history),
            weakest_attempts: weakest_exam_points(&exam_history),
        };
        let ai = StatisticsAiSection {
            summary: StatisticsAiSummary {
                ready_notes: overview_summary.ai_ready_notes,
                pending_notes: overview_summary.ai_pending_notes,
                failed_notes: overview_summary.ai_failed_notes,
                stale_notes: overview_summary.ai_stale_notes,
                missing_notes: overview_summary.ai_missing_notes,
            },
            history: history.clone(),
            status_breakdown: vec![
                StatisticsCountBucket {
                    label: "Ready".to_string(),
                    count: overview_summary.ai_ready_notes,
                },
                StatisticsCountBucket {
                    label: "Queued".to_string(),
                    count: overview_summary.ai_pending_notes,
                },
                StatisticsCountBucket {
                    label: "Failed".to_string(),
                    count: overview_summary.ai_failed_notes,
                },
                StatisticsCountBucket {
                    label: "Stale".to_string(),
                    count: overview_summary.ai_stale_notes,
                },
                StatisticsCountBucket {
                    label: "Missing".to_string(),
                    count: overview_summary.ai_missing_notes,
                },
            ],
            failed_notes: note_rows
                .iter()
                .filter(|note| note.ai_status == "failed")
                .take(8)
                .cloned()
                .collect(),
            stale_notes: note_rows
                .iter()
                .filter(|note| note.ai_status == "stale" || note.ai_status == "missing")
                .take(8)
                .cloned()
                .collect(),
            course_rows: course_rows.clone(),
        };
        let outputs = StatisticsOutputsSection {
            summary: StatisticsOutputsSummary {
                flashcard_set_count: overview_summary.flashcard_set_count,
                flashcard_total_cards: overview_summary.flashcard_total_cards,
                revision_run_count: overview_summary.revision_run_count,
                latest_revision_item_count: overview_summary.latest_revision_item_count,
                latest_flashcard_export: latest_flashcards.export_path.clone(),
                latest_revision_note: latest_revision.note_path.clone(),
            },
            history: history.clone(),
            output_mix: vec![
                StatisticsCountBucket {
                    label: "Flashcard sets".to_string(),
                    count: overview_summary.flashcard_set_count,
                },
                StatisticsCountBucket {
                    label: "Flashcard cards".to_string(),
                    count: overview_summary.flashcard_total_cards,
                },
                StatisticsCountBucket {
                    label: "Revision runs".to_string(),
                    count: overview_summary.revision_run_count,
                },
            ],
            latest_flashcards,
            latest_revision,
            course_rows: course_rows.clone(),
        };
        let vault_activity = StatisticsVaultActivitySection {
            summary: build_vault_activity_summary(&activity_buckets, &note_rows),
            activity_buckets,
            recent_notes,
            course_activity: if scope == StatisticsScope::Vault {
                course_rows.clone()
            } else {
                Vec::new()
            },
            git_timeline: git_data
                .as_ref()
                .map(|git| git.commit_timeline.clone())
                .unwrap_or_default(),
            git_course_activity: git_data
                .as_ref()
                .map(|git| git.course_activity.clone())
                .unwrap_or_default(),
            git_top_notes: git_data
                .as_ref()
                .map(|git| git.top_notes.clone())
                .unwrap_or_default(),
            recent_commits: git_data
                .as_ref()
                .map(|git| git.recent_commits.clone())
                .unwrap_or_default(),
        };

        Ok(Some(StatisticsResponse {
            scope,
            generated_at: now_string(),
            course_id: selected_course.as_ref().map(|course| course.id.clone()),
            course_name: selected_course.as_ref().map(|course| course.name.clone()),
            git_available,
            git_error,
            overview,
            knowledge,
            notes,
            exams,
            ai,
            outputs,
            vault_activity,
            git: git_data.map(|git| StatisticsGitSection {
                summary: git.summary,
                commit_timeline: git.commit_timeline,
                churn_timeline: git.churn_timeline,
                course_activity: git.course_activity,
                top_notes: git.top_notes,
                recent_commits: git.recent_commits,
            }),
        }))
    }

    fn build_course_statistics_bundle(&self, course: &StoredCourse) -> Result<CourseStatisticsBundle> {
        let notes = self.list_notes(&course.id)?;
        let concepts = self.list_concepts(&course.id)?;
        let formulas = self.list_formulas(&course.id)?;
        let edges = self.list_edges(&course.id)?;
        let flashcard_sets = self.list_flashcard_sets(&course.id)?;
        let revision_runs = self.list_revision_runs(&course.id)?;
        let ai_states = self.list_ai_note_states(&course.id)?;
        let flashcards = build_flashcard_summary(&flashcard_sets);
        let revision = build_revision_summary(&revision_runs);
        let note_summaries = build_note_summaries(&notes, &edges, &concepts, &formulas, &ai_states);
        let note_lookup = notes
            .iter()
            .map(|note| (note.id.clone(), note))
            .collect::<HashMap<_, _>>();
        let note_rows = note_summaries
            .into_iter()
            .map(|summary| StatisticsNoteRow {
                note_id: summary.id.clone(),
                title: summary.title.clone(),
                relative_path: summary.relative_path.clone(),
                course_id: Some(course.id.clone()),
                course_name: Some(course.name.clone()),
                ai_status: summary.ai_status.clone(),
                strength: summary.strength,
                link_count: summary.link_count,
                concept_count: summary.concept_count,
                formula_count: summary.formula_count,
                modified_at: note_lookup
                    .get(&summary.id)
                    .and_then(|note| note.source_modified_at.clone()),
            })
            .collect::<Vec<_>>();

        Ok(CourseStatisticsBundle {
            overview: self.build_course_statistics_overview(&course.id)?,
            flashcards,
            revision,
            notes: note_rows,
            concepts: build_top_concepts(&concepts),
            formulas: build_top_formulas(&formulas),
        })
    }

    fn build_recent_notes(&self, note_rows: &[StatisticsNoteRow]) -> Vec<StatisticsNoteRow> {
        let mut rows = note_rows.to_vec();
        rows.sort_by(|left, right| right.modified_at.cmp(&left.modified_at));
        rows.into_iter().take(10).collect()
    }

    fn build_exam_verdict_mix(
        &self,
        course_id: Option<&str>,
    ) -> Result<Vec<StatisticsCountBucket>> {
        let mut buckets = vec![
            StatisticsCountBucket {
                label: "Correct".to_string(),
                count: 0,
            },
            StatisticsCountBucket {
                label: "Partial".to_string(),
                count: 0,
            },
            StatisticsCountBucket {
                label: "Incorrect".to_string(),
                count: 0,
            },
        ];

        let rows = match course_id {
            Some(course_id) => {
                let mut statement = self.conn.prepare(
                    r#"
                    SELECT exam_attempt_question_results.verdict, COUNT(*)
                    FROM exam_attempt_question_results
                    INNER JOIN exam_attempts ON exam_attempts.id = exam_attempt_question_results.attempt_id
                    WHERE exam_attempts.course_id = ?1
                    GROUP BY exam_attempt_question_results.verdict
                    "#,
                )?;
                let rows = statement
                    .query_map(params![course_id], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
                    })?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                rows
            }
            None => {
                let mut statement = self.conn.prepare(
                    r#"
                    SELECT verdict, COUNT(*)
                    FROM exam_attempt_question_results
                    GROUP BY verdict
                    "#,
                )?;
                let rows = statement
                    .query_map([], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
                    })?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                rows
            }
        };

        for (label, count) in rows {
            if let Some(bucket) = buckets
                .iter_mut()
                .find(|bucket| bucket.label.eq_ignore_ascii_case(&label))
            {
                bucket.count = count;
            }
        }
        Ok(buckets)
    }

    fn build_mastery_distribution(
        &self,
        course_id: Option<&str>,
    ) -> Result<Vec<StatisticsCountBucket>> {
        Ok(vec![
            StatisticsCountBucket {
                label: "Active".to_string(),
                count: self.count_note_mastery_state(course_id, NoteMasteryState::Active)?,
            },
            StatisticsCountBucket {
                label: "Review".to_string(),
                count: self.count_note_mastery_state(course_id, NoteMasteryState::Review)?,
            },
            StatisticsCountBucket {
                label: "Mastered".to_string(),
                count: self.count_note_mastery_state(course_id, NoteMasteryState::Mastered)?,
            },
        ])
    }

    fn count_note_mastery_state(
        &self,
        course_id: Option<&str>,
        state: NoteMasteryState,
    ) -> Result<usize> {
        match course_id {
            Some(course_id) => self.conn.query_row(
                "SELECT COUNT(*) FROM note_mastery_states WHERE course_id = ?1 AND mastery_state = ?2",
                params![course_id, note_mastery_state_to_str(state)],
                |row| row.get::<_, i64>(0).map(|value| value as usize),
            ),
            None => self.conn.query_row(
                "SELECT COUNT(*) FROM note_mastery_states WHERE mastery_state = ?1",
                params![note_mastery_state_to_str(state)],
                |row| row.get::<_, i64>(0).map(|value| value as usize),
            ),
        }
        .map_err(Into::into)
    }

    fn try_build_git_analytics(
        &self,
        vault_path: &str,
        courses: &[StoredCourse],
        scope: StatisticsScope,
        selected_course: Option<&StoredCourse>,
    ) -> Result<std::result::Result<Option<GitAnalytics>, String>> {
        match self.build_git_analytics(vault_path, courses, scope, selected_course) {
            Ok(value) => Ok(Ok(value)),
            Err(error) => Ok(Err(error.to_string())),
        }
    }

    fn build_git_analytics(
        &self,
        vault_path: &str,
        courses: &[StoredCourse],
        scope: StatisticsScope,
        selected_course: Option<&StoredCourse>,
    ) -> Result<Option<GitAnalytics>> {
        let repo_root = match self.git_repo_root(vault_path)? {
            Some(root) => root,
            None => return Ok(None),
        };
        let commits = self.list_git_markdown_commits(&repo_root)?;
        if commits.is_empty() {
            return Ok(None);
        }

        let note_map = self.build_git_note_lookup(courses)?;
        let filtered = commits
            .into_iter()
            .filter_map(|commit| {
                let paths = commit
                    .paths
                    .into_iter()
                    .filter(|path| path.ends_with(".md"))
                    .filter(|path| match scope {
                        StatisticsScope::Course => selected_course
                            .map(|course| is_path_within_course(path, &course.folder))
                            .unwrap_or(false),
                        StatisticsScope::Vault => true,
                    })
                    .collect::<Vec<_>>();
                if paths.is_empty() {
                    None
                } else {
                    Some(GitCommitRecord { paths, ..commit })
                }
            })
            .collect::<Vec<_>>();
        if filtered.is_empty() {
            return Ok(None);
        }

        let commit_timeline = build_git_timeline(&filtered, false);
        let churn_timeline = build_git_timeline(&filtered, true);
        let course_activity = build_git_course_activity_rows(&filtered, courses);
        let top_notes = build_git_note_rows(&filtered, &note_map);
        let recent_commits = filtered
            .iter()
            .rev()
            .take(8)
            .map(|commit| GitCommitItem {
                sha: commit.sha.clone(),
                summary: commit.summary.clone(),
                author_name: commit.author_name.clone(),
                committed_at: commit.committed_at.clone(),
                changed_notes: commit.paths.len(),
            })
            .collect::<Vec<_>>();
        let recent_threshold = Utc::now() - chrono::Duration::days(30);
        let recent_commit_count = filtered
            .iter()
            .filter(|commit| {
                DateTime::parse_from_rfc3339(&commit.committed_at)
                    .map(|timestamp| timestamp.with_timezone(&Utc) >= recent_threshold)
                    .unwrap_or(false)
            })
            .count();
        let active_days_30 = filtered
            .iter()
            .filter_map(|commit| {
                DateTime::parse_from_rfc3339(&commit.committed_at)
                    .ok()
                    .map(|timestamp| timestamp.with_timezone(&Utc))
            })
            .filter(|timestamp| *timestamp >= recent_threshold)
            .map(|timestamp| timestamp.date_naive())
            .collect::<HashSet<_>>()
            .len();
        let summary = GitSummary {
            repo_root: repo_root.to_string_lossy().to_string(),
            total_markdown_commits: filtered.len(),
            total_markdown_file_changes: filtered.iter().map(|commit| commit.paths.len()).sum(),
            last_commit_at: filtered.last().map(|commit| commit.committed_at.clone()),
            recent_commit_count,
            active_days_30,
        };

        Ok(Some(GitAnalytics {
            summary,
            commit_timeline,
            churn_timeline,
            course_activity,
            top_notes,
            recent_commits,
        }))
    }

    fn git_repo_root(&self, vault_path: &str) -> Result<Option<PathBuf>> {
        let output = Command::new("git")
            .args(["-C", vault_path, "rev-parse", "--show-toplevel"])
            .output();
        let output = match output {
            Ok(output) => output,
            Err(error) => return Err(anyhow!("failed to run git: {error}")),
        };
        if !output.status.success() {
            return Ok(None);
        }
        let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if root.is_empty() {
            return Ok(None);
        }
        Ok(Some(PathBuf::from(root)))
    }

    fn list_git_markdown_commits(&self, repo_root: &Path) -> Result<Vec<GitCommitRecord>> {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo_root)
            .args([
                "log",
                "--date=iso-strict",
                "--name-only",
                "--pretty=format:__COD__%n%H%n%aI%n%an%n%s",
                "--",
                ".",
            ])
            .output()
            .context("failed to read git history")?;
        if !output.status.success() {
            bail!(
                "git log failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let mut records = Vec::new();
        let mut lines = text.lines().peekable();
        while let Some(line) = lines.next() {
            if line != "__COD__" {
                continue;
            }
            let sha = lines.next().unwrap_or_default().to_string();
            let committed_at = lines.next().unwrap_or_default().to_string();
            let author_name = lines.next().unwrap_or_default().to_string();
            let summary = lines.next().unwrap_or_default().to_string();
            let mut paths = Vec::new();
            while let Some(next) = lines.peek() {
                if *next == "__COD__" {
                    break;
                }
                let value = normalize_relative_path(lines.next().unwrap_or_default());
                if value.ends_with(".md") {
                    paths.push(value);
                }
            }
            records.push(GitCommitRecord {
                sha,
                committed_at,
                author_name,
                summary,
                paths,
            });
        }
        Ok(records)
    }

    fn build_git_note_lookup(
        &self,
        courses: &[StoredCourse],
    ) -> Result<HashMap<String, (String, String, String, String)>> {
        let mut map = HashMap::new();
        for course in courses {
            for note in self.list_notes(&course.id)? {
                map.insert(
                    normalize_relative_path(&note.relative_path),
                    (
                        note.id,
                        note.title,
                        course.id.clone(),
                        course.name.clone(),
                    ),
                );
            }
        }
        Ok(map)
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
    pub fn get_exam_workspace(
        &self,
        course_id: Option<String>,
    ) -> Result<Option<ExamWorkspaceSnapshot>> {
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
        Ok(Some(self.build_exam_workspace(&selected)?))
    }

    pub fn add_exam_source_notes(
        &self,
        course_id: &str,
        note_ids: &[String],
    ) -> Result<ExamWorkspaceSnapshot> {
        self.find_course(course_id)?
            .ok_or_else(|| anyhow!("course not found"))?;

        let valid_note_ids = self
            .list_notes(course_id)?
            .into_iter()
            .map(|note| note.id)
            .collect::<HashSet<_>>();
        let timestamp = now_string();

        for note_id in note_ids {
            if !valid_note_ids.contains(note_id) {
                continue;
            }
            self.conn.execute(
                r#"
                INSERT INTO exam_source_queue (course_id, note_id, queued_at)
                VALUES (?1, ?2, ?3)
                ON CONFLICT(course_id, note_id) DO UPDATE SET
                  queued_at = excluded.queued_at
                "#,
                params![course_id, note_id, timestamp],
            )?;
        }

        self.build_exam_workspace(course_id)
    }

    pub fn remove_exam_source_notes(
        &self,
        course_id: &str,
        note_ids: &[String],
    ) -> Result<ExamWorkspaceSnapshot> {
        self.find_course(course_id)?
            .ok_or_else(|| anyhow!("course not found"))?;

        for note_id in note_ids {
            self.conn.execute(
                "DELETE FROM exam_source_queue WHERE course_id = ?1 AND note_id = ?2",
                params![course_id, note_id],
            )?;
        }

        self.build_exam_workspace(course_id)
    }

    pub fn clear_exam_source_queue(&self, course_id: &str) -> Result<ExamWorkspaceSnapshot> {
        self.find_course(course_id)?
            .ok_or_else(|| anyhow!("course not found"))?;
        self.conn.execute(
            "DELETE FROM exam_source_queue WHERE course_id = ?1",
            params![course_id],
        )?;
        self.build_exam_workspace(course_id)
    }

    pub fn queue_exams(&self, request: ExamBuilderInput) -> Result<ExamWorkspaceSnapshot> {
        self.ai_settings_for_runtime()?.ok_or_else(|| {
            anyhow!("enable AI in Setup and save reachable provider settings first")
        })?;
        let course = self
            .find_course(&request.course_id)?
            .ok_or_else(|| anyhow!("course not found"))?;
        let source_note_ids = self.list_exam_source_queue_ids(&course.id)?;
        if source_note_ids.is_empty() {
            bail!("add one or more notes to the exam source queue first");
        }

        let request = normalize_exam_builder_input(request)?;
        let existing_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM exams WHERE course_id = ?1",
            params![&course.id],
            |row| row.get::<_, i64>(0).map(|value| value as usize),
        )?;

        for offset in 0..request.generate_count {
            let created_at = now_string();
            let title = request
                .title
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| {
                    format!(
                        "{} Exam {}",
                        exam_preset_label(request.preset),
                        existing_count + offset + 1
                    )
                });
            let exam_id = Uuid::new_v4().to_string();
            self.conn.execute(
                r#"
                INSERT INTO exams (
                  id, course_id, title, preset, status, difficulty, question_count, source_note_count,
                  multiple_choice_count, short_answer_count, time_limit_minutes, source_note_ids_json,
                  instructions, summary, created_at, updated_at, generated_at, last_error, model
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, '', '', ?13, ?13, NULL, NULL, NULL)
                "#,
                params![
                    exam_id,
                    course.id,
                    title,
                    exam_preset_to_str(request.preset),
                    exam_status_to_str(ExamStatus::Queued),
                    exam_difficulty_to_str(request.difficulty),
                    (request.multiple_choice_count + request.short_answer_count) as i64,
                    source_note_ids.len() as i64,
                    request.multiple_choice_count as i64,
                    request.short_answer_count as i64,
                    request.time_limit_minutes as i64,
                    to_json(&source_note_ids),
                    created_at,
                ],
            )?;
        }

        self.build_exam_workspace(&course.id)
    }

    pub fn has_pending_exam_jobs(&self, course_id: &str) -> Result<bool> {
        let count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM exams WHERE course_id = ?1 AND status IN ('queued', 'generating')",
            params![course_id],
            |row| row.get::<_, i64>(0).map(|value| value as usize),
        )?;
        Ok(count > 0)
    }

    pub fn run_exam_generation_queue(&self, course_id: &str) -> Result<()> {
        let settings = self.ai_settings_for_runtime()?.ok_or_else(|| {
            anyhow!("enable AI in Setup and save reachable provider settings first")
        })?;
        let course = self
            .find_course(course_id)?
            .ok_or_else(|| anyhow!("course not found"))?;

        while let Some(job) = self.claim_next_exam_generation_job(course_id)? {
            let input = build_exam_builder_input_from_record(&job.exam);
            let result = ai::generate_exam(&settings, &course.name, &input, &job.notes);

            match result {
                Ok(payload) => {
                    self.store_generated_exam(
                        &job.exam.id,
                        &payload.instructions,
                        &payload.summary,
                        &payload.questions,
                        &settings.model,
                    )?;
                }
                Err(error) => {
                    self.fail_exam_generation(&job.exam.id, &error.to_string())?;
                }
            }
        }

        Ok(())
    }

    pub fn mark_exam_generation_failed(&self, course_id: &str, message: &str) -> Result<()> {
        if let Some(exam) = self.find_generating_exam(course_id)? {
            self.fail_exam_generation(&exam.id, message)?;
        }
        Ok(())
    }

    pub fn get_exam_details(&self, exam_id: &str) -> Result<ExamDetails> {
        let exam = self
            .find_exam_record(exam_id)?
            .ok_or_else(|| anyhow!("exam not found"))?;
        let questions = self.load_exam_questions(exam_id, true)?;
        let source_notes =
            self.load_exam_source_notes_snapshot(&exam.course_id, &exam.source_note_ids)?;

        Ok(build_exam_details(exam, questions, source_notes))
    }

    pub fn submit_exam_attempt(&self, request: ExamSubmissionRequest) -> Result<ExamAttemptResult> {
        let settings = self.ai_settings_for_runtime()?.ok_or_else(|| {
            anyhow!("enable AI in Setup and save reachable provider settings first")
        })?;
        let exam = self
            .find_exam_record(&request.exam_id)?
            .ok_or_else(|| anyhow!("exam not found"))?;
        if exam.status != ExamStatus::Ready {
            bail!("exam is not ready yet");
        }

        let course = self
            .find_course(&exam.course_id)?
            .ok_or_else(|| anyhow!("course not found"))?;
        let questions = self.load_exam_questions(&exam.id, false)?;
        if questions.is_empty() {
            bail!("exam has no generated questions");
        }

        let answers = request
            .answers
            .into_iter()
            .map(|entry| (entry.question_id, entry.answer))
            .collect::<HashMap<_, _>>();
        let grading_input = questions
            .iter()
            .map(|question| ai::ExamGradingQuestionInput {
                question_id: question.id.clone(),
                index: question.index,
                question_type: question.question_type,
                prompt: question.prompt.clone(),
                options: question.options.clone(),
                source_note_id: question.source_note_id.clone(),
                source_note_title: question.source_note_title.clone(),
                expected_answer: question.expected_answer.clone().unwrap_or_default(),
                explanation: question.explanation.clone().unwrap_or_default(),
                user_answer: answers
                    .get(&question.id)
                    .cloned()
                    .unwrap_or_else(|| empty_answer_for_question(question.question_type)),
            })
            .collect::<Vec<_>>();
        let grading = ai::grade_exam_attempt(&settings, &course.name, &exam.title, &grading_input)?;
        let grading_by_question = grading
            .results
            .into_iter()
            .map(|result| (result.question_id.clone(), result))
            .collect::<HashMap<_, _>>();

        let mut question_results = Vec::with_capacity(questions.len());
        let mut note_scores = HashMap::<String, (f64, usize)>::new();
        let mut correct_count = 0usize;
        let mut partial_count = 0usize;
        let mut incorrect_count = 0usize;

        for question in questions {
            let grading_result = grading_by_question
                .get(&question.id)
                .ok_or_else(|| anyhow!("grading response omitted question {}", question.id))?;
            let user_answer = answers
                .get(&question.id)
                .cloned()
                .unwrap_or_else(|| empty_answer_for_question(question.question_type));
            let is_correct = grading_result.verdict == ExamVerdict::Correct;

            match grading_result.verdict {
                ExamVerdict::Correct => correct_count += 1,
                ExamVerdict::Partial => partial_count += 1,
                ExamVerdict::Incorrect => incorrect_count += 1,
            }

            let entry = note_scores
                .entry(question.source_note_id.clone())
                .or_insert((0.0, 0usize));
            entry.0 += match grading_result.verdict {
                ExamVerdict::Correct => 1.0,
                ExamVerdict::Partial => 0.5,
                ExamVerdict::Incorrect => 0.0,
            };
            entry.1 += 1;

            question_results.push(ExamQuestionResult {
                question_id: question.id.clone(),
                index: question.index,
                question_type: question.question_type,
                prompt: question.prompt.clone(),
                options: question.options.clone(),
                source_note_id: question.source_note_id.clone(),
                source_note_title: question.source_note_title.clone(),
                user_answer,
                verdict: grading_result.verdict,
                is_correct,
                expected_answer: grading_result.expected_answer.clone(),
                explanation: grading_result.explanation.clone(),
                feedback: grading_result.feedback.clone(),
            });
        }

        let total_questions = question_results.len().max(1);
        let score_percent = round_percentage(
            ((correct_count as f64 + partial_count as f64 * 0.5) / total_questions as f64) * 100.0,
        );
        let submitted_at = now_string();
        let attempt_id = Uuid::new_v4().to_string();

        self.conn.execute(
            r#"
            INSERT INTO exam_attempts (
              id, exam_id, course_id, submitted_at, score_percent, correct_count,
              partial_count, incorrect_count, overall_feedback
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                &attempt_id,
                &exam.id,
                &exam.course_id,
                &submitted_at,
                score_percent,
                correct_count as i64,
                partial_count as i64,
                incorrect_count as i64,
                &grading.overall_feedback,
            ],
        )?;

        for result in &question_results {
            self.conn.execute(
                r#"
                INSERT INTO exam_attempt_question_results (
                  id, attempt_id, question_id, position, question_type, prompt, options_json,
                  source_note_id, source_note_title, user_answer_json, verdict, is_correct,
                  expected_answer, explanation, feedback
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                "#,
                params![
                    Uuid::new_v4().to_string(),
                    &attempt_id,
                    result.question_id,
                    result.index as i64,
                    exam_question_type_to_str(result.question_type),
                    result.prompt,
                    to_json(&result.options),
                    result.source_note_id,
                    result.source_note_title,
                    to_json(&result.user_answer),
                    exam_verdict_to_str(result.verdict),
                    bool_to_int(result.is_correct),
                    result.expected_answer,
                    result.explanation,
                    result.feedback,
                ],
            )?;
        }

        let note_suggestions = self.build_note_suggestions(&exam.course_id, &note_scores)?;
        let overall_feedback = self.conn.query_row(
            "SELECT overall_feedback FROM exam_attempts WHERE id = ?1",
            params![&attempt_id],
            |row| row.get::<_, String>(0),
        )?;

        Ok(ExamAttemptResult {
            exam_id: exam.id,
            attempt_id,
            submitted_at,
            score_percent,
            correct_count,
            partial_count,
            incorrect_count,
            overall_feedback,
            question_results,
            note_suggestions,
        })
    }

    pub fn apply_exam_review_actions(
        &self,
        request: ApplyExamReviewActionsRequest,
    ) -> Result<ExamWorkspaceSnapshot> {
        let attempt = self
            .find_exam_attempt(&request.attempt_id)?
            .ok_or_else(|| anyhow!("exam attempt not found"))?;

        for action in request.actions {
            let note_course_id = self.note_course_id(&action.note_id)?;
            if note_course_id != attempt.course_id {
                bail!("note does not belong to the exam course");
            }

            let accuracy = self
                .note_mastery_for_note(&action.note_id)?
                .and_then(|row| row.last_accuracy);
            self.upsert_note_mastery(
                &action.note_id,
                &attempt.course_id,
                action.next_state,
                accuracy,
            )?;

            if action.add_to_exam_queue {
                self.conn.execute(
                    r#"
                    INSERT INTO exam_source_queue (course_id, note_id, queued_at)
                    VALUES (?1, ?2, ?3)
                    ON CONFLICT(course_id, note_id) DO UPDATE SET
                      queued_at = excluded.queued_at
                    "#,
                    params![&attempt.course_id, action.note_id, now_string()],
                )?;
            } else {
                self.conn.execute(
                    "DELETE FROM exam_source_queue WHERE course_id = ?1 AND note_id = ?2",
                    params![&attempt.course_id, action.note_id],
                )?;
            }
        }

        self.build_exam_workspace(&attempt.course_id)
    }

    pub fn get_formula_workspace(
        &self,
        course_id: Option<String>,
    ) -> Result<Option<FormulaWorkspaceSnapshot>> {
        let selected = self.resolve_course_id(course_id)?;
        let Some(course_id) = selected else {
            return Ok(None);
        };
        self.set_selected_course(Some(&course_id))?;
        let course = self
            .find_course(&course_id)?
            .ok_or_else(|| anyhow!("course not found"))?;

        let aggregates = self.list_formula_aggregates(&course_id)?;
        let notes_with_formulas = aggregates
            .iter()
            .flat_map(|formula| formula.source_note_ids.iter().cloned())
            .collect::<HashSet<_>>()
            .len();
        let formula_mentions: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM formula_records WHERE course_id = ?1",
            params![&course_id],
            |row| row.get::<_, i64>(0).map(|value| value as usize),
        )?;
        let briefed_count = aggregates
            .iter()
            .filter(|formula| {
                self.get_cached_formula_brief(&formula.id, &course_id, &formula.source_hash)
                    .ok()
                    .flatten()
                    .is_some()
            })
            .count();

        Ok(Some(FormulaWorkspaceSnapshot {
            course_id,
            course_name: course.name,
            generated_at: now_string(),
            formulas: aggregates
                .iter()
                .map(|formula| FormulaSummary {
                    id: formula.id.clone(),
                    latex: formula.latex.clone(),
                    normalized_latex: formula.normalized_latex.clone(),
                    note_count: formula.note_count,
                    source_note_ids: formula.source_note_ids.clone(),
                    source_note_titles: formula.source_note_titles.clone(),
                })
                .collect(),
            summary: FormulaWorkspaceSummary {
                formula_count: aggregates.len(),
                notes_with_formulas,
                formula_mentions,
                briefed_count,
            },
        }))
    }

    pub fn get_formula_details(&self, formula_id: &str, course_id: &str) -> Result<FormulaDetails> {
        let aggregate = self
            .list_formula_aggregates(course_id)?
            .into_iter()
            .find(|formula| formula.id == formula_id)
            .ok_or_else(|| anyhow!("formula not found"))?;
        self.build_formula_details(course_id, aggregate)
    }

    pub fn generate_formula_brief(
        &self,
        request: GenerateFormulaBriefRequest,
    ) -> Result<FormulaBrief> {
        let settings = self
            .ai_settings_for_runtime()?
            .ok_or_else(|| anyhow!("enable AI in Setup before generating formula briefs"))?;
        let course = self
            .find_course(&request.course_id)?
            .ok_or_else(|| anyhow!("course not found"))?;
        let aggregate = self
            .list_formula_aggregates(&request.course_id)?
            .into_iter()
            .find(|formula| formula.id == request.formula_id)
            .ok_or_else(|| anyhow!("formula not found"))?;
        if !request.force.unwrap_or(false) {
            if let Some(cached) = self.get_cached_formula_brief(
                &request.formula_id,
                &request.course_id,
                &aggregate.source_hash,
            )? {
                return Ok(cached);
            }
        }
        let details = self.build_formula_details(&request.course_id, aggregate.clone())?;
        let chunks = details
            .chunks
            .iter()
            .map(|chunk| ChatContextChunkInput {
                citation_id: chunk.chunk_id.clone(),
                note_id: chunk.note_id.clone(),
                note_title: chunk.note_title.clone(),
                relative_path: chunk.relative_path.clone(),
                heading_path: chunk.heading_path.clone(),
                text: chunk.text.clone(),
            })
            .collect::<Vec<_>>();
        let brief = ai::generate_formula_brief(
            &settings,
            &course.name,
            &details.latex,
            &details.related_concepts,
            &details.headings,
            &chunks,
        )?;
        let stored_brief = FormulaBrief {
            formula_id: request.formula_id.clone(),
            coach: brief.coach,
            practice: brief.practice,
            derivation: brief.derivation,
            generated_at: now_string(),
            model: settings.model.clone(),
            source_signature: aggregate.source_hash.clone(),
        };
        self.upsert_formula_brief(
            &request.formula_id,
            &request.course_id,
            &aggregate.source_hash,
            &stored_brief,
        )?;
        Ok(stored_brief)
    }

    pub fn list_chat_threads(
        &self,
        scope: ChatScope,
        course_id: Option<String>,
    ) -> Result<Vec<ChatThreadSummary>> {
        let mut statement = match scope {
            ChatScope::Course => self.conn.prepare(
                r#"
                SELECT id, scope, course_id, title, created_at, updated_at
                FROM chat_threads
                WHERE scope = 'course' AND course_id = ?1
                ORDER BY updated_at DESC
                "#,
            )?,
            ChatScope::Vault => self.conn.prepare(
                r#"
                SELECT id, scope, course_id, title, created_at, updated_at
                FROM chat_threads
                WHERE scope = 'vault'
                ORDER BY updated_at DESC
                "#,
            )?,
        };

        let rows = match scope {
            ChatScope::Course => {
                let resolved_course_id = self
                    .resolve_course_id(course_id)?
                    .ok_or_else(|| anyhow!("select a course before opening course chat"))?;
                statement
                    .query_map(params![resolved_course_id], |row| {
                        self.read_chat_thread_row(row)
                    })?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            }
            ChatScope::Vault => statement
                .query_map([], |row| self.read_chat_thread_row(row))?
                .collect::<std::result::Result<Vec<_>, _>>()?,
        };

        rows.into_iter()
            .map(|thread| self.build_chat_thread_summary(thread))
            .collect()
    }

    pub fn create_chat_thread(
        &self,
        request: CreateChatThreadRequest,
    ) -> Result<ChatThreadDetails> {
        let course_id = match request.scope {
            ChatScope::Course => Some(
                self.resolve_course_id(request.course_id)?
                    .ok_or_else(|| anyhow!("select a course before creating a course chat"))?,
            ),
            ChatScope::Vault => None,
        };
        let now = now_string();
        let id = Uuid::new_v4().to_string();
        self.conn.execute(
            r#"
            INSERT INTO chat_threads (id, scope, course_id, title, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?5)
            "#,
            params![
                &id,
                chat_scope_to_str(request.scope),
                course_id,
                request
                    .title
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("New conversation"),
                now,
            ],
        )?;
        self.get_chat_thread(&id)
    }

    pub fn get_chat_thread(&self, thread_id: &str) -> Result<ChatThreadDetails> {
        let thread = self
            .find_chat_thread(thread_id)?
            .ok_or_else(|| anyhow!("chat thread not found"))?;
        let messages = self.load_chat_messages(thread_id)?;
        let course_name = thread
            .course_id
            .as_deref()
            .and_then(|course_id| self.find_course(course_id).ok().flatten())
            .map(|course| course.name);
        Ok(ChatThreadDetails {
            id: thread.id,
            scope: thread.scope,
            course_id: thread.course_id,
            course_name,
            title: thread.title,
            created_at: thread.created_at,
            updated_at: thread.updated_at,
            messages,
        })
    }

    pub fn send_chat_message(&self, request: SendChatMessageRequest) -> Result<ChatThreadDetails> {
        let settings = self
            .ai_settings_for_runtime()?
            .ok_or_else(|| anyhow!("enable AI in Setup before sending chat messages"))?;
        let thread = self
            .find_chat_thread(&request.thread_id)?
            .ok_or_else(|| anyhow!("chat thread not found"))?;
        let trimmed_message = request.content.trim();
        if trimmed_message.is_empty() {
            bail!("message cannot be empty");
        }

        let user_message_id = Uuid::new_v4().to_string();
        let now = now_string();
        self.conn.execute(
            r#"
            INSERT INTO chat_messages (id, thread_id, role, content, used_fallback, fallback_reason, created_at)
            VALUES (?1, ?2, ?3, ?4, NULL, NULL, ?5)
            "#,
            params![
                &user_message_id,
                &thread.id,
                chat_message_role_to_str(ChatMessageRole::User),
                trimmed_message,
                &now
            ],
        )?;

        let recent_messages = self.load_chat_messages(&thread.id)?;
        let retrieved_chunks =
            self.search_chat_chunks(thread.scope, thread.course_id.as_deref(), trimmed_message)?;
        let allow_fallback = retrieved_chunks.len() < 2;
        let transcript = recent_messages
            .iter()
            .map(|message| (message.role, message.content.clone()))
            .collect::<Vec<_>>();
        let course_name = if let Some(course_id) = thread.course_id.as_deref() {
            self.find_course(course_id)?.map(|course| course.name)
        } else {
            None
        };
        let answer = ai::answer_chat_query(
            &settings,
            thread.scope,
            course_name.as_deref(),
            &transcript,
            trimmed_message,
            &retrieved_chunks
                .iter()
                .enumerate()
                .map(|(index, chunk)| ChatContextChunkInput {
                    citation_id: format!("C{}", index + 1),
                    note_id: chunk.note_id.clone(),
                    note_title: chunk.note_title.clone(),
                    relative_path: chunk.relative_path.clone(),
                    heading_path: chunk.heading_path.clone(),
                    text: chunk.text.clone(),
                })
                .collect::<Vec<_>>(),
            allow_fallback,
        )?;

        let assistant_message_id = Uuid::new_v4().to_string();
        let assistant_timestamp = now_string();
        self.conn.execute(
            r#"
            INSERT INTO chat_messages (id, thread_id, role, content, used_fallback, fallback_reason, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                &assistant_message_id,
                &thread.id,
                chat_message_role_to_str(ChatMessageRole::Assistant),
                &answer.answer,
                bool_to_int(answer.used_fallback),
                answer.fallback_reason.as_deref(),
                &assistant_timestamp,
            ],
        )?;

        let chunk_by_citation = retrieved_chunks
            .iter()
            .enumerate()
            .map(|(index, chunk)| (format!("C{}", index + 1), chunk.clone()))
            .collect::<HashMap<_, _>>();
        for (position, citation_id) in answer.citation_ids.iter().enumerate() {
            if let Some(chunk) = chunk_by_citation.get(citation_id) {
                let course_name = self
                    .find_course(&chunk.course_id)?
                    .map(|course| course.name)
                    .unwrap_or_else(|| "Unknown course".to_string());
                let relevance = (1.0 - position as f64 * 0.08).max(0.2);
                self.conn.execute(
                    r#"
                    INSERT INTO chat_citations (
                      id, message_id, note_id, chunk_id, note_title, relative_path,
                      heading_path, excerpt, course_id, course_name, relevance, position
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                    "#,
                    params![
                        Uuid::new_v4().to_string(),
                        &assistant_message_id,
                        &chunk.note_id,
                        &chunk.chunk_id,
                        &chunk.note_title,
                        &chunk.relative_path,
                        &chunk.heading_path,
                        truncate_excerpt(&chunk.text, 220),
                        &chunk.course_id,
                        course_name,
                        relevance,
                        (position + 1) as i64,
                    ],
                )?;
            }
        }

        let next_title = if thread.title == "New conversation" {
            summarize_thread_title(trimmed_message)
        } else {
            thread.title.clone()
        };
        self.conn.execute(
            "UPDATE chat_threads SET title = ?2, updated_at = ?3 WHERE id = ?1",
            params![&thread.id, next_title, assistant_timestamp],
        )?;

        self.get_chat_thread(&thread.id)
    }

    pub fn delete_chat_thread(&self, thread_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM chat_threads WHERE id = ?1", params![thread_id])?;
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
                let source_modified_at = fs::metadata(&path)
                    .ok()
                    .and_then(|metadata| metadata.modified().ok())
                    .map(system_time_to_rfc3339);

                if existing_hashes
                    .get(&note_id)
                    .map(|existing| existing == &content_hash)
                    .unwrap_or(false)
                {
                    self.update_note_source_modified_at(&note_id, source_modified_at.as_deref())?;
                    unchanged_notes += 1;
                    continue;
                }

                let file_stem = path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("Untitled");
                let parsed = self.prepare_note_for_storage(file_stem, &content)?;
                self.upsert_note(
                    &course.id,
                    &note_id,
                    &relative_path,
                    &content_hash,
                    source_modified_at.as_deref(),
                    parsed,
                )?;
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
        self.append_statistics_snapshots(&courses, &scanned_at)?;

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

    fn update_note_source_modified_at(
        &self,
        note_id: &str,
        source_modified_at: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE note_records SET source_modified_at = ?2 WHERE id = ?1",
            params![note_id, source_modified_at],
        )?;
        Ok(())
    }

    fn prepare_note_for_storage(
        &self,
        file_stem: &str,
        content: &str,
    ) -> Result<ParsedStorageNote> {
        let parsed = parse_markdown(file_stem, content);
        let chunks = build_note_chunks(content, &parsed.headings);
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
            chunks,
        })
    }

    fn upsert_note(
        &self,
        course_id: &str,
        note_id: &str,
        relative_path: &str,
        content_hash: &str,
        source_modified_at: Option<&str>,
        parsed: ParsedStorageNote,
    ) -> Result<()> {
        let now = now_string();
        self.conn.execute(
            r#"
            INSERT INTO note_records (
              id, course_id, relative_path, title, content_hash, source_modified_at, frontmatter,
              frontmatter_exam_date, excerpt, headings_json, links_json, tags_json,
              prerequisites_json, concept_count, formula_count, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?16)
            ON CONFLICT(id) DO UPDATE SET
              relative_path = excluded.relative_path,
              title = excluded.title,
              content_hash = excluded.content_hash,
              source_modified_at = excluded.source_modified_at,
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
                source_modified_at,
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
        self.conn.execute(
            "DELETE FROM note_chunks WHERE note_id = ?1",
            params![note_id],
        )?;
        self.conn.execute(
            "DELETE FROM note_chunks_fts WHERE note_id = ?1",
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

        for chunk in parsed.chunks {
            let chunk_id = build_note_chunk_id(note_id, chunk.ordinal);
            self.conn.execute(
                r#"
                INSERT INTO note_chunks (chunk_id, note_id, course_id, heading_path, text, ordinal, content_hash)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![
                    &chunk_id,
                    note_id,
                    course_id,
                    chunk.heading_path,
                    chunk.text,
                    chunk.ordinal as i64,
                    content_hash,
                ],
            )?;
            self.conn.execute(
                r#"
                INSERT INTO note_chunks_fts (chunk_id, note_id, course_id, heading_path, text)
                VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
                params![chunk_id, note_id, course_id, chunk.heading_path, chunk.text],
            )?;
        }

        Ok(())
    }

    fn delete_note(&self, note_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM note_chunks_fts WHERE note_id = ?1",
            params![note_id],
        )?;
        self.conn.execute(
            "DELETE FROM note_chunks WHERE note_id = ?1",
            params![note_id],
        )?;
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
            SELECT id, title, relative_path, content_hash, source_modified_at, excerpt, headings_json, links_json, prerequisites_json, frontmatter_exam_date
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
                    source_modified_at: row.get(4)?,
                    excerpt: row.get(5)?,
                    headings: from_json_vec::<String>(&row.get::<_, String>(6)?),
                    links: from_json_vec::<String>(&row.get::<_, String>(7)?),
                    prerequisites: from_json_vec::<String>(&row.get::<_, String>(8)?),
                    frontmatter_exam_date: row.get(9)?,
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

    fn resolve_course_id(&self, course_id: Option<String>) -> Result<Option<String>> {
        if course_id.is_some() {
            return Ok(course_id);
        }

        if let Some(selected) = self.get_selected_course_id()? {
            return Ok(Some(selected));
        }

        Ok(self.list_courses()?.first().map(|course| course.id.clone()))
    }

    fn list_formula_aggregates(&self, course_id: &str) -> Result<Vec<StoredFormulaAggregate>> {
        let note_lookup = self
            .list_notes(course_id)?
            .into_iter()
            .map(|note| (note.id.clone(), note))
            .collect::<HashMap<_, _>>();
        let mut grouped = BTreeMap::<String, (String, Vec<String>, Vec<String>)>::new();

        for (note_id, latex, normalized_latex) in self.list_formulas(course_id)? {
            let entry = grouped
                .entry(normalized_latex.clone())
                .or_insert_with(|| (latex.clone(), Vec::new(), Vec::new()));
            if !entry.1.contains(&note_id) {
                entry.1.push(note_id.clone());
                if let Some(note) = note_lookup.get(&note_id) {
                    entry.2.push(note.title.clone());
                }
            }
            if latex.len() > entry.0.len() {
                entry.0 = latex.clone();
            }
        }

        Ok(grouped
            .into_iter()
            .map(
                |(normalized_latex, (latex, note_ids, note_titles))| StoredFormulaAggregate {
                    id: build_course_formula_id(course_id, &normalized_latex),
                    latex,
                    normalized_latex,
                    note_count: note_ids.len(),
                    source_hash: build_formula_source_hash(&note_ids, &note_lookup),
                    source_note_ids: note_ids,
                    source_note_titles: note_titles,
                },
            )
            .collect())
    }

    fn build_formula_details(
        &self,
        course_id: &str,
        aggregate: StoredFormulaAggregate,
    ) -> Result<FormulaDetails> {
        let note_lookup = self
            .list_notes(course_id)?
            .into_iter()
            .map(|note| (note.id.clone(), note))
            .collect::<HashMap<_, _>>();
        let concept_map = self.list_concepts(course_id)?.into_iter().fold(
            HashMap::<String, Vec<String>>::new(),
            |mut acc, (note_id, concept, _, _)| {
                acc.entry(note_id).or_default().push(concept);
                acc
            },
        );
        let formula_counts = self.list_formulas(course_id)?.into_iter().fold(
            HashMap::<String, usize>::new(),
            |mut acc, (note_id, _, _)| {
                *acc.entry(note_id).or_insert(0) += 1;
                acc
            },
        );
        let linked_notes = aggregate
            .source_note_ids
            .iter()
            .filter_map(|note_id| note_lookup.get(note_id))
            .map(|note| FormulaLinkedNote {
                note_id: note.id.clone(),
                title: note.title.clone(),
                relative_path: note.relative_path.clone(),
                excerpt: note.excerpt.clone(),
                headings: note.headings.clone(),
                related_concepts: concept_map.get(&note.id).cloned().unwrap_or_default(),
                formula_count: formula_counts.get(&note.id).copied().unwrap_or(0),
            })
            .collect::<Vec<_>>();
        let related_concepts = self.list_related_concepts_for_notes(&aggregate.source_note_ids)?;
        let headings = aggregate
            .source_note_ids
            .iter()
            .filter_map(|note_id| note_lookup.get(note_id))
            .flat_map(|note| note.headings.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let chunks = self
            .load_note_chunks_for_notes(course_id, &aggregate.source_note_ids)?
            .into_iter()
            .filter(|chunk| {
                chunk.text.contains(&aggregate.latex)
                    || normalize_key(&chunk.text).contains(&aggregate.normalized_latex)
            })
            .take(8)
            .map(|chunk| NoteChunkPreview {
                chunk_id: chunk.chunk_id,
                note_id: chunk.note_id,
                note_title: chunk.note_title,
                relative_path: chunk.relative_path,
                heading_path: chunk.heading_path,
                text: chunk.text,
                ordinal: chunk.ordinal,
            })
            .collect::<Vec<_>>();
        let brief =
            self.get_cached_formula_brief(&aggregate.id, course_id, &aggregate.source_hash)?;

        Ok(FormulaDetails {
            course_id: course_id.to_string(),
            id: aggregate.id,
            latex: aggregate.latex,
            normalized_latex: aggregate.normalized_latex,
            note_count: aggregate.note_count,
            source_note_ids: aggregate.source_note_ids,
            source_note_titles: aggregate.source_note_titles,
            linked_notes,
            chunks,
            related_concepts,
            headings,
            brief,
        })
    }

    fn get_cached_formula_brief(
        &self,
        formula_id: &str,
        course_id: &str,
        source_hash: &str,
    ) -> Result<Option<FormulaBrief>> {
        self.conn
            .query_row(
                r#"
                SELECT coach_json, practice_json, derivation_json, generated_at, model
                FROM formula_briefs
                WHERE formula_id = ?1 AND course_id = ?2 AND source_hash = ?3
                ORDER BY generated_at DESC
                LIMIT 1
                "#,
                params![formula_id, course_id, source_hash],
                |row| {
                    Ok(FormulaBrief {
                        formula_id: formula_id.to_string(),
                        coach: serde_json::from_str(&row.get::<_, String>(0)?)
                            .map_err(|error| to_sql_error(anyhow!(error)))?,
                        practice: serde_json::from_str(&row.get::<_, String>(1)?)
                            .map_err(|error| to_sql_error(anyhow!(error)))?,
                        derivation: serde_json::from_str(&row.get::<_, String>(2)?)
                            .map_err(|error| to_sql_error(anyhow!(error)))?,
                        generated_at: row.get(3)?,
                        model: row.get(4)?,
                        source_signature: source_hash.to_string(),
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    fn upsert_formula_brief(
        &self,
        formula_id: &str,
        course_id: &str,
        source_hash: &str,
        brief: &FormulaBrief,
    ) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO formula_briefs (
              formula_id, course_id, source_hash, coach_json, practice_json, derivation_json, generated_at, model
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(formula_id, source_hash) DO UPDATE SET
              coach_json = excluded.coach_json,
              practice_json = excluded.practice_json,
              derivation_json = excluded.derivation_json,
              generated_at = excluded.generated_at,
              model = excluded.model
            "#,
            params![
                formula_id,
                course_id,
                source_hash,
                serde_json::to_string(&brief.coach)?,
                serde_json::to_string(&brief.practice)?,
                serde_json::to_string(&brief.derivation)?,
                &brief.generated_at,
                &brief.model,
            ],
        )?;
        Ok(())
    }

    fn list_related_concepts_for_notes(&self, note_ids: &[String]) -> Result<Vec<String>> {
        let note_id_set = note_ids.iter().cloned().collect::<HashSet<_>>();
        Ok(note_ids
            .iter()
            .flat_map(|note_id| {
                self.list_concepts_for_note(note_id)
                    .unwrap_or_default()
                    .into_iter()
            })
            .filter(|(note_id, _, _, _)| note_id_set.contains(note_id))
            .map(|(_, concept, _, _)| concept)
            .collect::<HashSet<_>>()
            .into_iter()
            .take(12)
            .collect())
    }

    fn list_concepts_for_note(&self, note_id: &str) -> Result<Vec<(String, String, String, f64)>> {
        let mut statement = self.conn.prepare(
            "SELECT note_id, name, normalized_name, support_score FROM concept_records WHERE note_id = ?1 ORDER BY support_score DESC",
        )?;
        let rows = statement
            .query_map(params![note_id], |row| {
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

    fn load_note_chunks_for_notes(
        &self,
        course_id: &str,
        note_ids: &[String],
    ) -> Result<Vec<StoredNoteChunk>> {
        let note_id_set = note_ids.iter().cloned().collect::<HashSet<_>>();
        let mut statement = self.conn.prepare(
            r#"
            SELECT note_chunks.chunk_id, note_chunks.note_id, note_chunks.course_id, note_records.title,
                   note_records.relative_path, note_chunks.heading_path, note_chunks.text, note_chunks.ordinal
            FROM note_chunks
            INNER JOIN note_records ON note_records.id = note_chunks.note_id
            WHERE note_chunks.course_id = ?1
            ORDER BY note_chunks.ordinal ASC
            "#,
        )?;
        let rows = statement
            .query_map(params![course_id], |row| {
                Ok(StoredNoteChunk {
                    chunk_id: row.get(0)?,
                    note_id: row.get(1)?,
                    course_id: row.get(2)?,
                    note_title: row.get(3)?,
                    relative_path: row.get(4)?,
                    heading_path: row.get(5)?,
                    text: row.get(6)?,
                    ordinal: row.get::<_, i64>(7)? as usize,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows
            .into_iter()
            .filter(|chunk| note_id_set.contains(&chunk.note_id))
            .collect())
    }

    fn search_chat_chunks(
        &self,
        scope: ChatScope,
        course_id: Option<&str>,
        query: &str,
    ) -> Result<Vec<StoredNoteChunk>> {
        let normalized_query = normalize_key(query);
        let fts_query = build_fts_query(&normalized_query);
        let mut rows = Vec::new();

        if let Some(fts_query) = fts_query {
            let query_sql = match scope {
                ChatScope::Course => {
                    let course_id =
                        course_id.ok_or_else(|| anyhow!("course chat requires a course"))?;
                    let mut statement = self.conn.prepare(
                        r#"
                        SELECT note_chunks.chunk_id, note_chunks.note_id, note_chunks.course_id, note_records.title,
                               note_records.relative_path, note_chunks.heading_path, note_chunks.text, note_chunks.ordinal
                        FROM note_chunks_fts
                        INNER JOIN note_chunks ON note_chunks.chunk_id = note_chunks_fts.chunk_id
                        INNER JOIN note_records ON note_records.id = note_chunks.note_id
                        WHERE note_chunks_fts MATCH ?1 AND note_chunks.course_id = ?2
                        ORDER BY bm25(note_chunks_fts)
                        LIMIT 10
                        "#
                    )?;
                    let mapped = statement.query_map(params![fts_query, course_id], |row| {
                        Ok(StoredNoteChunk {
                            chunk_id: row.get(0)?,
                            note_id: row.get(1)?,
                            course_id: row.get(2)?,
                            note_title: row.get(3)?,
                            relative_path: row.get(4)?,
                            heading_path: row.get(5)?,
                            text: row.get(6)?,
                            ordinal: row.get::<_, i64>(7)? as usize,
                        })
                    })?;
                    mapped.collect::<std::result::Result<Vec<_>, _>>()?
                }
                ChatScope::Vault => {
                    let mut statement = self.conn.prepare(
                        r#"
                        SELECT note_chunks.chunk_id, note_chunks.note_id, note_chunks.course_id, note_records.title,
                               note_records.relative_path, note_chunks.heading_path, note_chunks.text, note_chunks.ordinal
                        FROM note_chunks_fts
                        INNER JOIN note_chunks ON note_chunks.chunk_id = note_chunks_fts.chunk_id
                        INNER JOIN note_records ON note_records.id = note_chunks.note_id
                        WHERE note_chunks_fts MATCH ?1
                        ORDER BY bm25(note_chunks_fts)
                        LIMIT 10
                        "#
                    )?;
                    let mapped = statement.query_map(params![fts_query], |row| {
                        Ok(StoredNoteChunk {
                            chunk_id: row.get(0)?,
                            note_id: row.get(1)?,
                            course_id: row.get(2)?,
                            note_title: row.get(3)?,
                            relative_path: row.get(4)?,
                            heading_path: row.get(5)?,
                            text: row.get(6)?,
                            ordinal: row.get::<_, i64>(7)? as usize,
                        })
                    })?;
                    mapped.collect::<std::result::Result<Vec<_>, _>>()?
                }
            };
            rows.extend(query_sql);
        }

        let supplemental_note_ids = self.search_formula_note_ids(scope, course_id, query)?;
        if !supplemental_note_ids.is_empty() {
            let note_id_set = supplemental_note_ids.into_iter().collect::<HashSet<_>>();
            let supplemental = self
                .load_note_chunks_for_scope(scope, course_id)?
                .into_iter()
                .filter(|chunk| note_id_set.contains(&chunk.note_id))
                .take(4)
                .collect::<Vec<_>>();
            rows.extend(supplemental);
        }

        let mut seen = HashSet::new();
        rows.retain(|chunk| seen.insert(chunk.chunk_id.clone()));
        Ok(rows.into_iter().take(10).collect())
    }

    fn search_formula_note_ids(
        &self,
        scope: ChatScope,
        course_id: Option<&str>,
        query: &str,
    ) -> Result<Vec<String>> {
        let normalized = normalize_key(query);
        if normalized.is_empty() {
            return Ok(Vec::new());
        }

        let like_query = format!("%{normalized}%");
        let mut statement = match scope {
            ChatScope::Course => self.conn.prepare(
                "SELECT DISTINCT note_id FROM formula_records WHERE course_id = ?1 AND normalized_latex LIKE ?2 LIMIT 6",
            )?,
            ChatScope::Vault => self.conn.prepare(
                "SELECT DISTINCT note_id FROM formula_records WHERE normalized_latex LIKE ?1 LIMIT 6",
            )?,
        };

        let rows = match scope {
            ChatScope::Course => statement
                .query_map(
                    params![
                        course_id.ok_or_else(|| anyhow!("course chat requires a course"))?,
                        like_query
                    ],
                    |row| row.get::<_, String>(0),
                )?
                .collect::<std::result::Result<Vec<_>, _>>()?,
            ChatScope::Vault => statement
                .query_map(params![like_query], |row| row.get::<_, String>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?,
        };
        Ok(rows)
    }

    fn load_note_chunks_for_scope(
        &self,
        scope: ChatScope,
        course_id: Option<&str>,
    ) -> Result<Vec<StoredNoteChunk>> {
        let mut statement = match scope {
            ChatScope::Course => self.conn.prepare(
                r#"
                SELECT note_chunks.chunk_id, note_chunks.note_id, note_chunks.course_id, note_records.title,
                       note_records.relative_path, note_chunks.heading_path, note_chunks.text, note_chunks.ordinal
                FROM note_chunks
                INNER JOIN note_records ON note_records.id = note_chunks.note_id
                WHERE note_chunks.course_id = ?1
                ORDER BY note_chunks.ordinal ASC
                "#,
            )?,
            ChatScope::Vault => self.conn.prepare(
                r#"
                SELECT note_chunks.chunk_id, note_chunks.note_id, note_chunks.course_id, note_records.title,
                       note_records.relative_path, note_chunks.heading_path, note_chunks.text, note_chunks.ordinal
                FROM note_chunks
                INNER JOIN note_records ON note_records.id = note_chunks.note_id
                ORDER BY note_chunks.ordinal ASC
                "#,
            )?,
        };
        let rows = match scope {
            ChatScope::Course => statement
                .query_map(
                    params![course_id.ok_or_else(|| anyhow!("course chat requires a course"))?],
                    |row| {
                        Ok(StoredNoteChunk {
                            chunk_id: row.get(0)?,
                            note_id: row.get(1)?,
                            course_id: row.get(2)?,
                            note_title: row.get(3)?,
                            relative_path: row.get(4)?,
                            heading_path: row.get(5)?,
                            text: row.get(6)?,
                            ordinal: row.get::<_, i64>(7)? as usize,
                        })
                    },
                )?
                .collect::<std::result::Result<Vec<_>, _>>()?,
            ChatScope::Vault => statement
                .query_map([], |row| {
                    Ok(StoredNoteChunk {
                        chunk_id: row.get(0)?,
                        note_id: row.get(1)?,
                        course_id: row.get(2)?,
                        note_title: row.get(3)?,
                        relative_path: row.get(4)?,
                        heading_path: row.get(5)?,
                        text: row.get(6)?,
                        ordinal: row.get::<_, i64>(7)? as usize,
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?,
        };
        Ok(rows)
    }

    fn read_chat_thread_row(&self, row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredChatThread> {
        Ok(StoredChatThread {
            id: row.get(0)?,
            scope: chat_scope_from_str(&row.get::<_, String>(1)?).map_err(to_sql_error)?,
            course_id: row.get(2)?,
            title: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }

    fn build_chat_thread_summary(&self, thread: StoredChatThread) -> Result<ChatThreadSummary> {
        let message_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM chat_messages WHERE thread_id = ?1",
            params![&thread.id],
            |row| row.get::<_, i64>(0).map(|value| value as usize),
        )?;
        let last_message_preview = self.conn.query_row(
            "SELECT content FROM chat_messages WHERE thread_id = ?1 ORDER BY created_at DESC LIMIT 1",
            params![&thread.id],
            |row| row.get::<_, String>(0),
        ).optional()?.map(|content| truncate_excerpt(&content, 120));
        let course_name = thread
            .course_id
            .as_deref()
            .and_then(|course_id| self.find_course(course_id).ok().flatten())
            .map(|course| course.name);

        Ok(ChatThreadSummary {
            id: thread.id,
            scope: thread.scope,
            course_id: thread.course_id,
            course_name,
            title: thread.title,
            created_at: thread.created_at,
            updated_at: thread.updated_at,
            message_count,
            last_message_preview,
        })
    }

    fn find_chat_thread(&self, thread_id: &str) -> Result<Option<StoredChatThread>> {
        self.conn
            .query_row(
                r#"
                SELECT id, scope, course_id, title, created_at, updated_at
                FROM chat_threads
                WHERE id = ?1
                "#,
                params![thread_id],
                |row| self.read_chat_thread_row(row),
            )
            .optional()
            .map_err(Into::into)
    }

    fn load_chat_messages(&self, thread_id: &str) -> Result<Vec<ChatMessage>> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT id, role, content, used_fallback, fallback_reason, created_at
            FROM chat_messages
            WHERE thread_id = ?1
            ORDER BY created_at ASC
            "#,
        )?;
        let raw_rows = statement
            .query_map(params![thread_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut messages = Vec::with_capacity(raw_rows.len());
        for (id, role, content, used_fallback, fallback_reason, created_at) in raw_rows {
            messages.push(ChatMessage {
                citations: self.load_chat_citations(&id)?,
                id: id.clone(),
                thread_id: thread_id.to_string(),
                role: chat_message_role_from_str(&role)?,
                content,
                used_fallback: used_fallback.map(|value| value == 1).unwrap_or(false),
                fallback_reason,
                created_at,
            });
        }
        Ok(messages)
    }

    fn load_chat_citations(&self, message_id: &str) -> Result<Vec<ChatCitation>> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT note_id, note_title, relative_path, chunk_id, heading_path, excerpt,
                   course_id, course_name, relevance
            FROM chat_citations
            WHERE message_id = ?1
            ORDER BY position ASC
            "#,
        )?;
        let rows = statement
            .query_map(params![message_id], |row| {
                Ok(ChatCitation {
                    chunk_id: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    note_id: row.get(0)?,
                    note_title: row.get(1)?,
                    relative_path: row.get(2)?,
                    heading_path: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                    excerpt: row.get(5)?,
                    course_id: row.get(6)?,
                    course_name: row.get(7)?,
                    relevance: row.get(8)?,
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

    fn build_course_statistics_overview(&self, course_id: &str) -> Result<StatisticsOverview> {
        let notes = self.list_notes(course_id)?;
        let concepts = self.list_concepts(course_id)?;
        let formulas = self.list_formulas(course_id)?;
        let edges = self.list_edges(course_id)?;
        let weak_rows = self.list_weak_suggestions(course_id)?;
        let flashcard_sets = self.list_flashcard_sets(course_id)?;
        let revision_runs = self.list_revision_runs(course_id)?;
        let ai_states = self.list_ai_note_states(course_id)?;
        let ai_run = self.get_ai_course_run(course_id)?;
        let coverage = build_coverage(&notes, &concepts, &formulas, &flashcard_sets);
        let graph = build_graph_stats(&notes, &edges);
        let weak_note_count = build_weak_notes(&notes, &weak_rows).len();
        let formula_count = formulas
            .iter()
            .map(|(_, _, normalized)| normalized.clone())
            .collect::<HashSet<_>>()
            .len();
        let note_summaries = build_note_summaries(&notes, &edges, &concepts, &formulas, &ai_states);
        let average_note_strength = if note_summaries.is_empty() {
            0.0
        } else {
            round_percentage(
                note_summaries.iter().map(|note| note.strength).sum::<f64>()
                    / note_summaries.len() as f64,
            )
        };
        let notes_with_formulas = formulas
            .iter()
            .map(|(note_id, _, _)| note_id.clone())
            .collect::<HashSet<_>>()
            .len();
        let flashcard_summary = build_flashcard_summary(&flashcard_sets);
        let revision_summary = build_revision_summary(&revision_runs);
        let ai_summary =
            build_ai_course_summary(notes.len(), None, &notes, &ai_states, ai_run.as_ref());
        let (exam_attempt_count, latest_exam_score, average_exam_score) =
            self.build_exam_score_summary(Some(course_id))?;

        Ok(StatisticsOverview {
            note_count: notes.len(),
            total_concepts: coverage.total_concepts,
            covered_concepts: coverage.covered_concepts,
            coverage_percentage: coverage.percentage,
            edge_count: graph.edge_count,
            strong_links: graph.strong_links,
            inferred_links: graph.inferred_links,
            isolated_notes: graph.isolated_notes,
            weak_note_count,
            formula_count,
            notes_with_formulas,
            average_note_strength,
            flashcard_set_count: flashcard_summary.set_count,
            flashcard_total_cards: flashcard_summary.total_cards,
            revision_run_count: revision_runs.len(),
            latest_revision_item_count: revision_summary.item_count,
            ai_ready_notes: ai_summary.ready_notes,
            ai_pending_notes: ai_summary.pending_notes,
            ai_failed_notes: ai_summary.failed_notes,
            ai_stale_notes: ai_summary.stale_notes,
            ai_missing_notes: ai_summary.missing_notes,
            exam_attempt_count,
            latest_exam_score,
            average_exam_score,
        })
    }

    fn build_vault_statistics_overview(
        &self,
        courses: &[StoredCourse],
    ) -> Result<StatisticsOverview> {
        let mut combined = StatisticsOverview {
            note_count: 0,
            total_concepts: 0,
            covered_concepts: 0,
            coverage_percentage: 0.0,
            edge_count: 0,
            strong_links: 0,
            inferred_links: 0,
            isolated_notes: 0,
            weak_note_count: 0,
            formula_count: 0,
            notes_with_formulas: 0,
            average_note_strength: 0.0,
            flashcard_set_count: 0,
            flashcard_total_cards: 0,
            revision_run_count: 0,
            latest_revision_item_count: 0,
            ai_ready_notes: 0,
            ai_pending_notes: 0,
            ai_failed_notes: 0,
            ai_stale_notes: 0,
            ai_missing_notes: 0,
            exam_attempt_count: 0,
            latest_exam_score: None,
            average_exam_score: None,
        };

        for course in courses {
            let overview = self.build_course_statistics_overview(&course.id)?;
            combined.note_count += overview.note_count;
            combined.total_concepts += overview.total_concepts;
            combined.covered_concepts += overview.covered_concepts;
            combined.edge_count += overview.edge_count;
            combined.strong_links += overview.strong_links;
            combined.inferred_links += overview.inferred_links;
            combined.isolated_notes += overview.isolated_notes;
            combined.weak_note_count += overview.weak_note_count;
            combined.formula_count += overview.formula_count;
            combined.notes_with_formulas += overview.notes_with_formulas;
            combined.flashcard_set_count += overview.flashcard_set_count;
            combined.flashcard_total_cards += overview.flashcard_total_cards;
            combined.revision_run_count += overview.revision_run_count;
            combined.latest_revision_item_count += overview.latest_revision_item_count;
            combined.ai_ready_notes += overview.ai_ready_notes;
            combined.ai_pending_notes += overview.ai_pending_notes;
            combined.ai_failed_notes += overview.ai_failed_notes;
            combined.ai_stale_notes += overview.ai_stale_notes;
            combined.ai_missing_notes += overview.ai_missing_notes;
            combined.exam_attempt_count += overview.exam_attempt_count;
        }

        combined.coverage_percentage = if combined.total_concepts == 0 {
            0.0
        } else {
            round_percentage(
                (combined.covered_concepts as f64 / combined.total_concepts as f64) * 100.0,
            )
        };
        combined.average_note_strength = if combined.note_count == 0 {
            0.0
        } else {
            round_percentage(
                courses
                    .iter()
                    .map(|course| self.build_course_statistics_overview(&course.id))
                    .collect::<Result<Vec<_>>>()?
                    .iter()
                    .map(|overview| overview.average_note_strength * overview.note_count as f64)
                    .sum::<f64>()
                    / combined.note_count as f64,
            )
        };

        let (_, latest_exam_score, average_exam_score) = self.build_exam_score_summary(None)?;
        combined.latest_exam_score = latest_exam_score;
        combined.average_exam_score = average_exam_score;

        Ok(combined)
    }

    fn build_course_statistics_rows(
        &self,
        courses: &[StoredCourse],
    ) -> Result<Vec<CourseStatisticsRow>> {
        let mut rows = courses
            .iter()
            .map(|course| {
                let overview = self.build_course_statistics_overview(&course.id)?;
                Ok(CourseStatisticsRow {
                    course_id: course.id.clone(),
                    course_name: course.name.clone(),
                    note_count: overview.note_count,
                    coverage_percentage: overview.coverage_percentage,
                    edge_count: overview.edge_count,
                    weak_note_count: overview.weak_note_count,
                    formula_count: overview.formula_count,
                    average_note_strength: overview.average_note_strength,
                    flashcard_total_cards: overview.flashcard_total_cards,
                    revision_run_count: overview.revision_run_count,
                    ai_ready_notes: overview.ai_ready_notes,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        rows.sort_by(|left, right| left.course_name.cmp(&right.course_name));
        Ok(rows)
    }

    fn build_exam_score_summary(
        &self,
        course_id: Option<&str>,
    ) -> Result<(usize, Option<f64>, Option<f64>)> {
        let attempt_count = match course_id {
            Some(course_id) => self.conn.query_row(
                "SELECT COUNT(*) FROM exam_attempts WHERE course_id = ?1",
                params![course_id],
                |row| row.get::<_, i64>(0).map(|value| value as usize),
            )?,
            None => self.conn.query_row("SELECT COUNT(*) FROM exam_attempts", [], |row| {
                row.get::<_, i64>(0).map(|value| value as usize)
            })?,
        };

        let latest_exam_score = match course_id {
            Some(course_id) => self
                .conn
                .query_row(
                    "SELECT score_percent FROM exam_attempts WHERE course_id = ?1 ORDER BY submitted_at DESC LIMIT 1",
                    params![course_id],
                    |row| row.get::<_, f64>(0),
                )
                .optional()?,
            None => self
                .conn
                .query_row(
                    "SELECT score_percent FROM exam_attempts ORDER BY submitted_at DESC LIMIT 1",
                    [],
                    |row| row.get::<_, f64>(0),
                )
                .optional()?,
        };

        let average_exam_score = match course_id {
            Some(course_id) => self
                .conn
                .query_row(
                    "SELECT AVG(score_percent) FROM exam_attempts WHERE course_id = ?1",
                    params![course_id],
                    |row| row.get::<_, Option<f64>>(0),
                )?,
            None => self.conn.query_row("SELECT AVG(score_percent) FROM exam_attempts", [], |row| {
                row.get::<_, Option<f64>>(0)
            })?,
        }
        .map(round_percentage);

        Ok((attempt_count, latest_exam_score.map(round_percentage), average_exam_score))
    }

    fn list_statistics_history(
        &self,
        scope: StatisticsScope,
        course_id: Option<&str>,
    ) -> Result<Vec<StatisticsSnapshotPoint>> {
        let scope_value = statistics_scope_to_str(scope);
        let mut statement = match scope {
            StatisticsScope::Course => self.conn.prepare(
                r#"
                SELECT captured_at, note_count, total_concepts, covered_concepts, coverage_percentage,
                       edge_count, strong_links, inferred_links, isolated_notes, weak_note_count,
                       formula_count, notes_with_formulas, average_note_strength,
                       flashcard_set_count, flashcard_total_cards, revision_run_count,
                       latest_revision_item_count, ai_ready_notes, ai_pending_notes,
                       ai_failed_notes, ai_stale_notes, ai_missing_notes,
                       exam_attempt_count, latest_exam_score, average_exam_score
                FROM stats_snapshots
                WHERE scope = ?1 AND course_id = ?2
                ORDER BY captured_at ASC
                "#,
            )?,
            StatisticsScope::Vault => self.conn.prepare(
                r#"
                SELECT captured_at, note_count, total_concepts, covered_concepts, coverage_percentage,
                       edge_count, strong_links, inferred_links, isolated_notes, weak_note_count,
                       formula_count, notes_with_formulas, average_note_strength,
                       flashcard_set_count, flashcard_total_cards, revision_run_count,
                       latest_revision_item_count, ai_ready_notes, ai_pending_notes,
                       ai_failed_notes, ai_stale_notes, ai_missing_notes,
                       exam_attempt_count, latest_exam_score, average_exam_score
                FROM stats_snapshots
                WHERE scope = ?1 AND course_id IS NULL
                ORDER BY captured_at ASC
                "#,
            )?,
        };

        let rows = match scope {
            StatisticsScope::Course => statement
                .query_map(
                    params![scope_value, course_id.ok_or_else(|| anyhow!("course statistics require a course"))?],
                    read_statistics_snapshot_row,
                )?
                .collect::<std::result::Result<Vec<_>, _>>()?,
            StatisticsScope::Vault => statement
                .query_map(params![scope_value], read_statistics_snapshot_row)?
                .collect::<std::result::Result<Vec<_>, _>>()?,
        };
        Ok(rows)
    }

    fn list_statistics_exam_points(
        &self,
        course_id: Option<&str>,
    ) -> Result<Vec<StatisticsExamPoint>> {
        let mut statement = match course_id {
            Some(_) => self.conn.prepare(
                r#"
                SELECT exam_attempts.submitted_at, exam_attempts.exam_id, exams.title,
                       exam_attempts.score_percent, course_configs.id, course_configs.name
                FROM exam_attempts
                INNER JOIN exams ON exams.id = exam_attempts.exam_id
                INNER JOIN course_configs ON course_configs.id = exam_attempts.course_id
                WHERE exam_attempts.course_id = ?1
                ORDER BY exam_attempts.submitted_at ASC
                "#,
            )?,
            None => self.conn.prepare(
                r#"
                SELECT exam_attempts.submitted_at, exam_attempts.exam_id, exams.title,
                       exam_attempts.score_percent, course_configs.id, course_configs.name
                FROM exam_attempts
                INNER JOIN exams ON exams.id = exam_attempts.exam_id
                INNER JOIN course_configs ON course_configs.id = exam_attempts.course_id
                ORDER BY exam_attempts.submitted_at ASC
                "#,
            )?,
        };

        let rows = match course_id {
            Some(course_id) => statement
                .query_map(params![course_id], |row| {
                    Ok(StatisticsExamPoint {
                        submitted_at: row.get(0)?,
                        exam_id: row.get(1)?,
                        exam_title: row.get(2)?,
                        score_percent: round_percentage(row.get(3)?),
                        course_id: Some(row.get(4)?),
                        course_name: Some(row.get(5)?),
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?,
            None => statement
                .query_map([], |row| {
                    Ok(StatisticsExamPoint {
                        submitted_at: row.get(0)?,
                        exam_id: row.get(1)?,
                        exam_title: row.get(2)?,
                        score_percent: round_percentage(row.get(3)?),
                        course_id: Some(row.get(4)?),
                        course_name: Some(row.get(5)?),
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?,
        };
        Ok(rows)
    }

    fn build_activity_buckets(&self, course_id: Option<&str>) -> Result<Vec<VaultActivityBucket>> {
        let total_notes = match course_id {
            Some(course_id) => self.conn.query_row(
                "SELECT COUNT(*) FROM note_records WHERE course_id = ?1",
                params![course_id],
                |row| row.get::<_, i64>(0).map(|value| value as usize),
            )?,
            None => self.conn.query_row("SELECT COUNT(*) FROM note_records", [], |row| {
                row.get::<_, i64>(0).map(|value| value as usize)
            })?,
        };

        let mut statement = match course_id {
            Some(_) => self
                .conn
                .prepare("SELECT source_modified_at FROM note_records WHERE course_id = ?1")?,
            None => self.conn.prepare("SELECT source_modified_at FROM note_records")?,
        };

        let modified_rows = match course_id {
            Some(course_id) => statement
                .query_map(params![course_id], |row| row.get::<_, Option<String>>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?,
            None => statement
                .query_map([], |row| row.get::<_, Option<String>>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?,
        };

        let mut counts = [0usize; 5];
        let now = Utc::now();

        for value in modified_rows {
            match value {
                Some(value) => match DateTime::parse_from_rfc3339(&value) {
                    Ok(parsed) => {
                        let age_days = (now - parsed.with_timezone(&Utc)).num_days().max(0);
                        match age_days {
                            0..=7 => counts[0] += 1,
                            8..=30 => counts[1] += 1,
                            31..=90 => counts[2] += 1,
                            _ => counts[3] += 1,
                        }
                    }
                    Err(_) => counts[4] += 1,
                },
                None => counts[4] += 1,
            }
        }

        let known_notes = counts[..4].iter().sum::<usize>() + counts[4];
        if total_notes > known_notes {
            counts[4] += total_notes - known_notes;
        }

        Ok(vec![
            VaultActivityBucket {
                label: "0-7 days".to_string(),
                note_count: counts[0],
            },
            VaultActivityBucket {
                label: "8-30 days".to_string(),
                note_count: counts[1],
            },
            VaultActivityBucket {
                label: "31-90 days".to_string(),
                note_count: counts[2],
            },
            VaultActivityBucket {
                label: "90+ days".to_string(),
                note_count: counts[3],
            },
            VaultActivityBucket {
                label: "Unknown".to_string(),
                note_count: counts[4],
            },
        ])
    }

    fn append_statistics_snapshots(&self, courses: &[StoredCourse], captured_at: &str) -> Result<()> {
        for course in courses {
            let overview = self.build_course_statistics_overview(&course.id)?;
            self.insert_statistics_snapshot(
                StatisticsScope::Course,
                Some(&course.id),
                captured_at,
                &overview,
            )?;
        }

        let vault_overview = self.build_vault_statistics_overview(courses)?;
        self.insert_statistics_snapshot(StatisticsScope::Vault, None, captured_at, &vault_overview)?;
        Ok(())
    }

    fn insert_statistics_snapshot(
        &self,
        scope: StatisticsScope,
        course_id: Option<&str>,
        captured_at: &str,
        overview: &StatisticsOverview,
    ) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO stats_snapshots (
              id, scope, course_id, captured_at, note_count, total_concepts, covered_concepts,
              coverage_percentage, edge_count, strong_links, inferred_links, isolated_notes,
              weak_note_count, formula_count, notes_with_formulas, average_note_strength,
              flashcard_set_count, flashcard_total_cards, revision_run_count,
              latest_revision_item_count, ai_ready_notes, ai_pending_notes, ai_failed_notes,
              ai_stale_notes, ai_missing_notes, exam_attempt_count, latest_exam_score,
              average_exam_score
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28)
            "#,
            params![
                Uuid::new_v4().to_string(),
                statistics_scope_to_str(scope),
                course_id,
                captured_at,
                overview.note_count as i64,
                overview.total_concepts as i64,
                overview.covered_concepts as i64,
                overview.coverage_percentage,
                overview.edge_count as i64,
                overview.strong_links as i64,
                overview.inferred_links as i64,
                overview.isolated_notes as i64,
                overview.weak_note_count as i64,
                overview.formula_count as i64,
                overview.notes_with_formulas as i64,
                overview.average_note_strength,
                overview.flashcard_set_count as i64,
                overview.flashcard_total_cards as i64,
                overview.revision_run_count as i64,
                overview.latest_revision_item_count as i64,
                overview.ai_ready_notes as i64,
                overview.ai_pending_notes as i64,
                overview.ai_failed_notes as i64,
                overview.ai_stale_notes as i64,
                overview.ai_missing_notes as i64,
                overview.exam_attempt_count as i64,
                overview.latest_exam_score,
                overview.average_exam_score,
            ],
        )?;
        Ok(())
    }

    fn compute_coverage_for_course(&self, course_id: &str) -> Result<CoverageStats> {
        let notes = self.list_notes(course_id)?;
        let concepts = self.list_concepts(course_id)?;
        let formulas = self.list_formulas(course_id)?;
        let flashcards = self.list_flashcard_sets(course_id)?;
        Ok(build_coverage(&notes, &concepts, &formulas, &flashcards))
    }
}

impl Database {
    fn get_exam_workspace_summary(&self, course_id: &str) -> Result<ExamWorkspaceSummary> {
        let source_queue_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM exam_source_queue WHERE course_id = ?1",
            params![course_id],
            |row| row.get::<_, i64>(0).map(|value| value as usize),
        )?;
        let queued_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM exams WHERE course_id = ?1 AND status = 'queued'",
            params![course_id],
            |row| row.get::<_, i64>(0).map(|value| value as usize),
        )?;
        let generating_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM exams WHERE course_id = ?1 AND status = 'generating'",
            params![course_id],
            |row| row.get::<_, i64>(0).map(|value| value as usize),
        )?;
        let ready_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM exams WHERE course_id = ?1 AND status = 'ready'",
            params![course_id],
            |row| row.get::<_, i64>(0).map(|value| value as usize),
        )?;
        let failed_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM exams WHERE course_id = ?1 AND status = 'failed'",
            params![course_id],
            |row| row.get::<_, i64>(0).map(|value| value as usize),
        )?;
        let review_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM note_mastery_states WHERE course_id = ?1 AND mastery_state = 'review'",
            params![course_id],
            |row| row.get::<_, i64>(0).map(|value| value as usize),
        )?;
        let mastered_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM note_mastery_states WHERE course_id = ?1 AND mastery_state = 'mastered'",
            params![course_id],
            |row| row.get::<_, i64>(0).map(|value| value as usize),
        )?;
        let latest_attempted_at = self
            .conn
            .query_row(
                "SELECT submitted_at FROM exam_attempts WHERE course_id = ?1 ORDER BY submitted_at DESC LIMIT 1",
                params![course_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?;

        Ok(ExamWorkspaceSummary {
            source_queue_count,
            queued_count,
            generating_count,
            ready_count,
            failed_count,
            review_count,
            mastered_count,
            latest_attempted_at,
        })
    }

    fn build_exam_workspace(&self, course_id: &str) -> Result<ExamWorkspaceSnapshot> {
        let source_queue = self.list_exam_source_queue_notes(course_id)?;
        let note_mastery = self.list_note_mastery_by_course(course_id)?;
        let note_lookup = self
            .list_notes(course_id)?
            .into_iter()
            .map(|note| (note.id.clone(), note))
            .collect::<HashMap<_, _>>();
        let concept_counts = self.list_concepts(course_id)?.into_iter().fold(
            HashMap::<String, usize>::new(),
            |mut acc, (note_id, _, _, _)| {
                *acc.entry(note_id).or_insert(0) += 1;
                acc
            },
        );
        let formula_counts = self.list_formulas(course_id)?.into_iter().fold(
            HashMap::<String, usize>::new(),
            |mut acc, (note_id, _, _)| {
                *acc.entry(note_id).or_insert(0) += 1;
                acc
            },
        );
        let ai_states = self.list_ai_note_states(course_id)?;
        let exam_records = self.list_exam_records(course_id)?;
        let history = self.list_exam_attempt_summaries(course_id)?;

        let mut queued_exams = Vec::new();
        let mut ready_exams = Vec::new();
        let mut failed_exams = Vec::new();
        for record in exam_records {
            match record.status {
                ExamStatus::Queued | ExamStatus::Generating => {
                    queued_exams.push(self.build_exam_summary(record)?)
                }
                ExamStatus::Ready => ready_exams.push(self.build_exam_summary(record)?),
                ExamStatus::Failed => failed_exams.push(self.build_exam_summary(record)?),
            }
        }

        let review_notes = note_lookup
            .values()
            .filter(|note| {
                note_mastery
                    .get(&note.id)
                    .map(|row| row.mastery_state == NoteMasteryState::Review)
                    .unwrap_or(false)
            })
            .map(|note| {
                self.build_exam_source_note(
                    note,
                    note_mastery.get(&note.id),
                    &concept_counts,
                    &formula_counts,
                    &ai_states,
                )
            })
            .collect::<Vec<_>>();
        let mastered_notes = note_lookup
            .values()
            .filter(|note| {
                note_mastery
                    .get(&note.id)
                    .map(|row| row.mastery_state == NoteMasteryState::Mastered)
                    .unwrap_or(false)
            })
            .map(|note| {
                self.build_exam_source_note(
                    note,
                    note_mastery.get(&note.id),
                    &concept_counts,
                    &formula_counts,
                    &ai_states,
                )
            })
            .collect::<Vec<_>>();

        Ok(ExamWorkspaceSnapshot {
            course_id: course_id.to_string(),
            defaults: default_exam_defaults(),
            source_queue,
            queued_exams,
            ready_exams,
            failed_exams,
            history,
            review_notes,
            mastered_notes,
            summary: self.get_exam_workspace_summary(course_id)?,
        })
    }

    fn list_exam_source_queue_ids(&self, course_id: &str) -> Result<Vec<String>> {
        let mut statement = self.conn.prepare(
            "SELECT note_id FROM exam_source_queue WHERE course_id = ?1 ORDER BY queued_at DESC, note_id ASC",
        )?;
        let rows = statement
            .query_map(params![course_id], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn list_exam_source_queue_notes(&self, course_id: &str) -> Result<Vec<ExamSourceNote>> {
        let note_mastery = self.list_note_mastery_by_course(course_id)?;
        let note_lookup = self
            .list_notes(course_id)?
            .into_iter()
            .map(|note| (note.id.clone(), note))
            .collect::<HashMap<_, _>>();
        let concept_counts = self.list_concepts(course_id)?.into_iter().fold(
            HashMap::<String, usize>::new(),
            |mut acc, (note_id, _, _, _)| {
                *acc.entry(note_id).or_insert(0) += 1;
                acc
            },
        );
        let formula_counts = self.list_formulas(course_id)?.into_iter().fold(
            HashMap::<String, usize>::new(),
            |mut acc, (note_id, _, _)| {
                *acc.entry(note_id).or_insert(0) += 1;
                acc
            },
        );
        let ai_states = self.list_ai_note_states(course_id)?;

        let rows = self
            .list_exam_source_queue_ids(course_id)?
            .into_iter()
            .filter_map(|note_id| note_lookup.get(&note_id).cloned())
            .map(|note| {
                self.build_exam_source_note(
                    &note,
                    note_mastery.get(&note.id),
                    &concept_counts,
                    &formula_counts,
                    &ai_states,
                )
            })
            .collect::<Vec<_>>();
        Ok(rows)
    }

    fn list_note_mastery_by_course(
        &self,
        course_id: &str,
    ) -> Result<HashMap<String, StoredNoteMastery>> {
        let mut statement = self.conn.prepare(
            "SELECT note_id, mastery_state, last_accuracy FROM note_mastery_states WHERE course_id = ?1",
        )?;
        let rows = statement
            .query_map(params![course_id], |row| {
                Ok(StoredNoteMastery {
                    note_id: row.get(0)?,
                    mastery_state: note_mastery_state_from_str(&row.get::<_, String>(1)?)
                        .map_err(to_sql_error)?,
                    last_accuracy: row.get(2)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows
            .into_iter()
            .map(|row| (row.note_id.clone(), row))
            .collect())
    }

    fn note_mastery_for_note(&self, note_id: &str) -> Result<Option<StoredNoteMastery>> {
        self.conn
            .query_row(
                "SELECT note_id, mastery_state, last_accuracy FROM note_mastery_states WHERE note_id = ?1",
                params![note_id],
                |row| {
                    Ok(StoredNoteMastery {
                        note_id: row.get(0)?,
                        mastery_state: note_mastery_state_from_str(&row.get::<_, String>(1)?)
                            .map_err(to_sql_error)?,
                        last_accuracy: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    fn upsert_note_mastery(
        &self,
        note_id: &str,
        course_id: &str,
        mastery_state: NoteMasteryState,
        last_accuracy: Option<f64>,
    ) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO note_mastery_states (note_id, course_id, mastery_state, last_accuracy, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(note_id) DO UPDATE SET
              course_id = excluded.course_id,
              mastery_state = excluded.mastery_state,
              last_accuracy = excluded.last_accuracy,
              updated_at = excluded.updated_at
            "#,
            params![
                note_id,
                course_id,
                note_mastery_state_to_str(mastery_state),
                last_accuracy,
                now_string(),
            ],
        )?;
        Ok(())
    }

    fn list_exam_records(&self, course_id: &str) -> Result<Vec<StoredExamRecord>> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT id, course_id, title, preset, status, difficulty, question_count, source_note_count,
                   multiple_choice_count, short_answer_count, time_limit_minutes, source_note_ids_json,
                   instructions, summary, created_at, updated_at, generated_at, last_error, model
            FROM exams
            WHERE course_id = ?1
            ORDER BY created_at DESC
            "#,
        )?;
        let rows = statement
            .query_map(params![course_id], |row| self.read_exam_record_row(row))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn find_exam_record(&self, exam_id: &str) -> Result<Option<StoredExamRecord>> {
        self.conn
            .query_row(
                r#"
                SELECT id, course_id, title, preset, status, difficulty, question_count, source_note_count,
                       multiple_choice_count, short_answer_count, time_limit_minutes, source_note_ids_json,
                       instructions, summary, created_at, updated_at, generated_at, last_error, model
                FROM exams
                WHERE id = ?1
                "#,
                params![exam_id],
                |row| self.read_exam_record_row(row),
            )
            .optional()
            .map_err(Into::into)
    }

    fn find_generating_exam(&self, course_id: &str) -> Result<Option<StoredExamRecord>> {
        self.conn
            .query_row(
                r#"
                SELECT id, course_id, title, preset, status, difficulty, question_count, source_note_count,
                       multiple_choice_count, short_answer_count, time_limit_minutes, source_note_ids_json,
                       instructions, summary, created_at, updated_at, generated_at, last_error, model
                FROM exams
                WHERE course_id = ?1 AND status = 'generating'
                ORDER BY updated_at ASC, created_at ASC
                LIMIT 1
                "#,
                params![course_id],
                |row| self.read_exam_record_row(row),
            )
            .optional()
            .map_err(Into::into)
    }

    fn read_exam_record_row(&self, row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredExamRecord> {
        Ok(StoredExamRecord {
            id: row.get(0)?,
            course_id: row.get(1)?,
            title: row.get(2)?,
            preset: exam_preset_from_str(&row.get::<_, String>(3)?).map_err(to_sql_error)?,
            status: exam_status_from_str(&row.get::<_, String>(4)?).map_err(to_sql_error)?,
            difficulty: exam_difficulty_from_str(&row.get::<_, String>(5)?)
                .map_err(to_sql_error)?,
            question_count: row.get::<_, i64>(6)? as usize,
            source_note_count: row.get::<_, i64>(7)? as usize,
            multiple_choice_count: row.get::<_, i64>(8)? as usize,
            short_answer_count: row.get::<_, i64>(9)? as usize,
            time_limit_minutes: row.get::<_, i64>(10)? as usize,
            source_note_ids: from_json_vec::<String>(&row.get::<_, String>(11)?),
            instructions: row.get(12)?,
            summary: row.get(13)?,
            created_at: row.get(14)?,
            updated_at: row.get(15)?,
            generated_at: row.get(16)?,
            last_error: row.get(17)?,
            _model: row.get(18)?,
        })
    }

    fn load_exam_questions(
        &self,
        exam_id: &str,
        hide_answer_key: bool,
    ) -> Result<Vec<ExamQuestion>> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT id, exam_id, position, question_type, prompt, options_json, correct_answer,
                   explanation, source_note_id, source_note_title
            FROM exam_questions
            WHERE exam_id = ?1
            ORDER BY position ASC
            "#,
        )?;
        let rows = statement
            .query_map(params![exam_id], |row| {
                let question_type =
                    exam_question_type_from_str(&row.get::<_, String>(3)?).map_err(to_sql_error)?;
                Ok(ExamQuestion {
                    id: row.get(0)?,
                    exam_id: row.get(1)?,
                    index: row.get::<_, i64>(2)? as usize,
                    question_type,
                    prompt: row.get(4)?,
                    options: from_json_vec::<String>(&row.get::<_, String>(5)?),
                    source_note_id: row.get(8)?,
                    source_note_title: row.get(9)?,
                    expected_answer: if hide_answer_key {
                        None
                    } else {
                        Some(row.get(6)?)
                    },
                    explanation: if hide_answer_key {
                        None
                    } else {
                        Some(row.get(7)?)
                    },
                    user_answer: None,
                    is_correct: None,
                    feedback: None,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn list_exam_attempt_summaries(&self, course_id: &str) -> Result<Vec<ExamAttemptSummary>> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT exam_attempts.id, exam_attempts.exam_id, exams.title, exam_attempts.submitted_at,
                   exam_attempts.score_percent, exam_attempts.correct_count, exam_attempts.partial_count,
                   exam_attempts.incorrect_count
            FROM exam_attempts
            INNER JOIN exams ON exams.id = exam_attempts.exam_id
            WHERE exam_attempts.course_id = ?1
            ORDER BY exam_attempts.submitted_at DESC
            "#,
        )?;
        let rows = statement
            .query_map(params![course_id], |row| {
                Ok(ExamAttemptSummary {
                    id: row.get(0)?,
                    exam_id: row.get(1)?,
                    exam_title: row.get(2)?,
                    submitted_at: row.get(3)?,
                    score_percent: row.get(4)?,
                    correct_count: row.get::<_, i64>(5)? as usize,
                    partial_count: row.get::<_, i64>(6)? as usize,
                    incorrect_count: row.get::<_, i64>(7)? as usize,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn find_exam_attempt(&self, attempt_id: &str) -> Result<Option<StoredExamAttempt>> {
        self.conn
            .query_row(
                r#"
                SELECT course_id
                FROM exam_attempts
                WHERE exam_attempts.id = ?1
                "#,
                params![attempt_id],
                |row| {
                    Ok(StoredExamAttempt {
                        course_id: row.get(0)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    fn load_exam_source_notes_snapshot(
        &self,
        course_id: &str,
        note_ids: &[String],
    ) -> Result<Vec<ExamSourceNote>> {
        let note_mastery = self.list_note_mastery_by_course(course_id)?;
        let note_lookup = self
            .list_notes(course_id)?
            .into_iter()
            .map(|note| (note.id.clone(), note))
            .collect::<HashMap<_, _>>();
        let concept_counts = self.list_concepts(course_id)?.into_iter().fold(
            HashMap::<String, usize>::new(),
            |mut acc, (note_id, _, _, _)| {
                *acc.entry(note_id).or_insert(0) += 1;
                acc
            },
        );
        let formula_counts = self.list_formulas(course_id)?.into_iter().fold(
            HashMap::<String, usize>::new(),
            |mut acc, (note_id, _, _)| {
                *acc.entry(note_id).or_insert(0) += 1;
                acc
            },
        );
        let ai_states = self.list_ai_note_states(course_id)?;

        Ok(note_ids
            .iter()
            .filter_map(|note_id| note_lookup.get(note_id))
            .map(|note| {
                self.build_exam_source_note(
                    note,
                    note_mastery.get(&note.id),
                    &concept_counts,
                    &formula_counts,
                    &ai_states,
                )
            })
            .collect())
    }

    fn claim_next_exam_generation_job(
        &self,
        course_id: &str,
    ) -> Result<Option<StoredExamGenerationJob>> {
        let existing_generating = self.find_generating_exam(course_id)?;
        let exam = if let Some(exam) = existing_generating {
            exam
        } else {
            let queued = self
                .conn
                .query_row(
                    r#"
                    SELECT id, course_id, title, preset, status, difficulty, question_count, source_note_count,
                           multiple_choice_count, short_answer_count, time_limit_minutes, source_note_ids_json,
                           instructions, summary, created_at, updated_at, generated_at, last_error, model
                    FROM exams
                    WHERE course_id = ?1 AND status = 'queued'
                    ORDER BY created_at ASC
                    LIMIT 1
                    "#,
                    params![course_id],
                    |row| self.read_exam_record_row(row),
                )
                .optional()?;

            let Some(exam) = queued else {
                return Ok(None);
            };

            self.conn.execute(
                "UPDATE exams SET status = 'generating', updated_at = ?2, last_error = NULL WHERE id = ?1",
                params![&exam.id, now_string()],
            )?;
            self.find_exam_record(&exam.id)?
                .ok_or_else(|| anyhow!("queued exam disappeared before generation"))?
        };

        let notes = self.load_exam_generation_notes(&exam.course_id, &exam.source_note_ids)?;
        if notes.is_empty() {
            self.fail_exam_generation(&exam.id, "exam source queue snapshot is empty")?;
            return self.claim_next_exam_generation_job(course_id);
        }

        Ok(Some(StoredExamGenerationJob { exam, notes }))
    }

    fn load_exam_generation_notes(
        &self,
        course_id: &str,
        note_ids: &[String],
    ) -> Result<Vec<ai::ExamGenerationNoteInput>> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT id, title, relative_path, excerpt, headings_json, links_json
            FROM note_records
            WHERE course_id = ?1
            "#,
        )?;
        let rows = statement
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

        let concepts = self.list_concepts(course_id)?.into_iter().fold(
            HashMap::<String, Vec<String>>::new(),
            |mut acc, (note_id, name, _, _)| {
                acc.entry(note_id).or_default().push(name);
                acc
            },
        );
        let formulas = self.list_formulas(course_id)?.into_iter().fold(
            HashMap::<String, Vec<String>>::new(),
            |mut acc, (note_id, latex, _)| {
                acc.entry(note_id).or_default().push(latex);
                acc
            },
        );
        let row_lookup = rows
            .into_iter()
            .map(|row| (row.0.clone(), row))
            .collect::<HashMap<_, _>>();

        Ok(note_ids
            .iter()
            .filter_map(|note_id| row_lookup.get(note_id))
            .map(|row| ai::ExamGenerationNoteInput {
                note_id: row.0.clone(),
                title: row.1.clone(),
                relative_path: row.2.clone(),
                excerpt: row.3.clone(),
                headings: row.4.clone(),
                concepts: concepts.get(&row.0).cloned().unwrap_or_default(),
                formulas: formulas.get(&row.0).cloned().unwrap_or_default(),
                links: row.5.clone(),
            })
            .collect())
    }

    fn build_exam_source_note(
        &self,
        note: &StoredNote,
        mastery: Option<&StoredNoteMastery>,
        concept_counts: &HashMap<String, usize>,
        formula_counts: &HashMap<String, usize>,
        ai_states: &HashMap<String, StoredAiNoteState>,
    ) -> ExamSourceNote {
        ExamSourceNote {
            note_id: note.id.clone(),
            title: note.title.clone(),
            relative_path: note.relative_path.clone(),
            ai_status: current_ai_status(note, ai_states),
            mastery_state: mastery
                .map(|row| row.mastery_state)
                .unwrap_or(NoteMasteryState::Active),
            last_accuracy: mastery.and_then(|row| row.last_accuracy),
            concept_count: concept_counts.get(&note.id).copied().unwrap_or(0),
            formula_count: formula_counts.get(&note.id).copied().unwrap_or(0),
        }
    }

    fn build_exam_summary(&self, record: StoredExamRecord) -> Result<ExamSummary> {
        let latest_attempt = self
            .conn
            .query_row(
                r#"
                SELECT submitted_at, score_percent
                FROM exam_attempts
                WHERE exam_id = ?1
                ORDER BY submitted_at DESC
                LIMIT 1
                "#,
                params![&record.id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?)),
            )
            .optional()?;
        let attempt_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM exam_attempts WHERE exam_id = ?1",
            params![&record.id],
            |row| row.get::<_, i64>(0).map(|value| value as usize),
        )?;

        Ok(ExamSummary {
            id: record.id,
            course_id: record.course_id,
            title: record.title,
            preset: record.preset,
            status: record.status,
            difficulty: record.difficulty,
            question_count: record.question_count,
            source_note_count: record.source_note_count,
            multiple_choice_count: record.multiple_choice_count,
            short_answer_count: record.short_answer_count,
            time_limit_minutes: record.time_limit_minutes,
            created_at: record.created_at,
            updated_at: record.updated_at,
            generated_at: record.generated_at,
            latest_score_percent: latest_attempt.as_ref().map(|(_, score)| *score),
            latest_attempted_at: latest_attempt.map(|(submitted_at, _)| submitted_at),
            attempt_count,
            last_error: record.last_error,
        })
    }

    fn store_generated_exam(
        &self,
        exam_id: &str,
        instructions: &str,
        summary: &str,
        questions: &[ai::GeneratedExamQuestion],
        model: &str,
    ) -> Result<()> {
        self.conn.execute(
            "DELETE FROM exam_questions WHERE exam_id = ?1",
            params![exam_id],
        )?;

        let created_at = now_string();
        for (index, question) in questions.iter().enumerate() {
            self.conn.execute(
                r#"
                INSERT INTO exam_questions (
                  id, exam_id, position, question_type, prompt, options_json, correct_answer,
                  explanation, source_note_id, source_note_title, created_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                "#,
                params![
                    Uuid::new_v4().to_string(),
                    exam_id,
                    (index + 1) as i64,
                    exam_question_type_to_str(question.question_type),
                    question.prompt,
                    to_json(&question.options),
                    question.correct_answer,
                    question.explanation,
                    question.source_note_id,
                    question.source_note_title,
                    created_at,
                ],
            )?;
        }

        self.conn.execute(
            r#"
            UPDATE exams
            SET status = 'ready',
                instructions = ?2,
                summary = ?3,
                updated_at = ?4,
                generated_at = ?4,
                last_error = NULL,
                model = ?5
            WHERE id = ?1
            "#,
            params![exam_id, instructions, summary, created_at, model],
        )?;
        Ok(())
    }

    fn fail_exam_generation(&self, exam_id: &str, message: &str) -> Result<()> {
        self.conn.execute(
            r#"
            UPDATE exams
            SET status = 'failed',
                updated_at = ?2,
                generated_at = NULL,
                last_error = ?3
            WHERE id = ?1
            "#,
            params![exam_id, now_string(), truncate_error(message)],
        )?;
        Ok(())
    }

    fn build_note_suggestions(
        &self,
        course_id: &str,
        note_scores: &HashMap<String, (f64, usize)>,
    ) -> Result<Vec<ExamReviewSuggestion>> {
        let source_queue = self
            .list_exam_source_queue_ids(course_id)?
            .into_iter()
            .collect::<HashSet<_>>();
        let note_lookup = self
            .list_notes(course_id)?
            .into_iter()
            .map(|note| (note.id.clone(), note))
            .collect::<HashMap<_, _>>();
        let note_mastery = self.list_note_mastery_by_course(course_id)?;

        let mut suggestions = Vec::new();
        for (note_id, (earned, total)) in note_scores {
            let accuracy = round_percentage((earned / (*total).max(1) as f64) * 100.0);
            let current_state = note_mastery
                .get(note_id)
                .map(|row| row.mastery_state)
                .unwrap_or(NoteMasteryState::Active);
            let recommended_state = if accuracy >= 80.0 {
                NoteMasteryState::Mastered
            } else if accuracy < 60.0 {
                NoteMasteryState::Review
            } else {
                NoteMasteryState::Active
            };
            self.upsert_note_mastery(note_id, course_id, current_state, Some(accuracy))?;

            let note = note_lookup.get(note_id).ok_or_else(|| {
                anyhow!("note {note_id} not found while building exam suggestions")
            })?;
            suggestions.push(ExamReviewSuggestion {
                note_id: note_id.clone(),
                title: note.title.clone(),
                relative_path: note.relative_path.clone(),
                current_state,
                recommended_state,
                accuracy,
                reason: match recommended_state {
                    NoteMasteryState::Mastered => {
                        "High accuracy in this attempt. You can safely put this note away for now."
                            .to_string()
                    }
                    NoteMasteryState::Review => {
                        "Low accuracy in this attempt. Bring this note back into the learning queue."
                            .to_string()
                    }
                    NoteMasteryState::Active => {
                        "Mixed performance. Keep this note active until recall stabilizes."
                            .to_string()
                    }
                },
                currently_in_source_queue: source_queue.contains(note_id),
            });
        }

        suggestions.sort_by(|left, right| {
            left.accuracy
                .total_cmp(&right.accuracy)
                .then_with(|| left.title.cmp(&right.title))
        });
        Ok(suggestions)
    }
}

fn default_exam_defaults() -> ExamDefaults {
    ExamDefaults {
        preset: ExamPreset::Sprint,
        multiple_choice_count: 6,
        short_answer_count: 2,
        difficulty: ExamDifficulty::Mixed,
        time_limit_minutes: 10,
        generate_count: 1,
    }
}

fn normalize_exam_builder_input(mut request: ExamBuilderInput) -> Result<ExamBuilderInput> {
    request.time_limit_minutes = request.time_limit_minutes.max(5);
    request.generate_count = request.generate_count.clamp(1, 5);
    if request.multiple_choice_count + request.short_answer_count == 0 {
        bail!("exam must contain at least one question");
    }
    Ok(request)
}

fn build_exam_builder_input_from_record(record: &StoredExamRecord) -> ExamBuilderInput {
    ExamBuilderInput {
        course_id: record.course_id.clone(),
        preset: record.preset,
        multiple_choice_count: record.multiple_choice_count,
        short_answer_count: record.short_answer_count,
        difficulty: record.difficulty,
        time_limit_minutes: record.time_limit_minutes,
        generate_count: 1,
        title: Some(record.title.clone()),
    }
}

fn build_exam_details(
    exam: StoredExamRecord,
    questions: Vec<ExamQuestion>,
    source_notes: Vec<ExamSourceNote>,
) -> ExamDetails {
    ExamDetails {
        id: exam.id,
        course_id: exam.course_id,
        title: exam.title,
        preset: exam.preset,
        status: exam.status,
        difficulty: exam.difficulty,
        time_limit_minutes: exam.time_limit_minutes,
        question_count: exam.question_count,
        multiple_choice_count: exam.multiple_choice_count,
        short_answer_count: exam.short_answer_count,
        created_at: exam.created_at,
        updated_at: exam.updated_at,
        generated_at: exam.generated_at,
        instructions: exam.instructions,
        summary: exam.summary,
        questions,
        source_notes,
        last_error: exam.last_error,
    }
}

fn exam_preset_label(preset: ExamPreset) -> &'static str {
    match preset {
        ExamPreset::Sprint => "Sprint",
        ExamPreset::Mock => "Mock",
        ExamPreset::Final => "Final",
    }
}

fn empty_answer_for_question(question_type: ExamQuestionType) -> ExamAnswerValue {
    match question_type {
        ExamQuestionType::MultipleChoice | ExamQuestionType::ShortAnswer => {
            ExamAnswerValue::Text(String::new())
        }
    }
}

fn exam_preset_to_str(value: ExamPreset) -> &'static str {
    match value {
        ExamPreset::Sprint => "sprint",
        ExamPreset::Mock => "mock",
        ExamPreset::Final => "final",
    }
}

fn exam_preset_from_str(value: &str) -> Result<ExamPreset> {
    match value {
        "sprint" => Ok(ExamPreset::Sprint),
        "mock" => Ok(ExamPreset::Mock),
        "final" => Ok(ExamPreset::Final),
        _ => Err(anyhow!("unknown exam preset `{value}`")),
    }
}

fn exam_difficulty_to_str(value: ExamDifficulty) -> &'static str {
    match value {
        ExamDifficulty::Easy => "easy",
        ExamDifficulty::Mixed => "mixed",
        ExamDifficulty::Hard => "hard",
    }
}

fn exam_difficulty_from_str(value: &str) -> Result<ExamDifficulty> {
    match value {
        "easy" => Ok(ExamDifficulty::Easy),
        "mixed" => Ok(ExamDifficulty::Mixed),
        "hard" => Ok(ExamDifficulty::Hard),
        _ => Err(anyhow!("unknown exam difficulty `{value}`")),
    }
}

fn exam_status_to_str(value: ExamStatus) -> &'static str {
    match value {
        ExamStatus::Queued => "queued",
        ExamStatus::Generating => "generating",
        ExamStatus::Ready => "ready",
        ExamStatus::Failed => "failed",
    }
}

fn exam_status_from_str(value: &str) -> Result<ExamStatus> {
    match value {
        "queued" => Ok(ExamStatus::Queued),
        "generating" => Ok(ExamStatus::Generating),
        "ready" => Ok(ExamStatus::Ready),
        "failed" => Ok(ExamStatus::Failed),
        _ => Err(anyhow!("unknown exam status `{value}`")),
    }
}

fn exam_question_type_to_str(value: ExamQuestionType) -> &'static str {
    match value {
        ExamQuestionType::MultipleChoice => "multiple-choice",
        ExamQuestionType::ShortAnswer => "short-answer",
    }
}

fn exam_question_type_from_str(value: &str) -> Result<ExamQuestionType> {
    match value {
        "multiple-choice" => Ok(ExamQuestionType::MultipleChoice),
        "short-answer" => Ok(ExamQuestionType::ShortAnswer),
        _ => Err(anyhow!("unknown exam question type `{value}`")),
    }
}

fn note_mastery_state_to_str(value: NoteMasteryState) -> &'static str {
    match value {
        NoteMasteryState::Active => "active",
        NoteMasteryState::Review => "review",
        NoteMasteryState::Mastered => "mastered",
    }
}

fn note_mastery_state_from_str(value: &str) -> Result<NoteMasteryState> {
    match value {
        "active" => Ok(NoteMasteryState::Active),
        "review" => Ok(NoteMasteryState::Review),
        "mastered" => Ok(NoteMasteryState::Mastered),
        _ => Err(anyhow!("unknown note mastery state `{value}`")),
    }
}

fn exam_verdict_to_str(value: ExamVerdict) -> &'static str {
    match value {
        ExamVerdict::Correct => "correct",
        ExamVerdict::Partial => "partial",
        ExamVerdict::Incorrect => "incorrect",
    }
}

fn chat_scope_to_str(value: ChatScope) -> &'static str {
    match value {
        ChatScope::Course => "course",
        ChatScope::Vault => "vault",
    }
}

fn chat_scope_from_str(value: &str) -> Result<ChatScope> {
    match value {
        "course" => Ok(ChatScope::Course),
        "vault" => Ok(ChatScope::Vault),
        _ => Err(anyhow!("unknown chat scope `{value}`")),
    }
}

fn chat_message_role_to_str(value: ChatMessageRole) -> &'static str {
    match value {
        ChatMessageRole::User => "user",
        ChatMessageRole::Assistant => "assistant",
    }
}

fn chat_message_role_from_str(value: &str) -> Result<ChatMessageRole> {
    match value {
        "user" => Ok(ChatMessageRole::User),
        "assistant" => Ok(ChatMessageRole::Assistant),
        _ => Err(anyhow!("unknown chat message role `{value}`")),
    }
}

fn to_sql_error<E>(error: E) -> rusqlite::Error
where
    E: Into<anyhow::Error>,
{
    let error = error.into();
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            error.to_string(),
        )),
    )
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

fn aggregate_flashcard_summary(course_bundles: &[CourseStatisticsBundle]) -> FlashcardSummary {
    let total_cards = course_bundles
        .iter()
        .map(|bundle| bundle.flashcards.total_cards)
        .sum();
    let set_count = course_bundles
        .iter()
        .map(|bundle| bundle.flashcards.set_count)
        .sum();
    let latest = course_bundles
        .iter()
        .filter_map(|bundle| {
            bundle
                .flashcards
                .last_generated_at
                .as_ref()
                .map(|created_at| (created_at.clone(), bundle.flashcards.export_path.clone()))
        })
        .max_by(|left, right| left.0.cmp(&right.0));
    FlashcardSummary {
        set_count,
        total_cards,
        last_generated_at: latest.as_ref().map(|value| value.0.clone()),
        export_path: latest.and_then(|value| value.1),
    }
}

fn aggregate_revision_summary(course_bundles: &[CourseStatisticsBundle]) -> RevisionSummary {
    let total_items = course_bundles
        .iter()
        .map(|bundle| bundle.revision.item_count)
        .sum::<usize>();
    let latest = course_bundles
        .iter()
        .filter_map(|bundle| {
            bundle
                .revision
                .last_generated_at
                .as_ref()
                .map(|created_at| (created_at.clone(), bundle.revision.note_path.clone(), bundle.revision.item_count))
        })
        .max_by(|left, right| left.0.cmp(&right.0));
    RevisionSummary {
        last_generated_at: latest.as_ref().map(|value| value.0.clone()),
        note_path: latest.as_ref().and_then(|value| value.1.clone()),
        item_count: latest.map(|value| value.2).unwrap_or(total_items),
    }
}

fn build_attempt_history(exam_history: &[StatisticsExamPoint]) -> Vec<StatisticsValuePoint> {
    let mut grouped = BTreeMap::<String, usize>::new();
    for point in exam_history {
        let label = point.submitted_at.get(0..10).unwrap_or(&point.submitted_at).to_string();
        *grouped.entry(label).or_insert(0) += 1;
    }
    grouped
        .into_iter()
        .map(|(label, value)| StatisticsValuePoint {
            label,
            value: value as f64,
        })
        .collect()
}

fn build_formula_density_buckets(note_rows: &[StatisticsNoteRow]) -> Vec<StatisticsCountBucket> {
    let mut counts = [0usize; 4];
    for note in note_rows {
        match note.formula_count {
            0 => counts[0] += 1,
            1 => counts[1] += 1,
            2 => counts[2] += 1,
            _ => counts[3] += 1,
        }
    }
    vec![
        StatisticsCountBucket {
            label: "0 formulas".to_string(),
            count: counts[0],
        },
        StatisticsCountBucket {
            label: "1 formula".to_string(),
            count: counts[1],
        },
        StatisticsCountBucket {
            label: "2 formulas".to_string(),
            count: counts[2],
        },
        StatisticsCountBucket {
            label: "3+ formulas".to_string(),
            count: counts[3],
        },
    ]
}

fn build_strength_buckets(note_rows: &[StatisticsNoteRow]) -> Vec<StatisticsCountBucket> {
    let mut counts = [0usize; 4];
    for note in note_rows {
        match note.strength {
            value if value < 1.5 => counts[0] += 1,
            value if value < 3.0 => counts[1] += 1,
            value if value < 5.0 => counts[2] += 1,
            _ => counts[3] += 1,
        }
    }
    vec![
        StatisticsCountBucket {
            label: "Fragile".to_string(),
            count: counts[0],
        },
        StatisticsCountBucket {
            label: "Developing".to_string(),
            count: counts[1],
        },
        StatisticsCountBucket {
            label: "Stable".to_string(),
            count: counts[2],
        },
        StatisticsCountBucket {
            label: "Dense".to_string(),
            count: counts[3],
        },
    ]
}

fn sort_note_rows_by_strength(note_rows: &[StatisticsNoteRow], descending: bool) -> Vec<StatisticsNoteRow> {
    let mut rows = note_rows.to_vec();
    rows.sort_by(|left, right| {
        if descending {
            right.strength.total_cmp(&left.strength)
        } else {
            left.strength.total_cmp(&right.strength)
        }
        .then_with(|| left.title.cmp(&right.title))
    });
    rows.into_iter().take(8).collect()
}

fn sort_note_rows_by_links(note_rows: &[StatisticsNoteRow]) -> Vec<StatisticsNoteRow> {
    let mut rows = note_rows.to_vec();
    rows.sort_by(|left, right| {
        right
            .link_count
            .cmp(&left.link_count)
            .then_with(|| right.strength.total_cmp(&left.strength))
            .then_with(|| left.title.cmp(&right.title))
    });
    rows.into_iter().take(8).collect()
}

fn sort_note_rows_by_modified_at(note_rows: &[StatisticsNoteRow]) -> Vec<StatisticsNoteRow> {
    let mut rows = note_rows.to_vec();
    rows.sort_by(|left, right| left.modified_at.cmp(&right.modified_at).then_with(|| left.title.cmp(&right.title)));
    rows.into_iter().take(8).collect()
}

fn recent_exam_points(exam_history: &[StatisticsExamPoint]) -> Vec<StatisticsExamPoint> {
    let mut rows = exam_history.to_vec();
    rows.sort_by(|left, right| right.submitted_at.cmp(&left.submitted_at));
    rows.into_iter().take(8).collect()
}

fn weakest_exam_points(exam_history: &[StatisticsExamPoint]) -> Vec<StatisticsExamPoint> {
    let mut rows = exam_history.to_vec();
    rows.sort_by(|left, right| {
        left.score_percent
            .total_cmp(&right.score_percent)
            .then_with(|| right.submitted_at.cmp(&left.submitted_at))
    });
    rows.into_iter().take(8).collect()
}

fn build_vault_activity_summary(
    activity_buckets: &[VaultActivityBucket],
    note_rows: &[StatisticsNoteRow],
) -> VaultActivitySummary {
    let recent_notes = activity_buckets
        .iter()
        .find(|bucket| bucket.label == "0-7 days")
        .map(|bucket| bucket.note_count)
        .unwrap_or(0);
    let stale_notes = activity_buckets
        .iter()
        .find(|bucket| bucket.label == "90+ days")
        .map(|bucket| bucket.note_count)
        .unwrap_or(0);
    let unknown_notes = activity_buckets
        .iter()
        .find(|bucket| bucket.label == "Unknown")
        .map(|bucket| bucket.note_count)
        .unwrap_or(0);
    VaultActivitySummary {
        total_notes: note_rows.len(),
        recent_notes,
        stale_notes,
        unknown_notes,
        most_recent_modified_at: note_rows.iter().filter_map(|note| note.modified_at.clone()).max(),
    }
}

fn build_overview_highlights(
    course_rows: &[CourseStatisticsRow],
    git: Option<&GitAnalytics>,
) -> Vec<StatisticsHighlight> {
    let mut highlights = Vec::new();
    if let Some(best) = course_rows
        .iter()
        .max_by(|left, right| left.coverage_percentage.total_cmp(&right.coverage_percentage))
    {
        highlights.push(StatisticsHighlight {
            label: "Strongest coverage".to_string(),
            value: format!("{} {:.1}%", best.course_name, best.coverage_percentage),
            tone: "success".to_string(),
        });
    }
    if let Some(weakest) = course_rows.iter().max_by_key(|row| row.weak_note_count) {
        highlights.push(StatisticsHighlight {
            label: "Most fragile course".to_string(),
            value: format!("{} {} weak notes", weakest.course_name, weakest.weak_note_count),
            tone: "warning".to_string(),
        });
    }
    if let Some(active) = git.and_then(|value| value.course_activity.first()) {
        highlights.push(StatisticsHighlight {
            label: "Most edited course".to_string(),
            value: format!("{} {} commits", active.course_name, active.commit_count),
            tone: "accent".to_string(),
        });
    }
    highlights
}

fn is_stale_modified_at(value: Option<&str>) -> bool {
    let Some(value) = value else {
        return true;
    };
    DateTime::parse_from_rfc3339(value)
        .map(|parsed| (Utc::now() - parsed.with_timezone(&Utc)).num_days() > 90)
        .unwrap_or(true)
}

fn build_git_timeline(commits: &[GitCommitRecord], count_paths: bool) -> Vec<GitTimelinePoint> {
    let mut grouped = BTreeMap::<String, (usize, usize)>::new();
    for commit in commits {
        let bucket = commit.committed_at.get(0..7).unwrap_or(&commit.committed_at).to_string();
        let entry = grouped.entry(bucket).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += if count_paths {
            commit.paths.len()
        } else {
            commit.paths.iter().collect::<HashSet<_>>().len()
        };
    }
    grouped
        .into_iter()
        .map(|(bucket, (commit_count, changed_notes))| GitTimelinePoint {
            bucket,
            commit_count,
            changed_notes,
        })
        .collect()
}

fn build_git_course_activity_rows(
    commits: &[GitCommitRecord],
    courses: &[StoredCourse],
) -> Vec<GitCourseActivityRow> {
    let mut map = HashMap::<String, GitCourseActivityRow>::new();
    for course in courses {
        map.insert(
            course.id.clone(),
            GitCourseActivityRow {
                course_id: Some(course.id.clone()),
                course_name: course.name.clone(),
                folder: course.folder.clone(),
                commit_count: 0,
                changed_notes: 0,
                last_commit_at: None,
            },
        );
    }
    for commit in commits {
        for course in courses {
            let changed_notes = commit
                .paths
                .iter()
                .filter(|path| is_path_within_course(path, &course.folder))
                .count();
            if changed_notes == 0 {
                continue;
            }
            if let Some(row) = map.get_mut(&course.id) {
                row.commit_count += 1;
                row.changed_notes += changed_notes;
                if row
                    .last_commit_at
                    .as_ref()
                    .map(|value| value < &commit.committed_at)
                    .unwrap_or(true)
                {
                    row.last_commit_at = Some(commit.committed_at.clone());
                }
            }
        }
    }
    let mut rows = map.into_values().filter(|row| row.commit_count > 0).collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        right
            .commit_count
            .cmp(&left.commit_count)
            .then_with(|| right.changed_notes.cmp(&left.changed_notes))
            .then_with(|| left.course_name.cmp(&right.course_name))
    });
    rows
}

fn build_git_note_rows(
    commits: &[GitCommitRecord],
    note_map: &HashMap<String, (String, String, String, String)>,
) -> Vec<GitNoteActivityRow> {
    let mut counts = HashMap::<String, (usize, Option<String>)>::new();
    for commit in commits {
        for path in &commit.paths {
            let entry = counts.entry(path.clone()).or_insert((0, None));
            entry.0 += 1;
            if entry
                .1
                .as_ref()
                .map(|value| value < &commit.committed_at)
                .unwrap_or(true)
            {
                entry.1 = Some(commit.committed_at.clone());
            }
        }
    }
    let mut rows = counts
        .into_iter()
        .map(|(path, (change_count, last_commit_at))| {
            let mapped = note_map.get(&path);
            GitNoteActivityRow {
                note_id: mapped.map(|value| value.0.clone()),
                title: mapped
                    .map(|value| value.1.clone())
                    .unwrap_or_else(|| {
                        Path::new(&path)
                            .file_stem()
                            .and_then(|value| value.to_str())
                            .unwrap_or("Note")
                            .to_string()
                    }),
                relative_path: path,
                course_id: mapped.map(|value| value.2.clone()),
                course_name: mapped.map(|value| value.3.clone()),
                change_count,
                last_commit_at,
            }
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        right
            .change_count
            .cmp(&left.change_count)
            .then_with(|| right.last_commit_at.cmp(&left.last_commit_at))
            .then_with(|| left.title.cmp(&right.title))
    });
    rows.truncate(10);
    rows
}

fn is_path_within_course(path: &str, folder: &str) -> bool {
    let normalized_path = normalize_relative_path(path);
    let normalized_folder = normalize_relative_path(folder);
    normalized_path == normalized_folder
        || normalized_path.starts_with(&format!("{normalized_folder}/"))
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

fn truncate_excerpt(message: &str, limit: usize) -> String {
    if message.chars().count() <= limit {
        return message.to_string();
    }

    let mut truncated = message.chars().take(limit).collect::<String>();
    truncated.push_str("...");
    truncated
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

fn build_fts_query(normalized_query: &str) -> Option<String> {
    let tokens = normalized_query
        .split_whitespace()
        .filter(|token| token.len() >= 2)
        .map(|token| format!("{token}*"))
        .collect::<Vec<_>>();
    if tokens.is_empty() {
        None
    } else {
        Some(tokens.join(" OR "))
    }
}

fn summarize_thread_title(message: &str) -> String {
    let normalized = message.trim();
    if normalized.is_empty() {
        return "New conversation".to_string();
    }

    let title = normalized
        .split_whitespace()
        .take(7)
        .collect::<Vec<_>>()
        .join(" ");
    if title.chars().count() > 56 {
        truncate_excerpt(&title, 56)
    } else {
        title
    }
}

fn now_string() -> String {
    Utc::now().to_rfc3339()
}

fn system_time_to_rfc3339(value: SystemTime) -> String {
    let value: DateTime<Utc> = value.into();
    value.to_rfc3339()
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

fn build_note_chunk_id(note_id: &str, ordinal: usize) -> String {
    format!("chunk::{note_id}::{ordinal}")
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

fn build_course_formula_id(course_id: &str, normalized_formula: &str) -> String {
    format!(
        "formula::{course_id}::{}",
        normalized_formula.replace(' ', "_")
    )
}

fn build_formula_source_hash(
    note_ids: &[String],
    note_lookup: &HashMap<String, StoredNote>,
) -> String {
    let mut values = note_ids
        .iter()
        .filter_map(|note_id| note_lookup.get(note_id))
        .map(|note| format!("{}::{}", note.id, note.content_hash))
        .collect::<Vec<_>>();
    values.sort();
    hash_content(&values.join("|"))
}

fn build_note_chunks(content: &str, headings: &[String]) -> Vec<ParsedNoteChunk> {
    let mut chunks = Vec::new();
    let mut current_heading = headings
        .first()
        .cloned()
        .unwrap_or_else(|| "Overview".to_string());
    let mut heading_stack = Vec::<String>::new();
    let mut current_lines = Vec::<String>::new();
    let mut ordinal = 0usize;

    for raw_line in content.lines() {
        let trimmed = raw_line.trim();
        if trimmed.starts_with('#') {
            if let Some(chunk) = flush_chunk(&current_heading, &current_lines, ordinal) {
                chunks.push(chunk);
                ordinal += 1;
            }

            let level = trimmed.chars().take_while(|ch| *ch == '#').count();
            let heading = trimmed[level..].trim();
            if !heading.is_empty() {
                while heading_stack.len() >= level {
                    heading_stack.pop();
                }
                heading_stack.push(heading.to_string());
                current_heading = heading_stack.join(" / ");
            }
            current_lines.clear();
            continue;
        }

        if !trimmed.is_empty() {
            current_lines.push(trimmed.to_string());
        }
    }

    if let Some(chunk) = flush_chunk(&current_heading, &current_lines, ordinal) {
        chunks.push(chunk);
    }

    if chunks.is_empty() {
        if let Some(chunk) = flush_chunk(
            "Overview",
            &content
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>(),
            0,
        ) {
            chunks.push(chunk);
        }
    }

    chunks
}

fn flush_chunk(heading_path: &str, lines: &[String], ordinal: usize) -> Option<ParsedNoteChunk> {
    let text = lines.join(" ");
    let cleaned = truncate_excerpt(text.trim(), 700);
    if cleaned.is_empty() {
        return None;
    }

    Some(ParsedNoteChunk {
        heading_path: heading_path.to_string(),
        text: cleaned,
        ordinal,
    })
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

fn statistics_scope_to_str(scope: StatisticsScope) -> &'static str {
    match scope {
        StatisticsScope::Course => "course",
        StatisticsScope::Vault => "vault",
    }
}

fn read_statistics_snapshot_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<StatisticsSnapshotPoint> {
    Ok(StatisticsSnapshotPoint {
        captured_at: row.get(0)?,
        note_count: row.get::<_, i64>(1)? as usize,
        total_concepts: row.get::<_, i64>(2)? as usize,
        covered_concepts: row.get::<_, i64>(3)? as usize,
        coverage_percentage: row.get(4)?,
        edge_count: row.get::<_, i64>(5)? as usize,
        strong_links: row.get::<_, i64>(6)? as usize,
        inferred_links: row.get::<_, i64>(7)? as usize,
        isolated_notes: row.get::<_, i64>(8)? as usize,
        weak_note_count: row.get::<_, i64>(9)? as usize,
        formula_count: row.get::<_, i64>(10)? as usize,
        notes_with_formulas: row.get::<_, i64>(11)? as usize,
        average_note_strength: row.get(12)?,
        flashcard_set_count: row.get::<_, i64>(13)? as usize,
        flashcard_total_cards: row.get::<_, i64>(14)? as usize,
        revision_run_count: row.get::<_, i64>(15)? as usize,
        latest_revision_item_count: row.get::<_, i64>(16)? as usize,
        ai_ready_notes: row.get::<_, i64>(17)? as usize,
        ai_pending_notes: row.get::<_, i64>(18)? as usize,
        ai_failed_notes: row.get::<_, i64>(19)? as usize,
        ai_stale_notes: row.get::<_, i64>(20)? as usize,
        ai_missing_notes: row.get::<_, i64>(21)? as usize,
        exam_attempt_count: row.get::<_, i64>(22)? as usize,
        latest_exam_score: row.get(23)?,
        average_exam_score: row.get(24)?,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::Database;
    use crate::models::{
        AiSettingsInput, ApplyExamReviewActionsRequest, ChatScope, CourseConfigInput,
        CreateChatThreadRequest, ExamBuilderInput, ExamDifficulty, ExamPreset, ExamReviewAction,
        ExamSubmissionRequest, FlashcardGenerationRequest, GenerateFormulaBriefRequest,
        NoteMasteryState, RevisionNoteRequest, SendChatMessageRequest, StatisticsScope,
    };

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

    #[test]
    fn statistics_snapshots_and_source_metadata_persist() {
        let temp = tempdir().expect("temp dir");
        let vault_dir = temp.path().join("vault");
        let course_dir = vault_dir.join("math");
        fs::create_dir_all(&course_dir).expect("course dir");
        fs::write(course_dir.join("limits.md"), "# Limits\n\nScan me.\n").expect("write note");

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
                exam_date: None,
                revision_folder: None,
                flashcards_folder: None,
            })
            .expect("save course");

        database.run_scan().expect("first scan");
        fs::write(course_dir.join("limits.md"), "# Limits\n\nScan me twice.\n").expect("rewrite note");
        database.run_scan().expect("second scan");

        let source_modified_at = database
            .conn
            .query_row(
                "SELECT source_modified_at FROM note_records WHERE course_id = ?1 LIMIT 1",
                [course_id.as_str()],
                |row| row.get::<_, Option<String>>(0),
            )
            .expect("source modified timestamp");
        assert!(source_modified_at.is_some());

        let course_stats = database
            .get_statistics(StatisticsScope::Course, Some(course_id.clone()))
            .expect("course stats")
            .expect("course stats present");
        assert_eq!(course_stats.overview.history.len(), 2);
        assert_eq!(course_stats.overview.summary.note_count, 1);
        assert_eq!(
            course_stats
                .vault_activity
                .activity_buckets
                .iter()
                .map(|bucket| bucket.note_count)
                .sum::<usize>(),
            1
        );

        let vault_stats = database
            .get_statistics(StatisticsScope::Vault, None)
            .expect("vault stats")
            .expect("vault stats present");
        assert_eq!(vault_stats.overview.history.len(), 2);
        assert_eq!(vault_stats.overview.course_rows.len(), 1);
    }

    #[test]
    fn vault_statistics_aggregate_multiple_courses() {
        let temp = tempdir().expect("temp dir");
        let (math_id, physics_id) = setup_formula_chat_vault(temp.path());
        let db_path = temp.path().join("exam-os.sqlite");
        let database = Database::open(&db_path).expect("database");
        database
            .connect_vault(temp.path().join("vault").to_string_lossy().as_ref())
            .expect("connect vault");
        database
            .save_course_config(CourseConfigInput {
                id: Some(math_id.clone()),
                name: "Math".to_string(),
                folder: "math".to_string(),
                exam_date: None,
                revision_folder: None,
                flashcards_folder: None,
            })
            .expect("save math");
        database
            .save_course_config(CourseConfigInput {
                id: Some(physics_id.clone()),
                name: "Physics".to_string(),
                folder: "physics".to_string(),
                exam_date: None,
                revision_folder: None,
                flashcards_folder: None,
            })
            .expect("save physics");

        database.run_scan().expect("scan");

        let math_stats = database
            .get_statistics(StatisticsScope::Course, Some(math_id))
            .expect("math stats")
            .expect("math stats present");
        let physics_stats = database
            .get_statistics(StatisticsScope::Course, Some(physics_id))
            .expect("physics stats")
            .expect("physics stats present");
        let vault_stats = database
            .get_statistics(StatisticsScope::Vault, None)
            .expect("vault stats")
            .expect("vault stats present");

        assert_eq!(vault_stats.overview.course_rows.len(), 2);
        assert_eq!(
            vault_stats.overview.summary.note_count,
            math_stats.overview.summary.note_count + physics_stats.overview.summary.note_count
        );
        assert_eq!(vault_stats.overview.history.len(), 1);
    }

    #[test]
    fn statistics_fall_back_cleanly_without_git_repo() {
        let temp = tempdir().expect("temp dir");
        let (course_id, _) = setup_formula_chat_vault(temp.path());
        let db_path = temp.path().join("exam-os.sqlite");
        let database = Database::open(&db_path).expect("database");
        database
            .connect_vault(temp.path().join("vault").to_string_lossy().as_ref())
            .expect("connect vault");
        database
            .save_course_config(CourseConfigInput {
                id: Some(course_id),
                name: "Math".to_string(),
                folder: "math".to_string(),
                exam_date: None,
                revision_folder: None,
                flashcards_folder: None,
            })
            .expect("save course");
        database.run_scan().expect("scan");

        let stats = database
            .get_statistics(StatisticsScope::Vault, None)
            .expect("stats")
            .expect("stats present");

        assert!(!stats.git_available);
        assert!(stats.git.is_none());
        assert!(!stats.vault_activity.activity_buckets.is_empty());
    }

    #[test]
    fn exam_generation_grading_and_review_actions_persist() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("exam-os.sqlite");
        let course_id = setup_exam_course(temp.path());

        let database = Database::open(&db_path).expect("database");
        database
            .connect_vault(temp.path().join("vault").to_string_lossy().as_ref())
            .expect("connect vault");
        let saved_course_id = database
            .save_course_config(CourseConfigInput {
                id: Some(course_id.clone()),
                name: "Math".to_string(),
                folder: "math".to_string(),
                exam_date: Some("2026-07-01".to_string()),
                revision_folder: None,
                flashcards_folder: None,
            })
            .expect("save course");
        assert_eq!(saved_course_id, course_id);
        database.run_scan().expect("scan");
        let note_ids = database
            .get_dashboard(Some(course_id.clone()))
            .expect("dashboard")
            .expect("dashboard present")
            .notes
            .into_iter()
            .map(|note| note.id)
            .collect::<Vec<_>>();
        database
            .save_ai_settings(AiSettingsInput {
                base_url: "mock://exam".to_string(),
                model: "mock-model".to_string(),
                api_key: None,
                enabled: true,
                timeout_ms: Some(2_000),
            })
            .expect("save ai settings");

        let queued_workspace = database
            .add_exam_source_notes(&course_id, &note_ids[..2])
            .expect("queue source notes");
        assert_eq!(queued_workspace.source_queue.len(), 2);

        let queue_request = ExamBuilderInput {
            course_id: course_id.clone(),
            preset: ExamPreset::Sprint,
            multiple_choice_count: 2,
            short_answer_count: 1,
            difficulty: ExamDifficulty::Mixed,
            time_limit_minutes: 10,
            generate_count: 2,
            title: None,
        };
        let workspace_after_queue = database.queue_exams(queue_request).expect("queue exams");
        assert_eq!(workspace_after_queue.summary.queued_count, 2);

        database
            .run_exam_generation_queue(&course_id)
            .expect("generate exams");
        let workspace_ready = database
            .get_exam_workspace(Some(course_id.clone()))
            .expect("workspace")
            .expect("workspace present");
        assert_eq!(workspace_ready.summary.ready_count, 2);
        assert_eq!(workspace_ready.ready_exams.len(), 2);

        let exam_id = workspace_ready.ready_exams[0].id.clone();
        let public_details = database.get_exam_details(&exam_id).expect("exam details");
        assert!(public_details
            .questions
            .iter()
            .all(|question| question.expected_answer.is_none() && question.explanation.is_none()));

        let stored_questions = database
            .load_exam_questions(&exam_id, false)
            .expect("stored questions");
        let mastered_note_id = stored_questions[0].source_note_id.clone();
        let answers = stored_questions
            .iter()
            .map(|question| {
                let answer = if question.source_note_id == mastered_note_id {
                    question
                        .expected_answer
                        .clone()
                        .expect("stored answer key should be present")
                } else {
                    String::new()
                };
                crate::models::ExamAnswerInput {
                    question_id: question.id.clone(),
                    answer: crate::models::ExamAnswerValue::Text(answer),
                }
            })
            .collect();

        let attempt = database
            .submit_exam_attempt(ExamSubmissionRequest { exam_id, answers })
            .expect("submit exam");
        assert_eq!(attempt.note_suggestions.len(), 2);
        assert!(attempt
            .note_suggestions
            .iter()
            .any(|suggestion| suggestion.recommended_state == NoteMasteryState::Mastered));
        assert!(attempt
            .note_suggestions
            .iter()
            .any(|suggestion| suggestion.recommended_state == NoteMasteryState::Review));

        let review_actions = attempt
            .note_suggestions
            .iter()
            .map(|suggestion| ExamReviewAction {
                note_id: suggestion.note_id.clone(),
                next_state: suggestion.recommended_state,
                add_to_exam_queue: suggestion.recommended_state == NoteMasteryState::Review,
            })
            .collect::<Vec<_>>();
        let updated_workspace = database
            .apply_exam_review_actions(ApplyExamReviewActionsRequest {
                attempt_id: attempt.attempt_id.clone(),
                actions: review_actions,
            })
            .expect("apply review actions");

        assert_eq!(updated_workspace.summary.review_count, 1);
        assert_eq!(updated_workspace.summary.mastered_count, 1);
        assert_eq!(updated_workspace.history.len(), 1);
        assert!(updated_workspace
            .review_notes
            .iter()
            .any(|note| note.note_id != mastered_note_id));
        assert!(updated_workspace
            .mastered_notes
            .iter()
            .any(|note| note.note_id == mastered_note_id));

        let reopened = Database::open(&db_path).expect("reopen database");
        let reopened_workspace = reopened
            .get_exam_workspace(Some(course_id))
            .expect("reopened workspace")
            .expect("reopened workspace present");
        assert_eq!(reopened_workspace.summary.ready_count, 2);
        assert_eq!(reopened_workspace.summary.review_count, 1);
        assert_eq!(reopened_workspace.summary.mastered_count, 1);
        assert_eq!(reopened_workspace.history.len(), 1);
    }

    #[test]
    fn queue_exams_requires_enabled_ai() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("exam-os.sqlite");
        let course_id = setup_exam_course(temp.path());

        let database = Database::open(&db_path).expect("database");
        database
            .connect_vault(temp.path().join("vault").to_string_lossy().as_ref())
            .expect("connect vault");
        database
            .save_course_config(CourseConfigInput {
                id: Some(course_id.clone()),
                name: "Math".to_string(),
                folder: "math".to_string(),
                exam_date: None,
                revision_folder: None,
                flashcards_folder: None,
            })
            .expect("save course");
        database.run_scan().expect("scan");
        let note_ids = database
            .get_dashboard(Some(course_id.clone()))
            .expect("dashboard")
            .expect("dashboard present")
            .notes
            .into_iter()
            .map(|note| note.id)
            .collect::<Vec<_>>();
        database
            .add_exam_source_notes(&course_id, &note_ids[..1])
            .expect("queue source note");

        let error = database
            .queue_exams(ExamBuilderInput {
                course_id,
                preset: ExamPreset::Sprint,
                multiple_choice_count: 1,
                short_answer_count: 1,
                difficulty: ExamDifficulty::Mixed,
                time_limit_minutes: 10,
                generate_count: 1,
                title: None,
            })
            .expect_err("queueing should fail without AI");
        assert!(error.to_string().contains("enable AI"));
    }

    #[test]
    fn formula_workspace_and_brief_cache_invalidation_work() {
        let temp = tempdir().expect("temp dir");
        let vault_dir = temp.path().join("vault");
        let course_dir = vault_dir.join("math");
        fs::create_dir_all(&course_dir).expect("course dir");
        fs::write(
            course_dir.join("limits.md"),
            "# Limits\n\nDefinition of limits.\n\n$\\lim_{x \\to a} f(x) = L$\n",
        )
        .expect("write limits");
        fs::write(
            course_dir.join("continuity.md"),
            "# Continuity\n\nLimits connect to continuity.\n\n$\\lim_{x \\to a} f(x) = L$\n",
        )
        .expect("write continuity");

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
                exam_date: None,
                revision_folder: None,
                flashcards_folder: None,
            })
            .expect("save course");
        database.run_scan().expect("scan");
        database
            .save_ai_settings(AiSettingsInput {
                base_url: "mock://formula".to_string(),
                model: "mock-model".to_string(),
                api_key: None,
                enabled: true,
                timeout_ms: Some(2_000),
            })
            .expect("save ai");

        let formula_workspace = database
            .get_formula_workspace(Some(course_id.clone()))
            .expect("workspace")
            .expect("workspace present");
        assert_eq!(formula_workspace.formulas.len(), 1);
        assert_eq!(formula_workspace.summary.notes_with_formulas, 2);
        let formula_id = formula_workspace.formulas[0].id.clone();
        let details = database
            .get_formula_details(&formula_id, &course_id)
            .expect("formula details");
        assert_eq!(details.linked_notes.len(), 2);
        assert!(!details.chunks.is_empty());

        let first_brief = database
            .generate_formula_brief(GenerateFormulaBriefRequest {
                course_id: course_id.clone(),
                formula_id: formula_id.clone(),
                force: Some(false),
            })
            .expect("first brief");
        let cached_brief = database
            .generate_formula_brief(GenerateFormulaBriefRequest {
                course_id: course_id.clone(),
                formula_id: formula_id.clone(),
                force: Some(false),
            })
            .expect("cached brief");
        assert_eq!(first_brief.source_signature, cached_brief.source_signature);
        assert_eq!(first_brief.generated_at, cached_brief.generated_at);

        fs::write(
            course_dir.join("continuity.md"),
            "# Continuity\n\nLimits connect to continuity and local behavior.\n\n$\\lim_{x \\to a} f(x) = L$\n",
        )
        .expect("rewrite continuity");
        database.run_scan().expect("rescan");

        let refreshed_brief = database
            .generate_formula_brief(GenerateFormulaBriefRequest {
                course_id,
                formula_id,
                force: Some(false),
            })
            .expect("refreshed brief");
        assert_ne!(
            first_brief.source_signature,
            refreshed_brief.source_signature
        );
    }

    #[test]
    fn chat_threads_persist_and_scope_filters_citations() {
        let temp = tempdir().expect("temp dir");
        let vault_dir = temp.path().join("vault");
        let math_dir = vault_dir.join("math");
        let physics_dir = vault_dir.join("physics");
        fs::create_dir_all(&math_dir).expect("math dir");
        fs::create_dir_all(&physics_dir).expect("physics dir");
        fs::write(
            math_dir.join("limits.md"),
            "# Limits\n\nLimits describe how a function behaves near a point.\n",
        )
        .expect("write limits");
        fs::write(
            physics_dir.join("circuits.md"),
            "# Circuits\n\nCurrent through a resistor follows Ohm's law.\n",
        )
        .expect("write circuits");

        let db_path = temp.path().join("exam-os.sqlite");
        let database = Database::open(&db_path).expect("database");
        database
            .connect_vault(vault_dir.to_string_lossy().as_ref())
            .expect("connect vault");
        let math_course_id = database
            .save_course_config(CourseConfigInput {
                id: None,
                name: "Math".to_string(),
                folder: "math".to_string(),
                exam_date: None,
                revision_folder: None,
                flashcards_folder: None,
            })
            .expect("save math");
        let physics_course_id = database
            .save_course_config(CourseConfigInput {
                id: None,
                name: "Physics".to_string(),
                folder: "physics".to_string(),
                exam_date: None,
                revision_folder: None,
                flashcards_folder: None,
            })
            .expect("save physics");
        database
            .get_dashboard(Some(math_course_id.clone()))
            .expect("math dashboard");
        database.run_scan().expect("scan math");
        database
            .get_dashboard(Some(physics_course_id.clone()))
            .expect("physics dashboard");
        database.run_scan().expect("scan physics");
        database
            .save_ai_settings(AiSettingsInput {
                base_url: "mock://chat".to_string(),
                model: "mock-model".to_string(),
                api_key: None,
                enabled: true,
                timeout_ms: Some(2_000),
            })
            .expect("save ai");

        let course_thread = database
            .create_chat_thread(CreateChatThreadRequest {
                scope: ChatScope::Course,
                course_id: Some(math_course_id.clone()),
                title: None,
            })
            .expect("course thread");
        let course_reply = database
            .send_chat_message(SendChatMessageRequest {
                thread_id: course_thread.id.clone(),
                content: "What is a limit?".to_string(),
            })
            .expect("course reply");
        let last_course_message = course_reply.messages.last().expect("assistant message");
        assert!(last_course_message
            .citations
            .iter()
            .all(|citation| citation.course_id == math_course_id));

        let vault_thread = database
            .create_chat_thread(CreateChatThreadRequest {
                scope: ChatScope::Vault,
                course_id: None,
                title: None,
            })
            .expect("vault thread");
        let vault_reply = database
            .send_chat_message(SendChatMessageRequest {
                thread_id: vault_thread.id.clone(),
                content: "How does current behave in a resistor?".to_string(),
            })
            .expect("vault reply");
        let last_vault_message = vault_reply.messages.last().expect("assistant message");
        assert!(last_vault_message
            .citations
            .iter()
            .any(|citation| citation.course_id == physics_course_id));

        let reopened = Database::open(&db_path).expect("reopen");
        let reopened_thread = reopened
            .get_chat_thread(&course_thread.id)
            .expect("reopened thread");
        assert!(reopened_thread.messages.len() >= 2);
        let thread_summaries = reopened
            .list_chat_threads(ChatScope::Course, Some(math_course_id))
            .expect("thread summaries");
        assert_eq!(thread_summaries.len(), 1);
    }

    #[test]
    fn formula_and_chat_require_enabled_ai() {
        let temp = tempdir().expect("temp dir");
        let db_path = temp.path().join("exam-os.sqlite");
        let (math_id, _physics_id) = setup_formula_chat_vault(temp.path());

        let database = Database::open(&db_path).expect("database");
        database
            .connect_vault(temp.path().join("vault").to_string_lossy().as_ref())
            .expect("connect vault");
        database
            .save_course_config(CourseConfigInput {
                id: Some(math_id.clone()),
                name: "Math".to_string(),
                folder: "math".to_string(),
                exam_date: None,
                revision_folder: None,
                flashcards_folder: None,
            })
            .expect("save math");
        database
            .save_course_config(CourseConfigInput {
                id: Some("course::physics".to_string()),
                name: "Physics".to_string(),
                folder: "physics".to_string(),
                exam_date: None,
                revision_folder: None,
                flashcards_folder: None,
            })
            .expect("save physics");
        database
            .get_dashboard(Some(math_id.clone()))
            .expect("select math dashboard");
        database.run_scan().expect("scan");
        database
            .save_ai_settings(AiSettingsInput {
                base_url: "mock://chat".to_string(),
                model: "mock-model".to_string(),
                api_key: None,
                enabled: false,
                timeout_ms: Some(2_000),
            })
            .expect("disable ai");

        let formula_workspace = database
            .get_formula_workspace(Some(math_id.clone()))
            .expect("formula workspace")
            .expect("formula workspace present");
        let formula_id = formula_workspace
            .formulas
            .first()
            .expect("formula entry")
            .id
            .clone();
        let formula_error = database
            .generate_formula_brief(GenerateFormulaBriefRequest {
                course_id: math_id.clone(),
                formula_id,
                force: Some(false),
            })
            .expect_err("formula brief should fail when AI is disabled");
        assert!(formula_error.to_string().contains("enable AI"));

        let thread = database
            .create_chat_thread(CreateChatThreadRequest {
                scope: ChatScope::Course,
                course_id: Some(math_id),
                title: None,
            })
            .expect("create thread");
        let chat_error = database
            .send_chat_message(SendChatMessageRequest {
                thread_id: thread.id,
                content: "What is a limit?".to_string(),
            })
            .expect_err("chat should fail when AI is disabled");
        assert!(chat_error.to_string().contains("enable AI"));
    }

    fn setup_exam_course(root: &std::path::Path) -> String {
        let vault_dir = root.join("vault");
        let course_dir = vault_dir.join("math");
        fs::create_dir_all(&course_dir).expect("course dir");
        fs::write(
            course_dir.join("limits.md"),
            "# Limits\n\nLimits describe how a function behaves near a point.\n[[continuity]]",
        )
        .expect("write limits");
        fs::write(
            course_dir.join("continuity.md"),
            "# Continuity\n\nContinuity means no jumps in the function graph.\n[[limits]]",
        )
        .expect("write continuity");
        fs::write(
            course_dir.join("derivatives.md"),
            "# Derivatives\n\nDerivatives measure instantaneous rate of change.",
        )
        .expect("write derivatives");

        "course::math".to_string()
    }

    fn setup_formula_chat_vault(root: &std::path::Path) -> (String, String) {
        let vault_dir = root.join("vault");
        let math_dir = vault_dir.join("math");
        let physics_dir = vault_dir.join("physics");
        fs::create_dir_all(&math_dir).expect("math dir");
        fs::create_dir_all(&physics_dir).expect("physics dir");

        fs::write(
            math_dir.join("limits.md"),
            "# Limits\n\n## Definition\n\nLimits describe how a function behaves near a point.\n$\\lim_{x\\to a} f(x)$\n\n## Connections\n\nSee [[continuity]].",
        )
        .expect("write limits");
        fs::write(
            math_dir.join("continuity.md"),
            "# Continuity\n\n## Limit test\n\nContinuity connects to the same symbolic limit.\n$\\lim_{x\\to a} f(x)$\n\n## Rule\n\nA function is continuous when local behavior stays stable.",
        )
        .expect("write continuity");
        fs::write(
            math_dir.join("derivatives.md"),
            "# Derivatives\n\n## Rate of change\n\nThe derivative captures the instantaneous rate of change.\n$f'(x)$",
        )
        .expect("write derivatives");
        fs::write(
            physics_dir.join("velocity.md"),
            "# Velocity\n\n## Definition\n\nVelocity measures how position changes over time.\n$v = d / t$\n\n## Units\n\nUse displacement over time to calculate velocity.",
        )
        .expect("write velocity");

        ("course::math".to_string(), "course::physics".to_string())
    }
}
