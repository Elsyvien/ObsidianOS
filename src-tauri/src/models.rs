use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultConfig {
    pub vault_path: String,
    pub connected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    pub base_url: String,
    pub model: String,
    pub enabled: bool,
    pub timeout_ms: u64,
    pub has_api_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSettingsInput {
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub enabled: bool,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseConfigInput {
    pub id: Option<String>,
    pub name: String,
    pub folder: String,
    pub exam_date: Option<String>,
    pub revision_folder: Option<String>,
    pub flashcards_folder: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseConfig {
    pub id: String,
    pub name: String,
    pub folder: String,
    pub exam_date: Option<String>,
    pub revision_folder: String,
    pub flashcards_folder: String,
    pub note_count: usize,
    pub concept_count: usize,
    pub formula_count: usize,
    pub coverage: f64,
    pub weak_note_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Countdown {
    pub exam_date: Option<String>,
    pub days_remaining: Option<i64>,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageStats {
    pub total_concepts: usize,
    pub covered_concepts: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphStats {
    pub note_count: usize,
    pub edge_count: usize,
    pub strong_links: usize,
    pub inferred_links: usize,
    pub isolated_notes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeakNote {
    pub note_id: String,
    pub title: String,
    pub relative_path: String,
    pub score: f64,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConceptMetric {
    pub name: String,
    pub note_count: usize,
    pub support_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormulaMetric {
    pub latex: String,
    pub note_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashcardSummary {
    pub set_count: usize,
    pub total_cards: usize,
    pub last_generated_at: Option<String>,
    pub export_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionSummary {
    pub last_generated_at: Option<String>,
    pub note_path: Option<String>,
    pub item_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSummary {
    pub id: String,
    pub title: String,
    pub relative_path: String,
    pub link_count: usize,
    pub concept_count: usize,
    pub formula_count: usize,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardData {
    pub generated_at: String,
    pub vault_path: String,
    pub selected_course_id: Option<String>,
    pub countdown: Countdown,
    pub coverage: CoverageStats,
    pub graph: GraphStats,
    pub weak_notes: Vec<WeakNote>,
    pub top_concepts: Vec<ConceptMetric>,
    pub formulas: Vec<FormulaMetric>,
    pub flashcards: FlashcardSummary,
    pub revision: RevisionSummary,
    pub notes: Vec<NoteSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteDetails {
    pub id: String,
    pub title: String,
    pub relative_path: String,
    pub excerpt: String,
    pub headings: Vec<String>,
    pub links: Vec<String>,
    pub tags: Vec<String>,
    pub concepts: Vec<String>,
    pub formulas: Vec<String>,
    pub suggestions: Vec<String>,
    pub ai_insight: Option<AiNoteInsight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiNoteInsight {
    pub note_id: String,
    pub summary: String,
    pub takeaways: Vec<String>,
    pub exam_questions: Vec<String>,
    pub connection_opportunities: Vec<String>,
    pub generated_at: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatus {
    pub last_scan_at: Option<String>,
    pub note_count: usize,
    pub changed_count: usize,
    pub removed_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanReport {
    pub scanned_notes: usize,
    pub changed_notes: usize,
    pub unchanged_notes: usize,
    pub removed_notes: usize,
    pub generated_edges: usize,
    pub generated_weak_links: usize,
    pub scanned_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashcardGenerationRequest {
    pub course_id: String,
    pub note_ids: Vec<String>,
    pub flashcards_folder: Option<String>,
    pub export_csv: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashcardGenerationResult {
    pub markdown_path: String,
    pub csv_path: Option<String>,
    pub card_count: usize,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionNoteRequest {
    pub course_id: String,
    pub revision_folder: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionNoteResult {
    pub note_path: String,
    pub generated_at: String,
    pub item_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSnapshot {
    pub vault: Option<VaultConfig>,
    pub ai_settings: Option<AiSettings>,
    pub courses: Vec<CourseConfig>,
    pub selected_course_id: Option<String>,
    pub dashboard: Option<DashboardData>,
    pub scan_status: Option<ScanStatus>,
}
