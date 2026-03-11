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
pub struct FormulaSummary {
    pub id: String,
    pub latex: String,
    pub normalized_latex: String,
    pub note_count: usize,
    pub source_note_ids: Vec<String>,
    pub source_note_titles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormulaWorkspaceSummary {
    pub formula_count: usize,
    pub notes_with_formulas: usize,
    pub formula_mentions: usize,
    pub briefed_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormulaWorkspaceSnapshot {
    pub course_id: String,
    pub course_name: String,
    pub generated_at: String,
    pub formulas: Vec<FormulaSummary>,
    pub summary: FormulaWorkspaceSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormulaLinkedNote {
    pub note_id: String,
    pub title: String,
    pub relative_path: String,
    pub excerpt: String,
    pub headings: Vec<String>,
    pub related_concepts: Vec<String>,
    pub formula_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteChunkPreview {
    pub chunk_id: String,
    pub note_id: String,
    pub note_title: String,
    pub relative_path: String,
    pub heading_path: String,
    pub text: String,
    pub ordinal: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FormulaCoach {
    pub meaning: String,
    pub symbol_breakdown: Vec<String>,
    pub use_cases: Vec<String>,
    pub pitfalls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FormulaPractice {
    pub recall_prompts: Vec<String>,
    pub short_answer_drills: Vec<String>,
    pub multiple_choice_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FormulaDerivation {
    pub assumptions: Vec<String>,
    pub intuition: String,
    pub outline: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormulaBrief {
    pub formula_id: String,
    pub coach: FormulaCoach,
    pub practice: FormulaPractice,
    pub derivation: FormulaDerivation,
    pub generated_at: String,
    pub model: String,
    pub source_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormulaDetails {
    pub course_id: String,
    pub id: String,
    pub latex: String,
    pub normalized_latex: String,
    pub note_count: usize,
    pub source_note_ids: Vec<String>,
    pub source_note_titles: Vec<String>,
    pub linked_notes: Vec<FormulaLinkedNote>,
    pub chunks: Vec<NoteChunkPreview>,
    pub related_concepts: Vec<String>,
    pub headings: Vec<String>,
    pub brief: Option<FormulaBrief>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateFormulaBriefRequest {
    pub course_id: String,
    pub formula_id: String,
    pub force: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChatScope {
    Course,
    Vault,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChatMessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatCitation {
    pub chunk_id: String,
    pub note_id: String,
    pub note_title: String,
    pub relative_path: String,
    pub heading_path: String,
    pub excerpt: String,
    pub course_id: String,
    pub course_name: String,
    pub relevance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub thread_id: String,
    pub role: ChatMessageRole,
    pub content: String,
    pub created_at: String,
    pub citations: Vec<ChatCitation>,
    pub used_fallback: bool,
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatThreadSummary {
    pub id: String,
    pub scope: ChatScope,
    pub course_id: Option<String>,
    pub course_name: Option<String>,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
    pub last_message_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatThreadDetails {
    pub id: String,
    pub scope: ChatScope,
    pub course_id: Option<String>,
    pub course_name: Option<String>,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChatThreadRequest {
    pub scope: ChatScope,
    pub course_id: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendChatMessageRequest {
    pub thread_id: String,
    pub content: String,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExamPreset {
    Sprint,
    Mock,
    Final,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExamDifficulty {
    Easy,
    Mixed,
    Hard,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExamStatus {
    Queued,
    Generating,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExamQuestionType {
    #[serde(rename = "multiple-choice")]
    MultipleChoice,
    #[serde(rename = "short-answer")]
    ShortAnswer,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NoteMasteryState {
    Active,
    Review,
    Mastered,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExamVerdict {
    Correct,
    Partial,
    Incorrect,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ExamAnswerValue {
    Text(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamDefaults {
    pub preset: ExamPreset,
    pub multiple_choice_count: usize,
    pub short_answer_count: usize,
    pub difficulty: ExamDifficulty,
    pub time_limit_minutes: usize,
    pub generate_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamBuilderInput {
    pub course_id: String,
    pub preset: ExamPreset,
    pub multiple_choice_count: usize,
    pub short_answer_count: usize,
    pub difficulty: ExamDifficulty,
    pub time_limit_minutes: usize,
    pub generate_count: usize,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamSourceNote {
    pub note_id: String,
    pub title: String,
    pub relative_path: String,
    pub ai_status: String,
    pub mastery_state: NoteMasteryState,
    pub last_accuracy: Option<f64>,
    pub concept_count: usize,
    pub formula_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamSummary {
    pub id: String,
    pub course_id: String,
    pub title: String,
    pub preset: ExamPreset,
    pub status: ExamStatus,
    pub difficulty: ExamDifficulty,
    pub question_count: usize,
    pub source_note_count: usize,
    pub multiple_choice_count: usize,
    pub short_answer_count: usize,
    pub time_limit_minutes: usize,
    pub created_at: String,
    pub updated_at: String,
    pub generated_at: Option<String>,
    pub latest_score_percent: Option<f64>,
    pub latest_attempted_at: Option<String>,
    pub attempt_count: usize,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamQuestion {
    pub id: String,
    pub exam_id: String,
    pub index: usize,
    #[serde(rename = "type")]
    pub question_type: ExamQuestionType,
    pub prompt: String,
    pub options: Vec<String>,
    pub source_note_id: String,
    pub source_note_title: String,
    pub expected_answer: Option<String>,
    pub explanation: Option<String>,
    pub user_answer: Option<ExamAnswerValue>,
    pub is_correct: Option<bool>,
    pub feedback: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamDetails {
    pub id: String,
    pub course_id: String,
    pub title: String,
    pub preset: ExamPreset,
    pub status: ExamStatus,
    pub difficulty: ExamDifficulty,
    pub time_limit_minutes: usize,
    pub question_count: usize,
    pub multiple_choice_count: usize,
    pub short_answer_count: usize,
    pub created_at: String,
    pub updated_at: String,
    pub generated_at: Option<String>,
    pub instructions: String,
    pub summary: String,
    pub questions: Vec<ExamQuestion>,
    pub source_notes: Vec<ExamSourceNote>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamAnswerInput {
    pub question_id: String,
    pub answer: ExamAnswerValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamSubmissionRequest {
    pub exam_id: String,
    pub answers: Vec<ExamAnswerInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamQuestionResult {
    pub question_id: String,
    pub index: usize,
    #[serde(rename = "type")]
    pub question_type: ExamQuestionType,
    pub prompt: String,
    pub options: Vec<String>,
    pub source_note_id: String,
    pub source_note_title: String,
    pub user_answer: ExamAnswerValue,
    pub verdict: ExamVerdict,
    pub is_correct: bool,
    pub expected_answer: String,
    pub explanation: String,
    pub feedback: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamReviewSuggestion {
    pub note_id: String,
    pub title: String,
    pub relative_path: String,
    pub current_state: NoteMasteryState,
    pub recommended_state: NoteMasteryState,
    pub accuracy: f64,
    pub reason: String,
    pub currently_in_source_queue: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamAttemptResult {
    pub exam_id: String,
    pub attempt_id: String,
    pub submitted_at: String,
    pub score_percent: f64,
    pub correct_count: usize,
    pub partial_count: usize,
    pub incorrect_count: usize,
    pub overall_feedback: String,
    pub question_results: Vec<ExamQuestionResult>,
    pub note_suggestions: Vec<ExamReviewSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamReviewAction {
    pub note_id: String,
    pub next_state: NoteMasteryState,
    pub add_to_exam_queue: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyExamReviewActionsRequest {
    pub attempt_id: String,
    pub actions: Vec<ExamReviewAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamAttemptSummary {
    pub id: String,
    pub exam_id: String,
    pub exam_title: String,
    pub submitted_at: String,
    pub score_percent: f64,
    pub correct_count: usize,
    pub partial_count: usize,
    pub incorrect_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExamWorkspaceSummary {
    pub source_queue_count: usize,
    pub queued_count: usize,
    pub generating_count: usize,
    pub ready_count: usize,
    pub failed_count: usize,
    pub review_count: usize,
    pub mastered_count: usize,
    pub latest_attempted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamWorkspaceSnapshot {
    pub course_id: String,
    pub defaults: ExamDefaults,
    pub source_queue: Vec<ExamSourceNote>,
    pub queued_exams: Vec<ExamSummary>,
    pub ready_exams: Vec<ExamSummary>,
    pub failed_exams: Vec<ExamSummary>,
    pub history: Vec<ExamAttemptSummary>,
    pub review_notes: Vec<ExamSourceNote>,
    pub mastered_notes: Vec<ExamSourceNote>,
    pub summary: ExamWorkspaceSummary,
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
    pub ai_status: String,
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
    pub exams: ExamWorkspaceSummary,
    pub notes: Vec<NoteSummary>,
    pub ai: AiCourseSummary,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StatisticsScope {
    Course,
    Vault,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsOverview {
    pub note_count: usize,
    pub total_concepts: usize,
    pub covered_concepts: usize,
    pub coverage_percentage: f64,
    pub edge_count: usize,
    pub strong_links: usize,
    pub inferred_links: usize,
    pub isolated_notes: usize,
    pub weak_note_count: usize,
    pub formula_count: usize,
    pub notes_with_formulas: usize,
    pub average_note_strength: f64,
    pub flashcard_set_count: usize,
    pub flashcard_total_cards: usize,
    pub revision_run_count: usize,
    pub latest_revision_item_count: usize,
    pub ai_ready_notes: usize,
    pub ai_pending_notes: usize,
    pub ai_failed_notes: usize,
    pub ai_stale_notes: usize,
    pub ai_missing_notes: usize,
    pub exam_attempt_count: usize,
    pub latest_exam_score: Option<f64>,
    pub average_exam_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsSnapshotPoint {
    pub captured_at: String,
    pub note_count: usize,
    pub total_concepts: usize,
    pub covered_concepts: usize,
    pub coverage_percentage: f64,
    pub edge_count: usize,
    pub strong_links: usize,
    pub inferred_links: usize,
    pub isolated_notes: usize,
    pub weak_note_count: usize,
    pub formula_count: usize,
    pub notes_with_formulas: usize,
    pub average_note_strength: f64,
    pub flashcard_set_count: usize,
    pub flashcard_total_cards: usize,
    pub revision_run_count: usize,
    pub latest_revision_item_count: usize,
    pub ai_ready_notes: usize,
    pub ai_pending_notes: usize,
    pub ai_failed_notes: usize,
    pub ai_stale_notes: usize,
    pub ai_missing_notes: usize,
    pub exam_attempt_count: usize,
    pub latest_exam_score: Option<f64>,
    pub average_exam_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultActivityBucket {
    pub label: String,
    pub note_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsCountBucket {
    pub label: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsValuePoint {
    pub label: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsHighlight {
    pub label: String,
    pub value: String,
    pub tone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsExamPoint {
    pub submitted_at: String,
    pub exam_id: String,
    pub exam_title: String,
    pub score_percent: f64,
    pub course_id: Option<String>,
    pub course_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseStatisticsRow {
    pub course_id: String,
    pub course_name: String,
    pub note_count: usize,
    pub coverage_percentage: f64,
    pub edge_count: usize,
    pub weak_note_count: usize,
    pub formula_count: usize,
    pub average_note_strength: f64,
    pub flashcard_total_cards: usize,
    pub revision_run_count: usize,
    pub ai_ready_notes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsNoteRow {
    pub note_id: String,
    pub title: String,
    pub relative_path: String,
    pub course_id: Option<String>,
    pub course_name: Option<String>,
    pub ai_status: String,
    pub strength: f64,
    pub link_count: usize,
    pub concept_count: usize,
    pub formula_count: usize,
    pub modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsKnowledgeSummary {
    pub total_concepts: usize,
    pub covered_concepts: usize,
    pub coverage_percentage: f64,
    pub formula_count: usize,
    pub notes_with_formulas: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsNotesSummary {
    pub note_count: usize,
    pub average_note_strength: f64,
    pub weak_note_count: usize,
    pub isolated_notes: usize,
    pub stale_note_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsExamsSummary {
    pub attempt_count: usize,
    pub latest_score: Option<f64>,
    pub average_score: Option<f64>,
    pub review_count: usize,
    pub mastered_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsAiSummary {
    pub ready_notes: usize,
    pub pending_notes: usize,
    pub failed_notes: usize,
    pub stale_notes: usize,
    pub missing_notes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsOutputsSummary {
    pub flashcard_set_count: usize,
    pub flashcard_total_cards: usize,
    pub revision_run_count: usize,
    pub latest_revision_item_count: usize,
    pub latest_flashcard_export: Option<String>,
    pub latest_revision_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultActivitySummary {
    pub total_notes: usize,
    pub recent_notes: usize,
    pub stale_notes: usize,
    pub unknown_notes: usize,
    pub most_recent_modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitTimelinePoint {
    pub bucket: String,
    pub commit_count: usize,
    pub changed_notes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitCourseActivityRow {
    pub course_id: Option<String>,
    pub course_name: String,
    pub folder: String,
    pub commit_count: usize,
    pub changed_notes: usize,
    pub last_commit_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitNoteActivityRow {
    pub note_id: Option<String>,
    pub title: String,
    pub relative_path: String,
    pub course_id: Option<String>,
    pub course_name: Option<String>,
    pub change_count: usize,
    pub last_commit_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitCommitItem {
    pub sha: String,
    pub summary: String,
    pub author_name: String,
    pub committed_at: String,
    pub changed_notes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitSummary {
    pub repo_root: String,
    pub total_markdown_commits: usize,
    pub total_markdown_file_changes: usize,
    pub last_commit_at: Option<String>,
    pub recent_commit_count: usize,
    pub active_days_30: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsOverviewSection {
    pub summary: StatisticsOverview,
    pub history: Vec<StatisticsSnapshotPoint>,
    pub course_rows: Vec<CourseStatisticsRow>,
    pub highlights: Vec<StatisticsHighlight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsKnowledgeSection {
    pub summary: StatisticsKnowledgeSummary,
    pub history: Vec<StatisticsSnapshotPoint>,
    pub top_concepts: Vec<ConceptMetric>,
    pub top_formulas: Vec<FormulaMetric>,
    pub formula_density_buckets: Vec<StatisticsCountBucket>,
    pub course_rows: Vec<CourseStatisticsRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsNotesSection {
    pub summary: StatisticsNotesSummary,
    pub history: Vec<StatisticsSnapshotPoint>,
    pub strength_buckets: Vec<StatisticsCountBucket>,
    pub activity_buckets: Vec<VaultActivityBucket>,
    pub weakest_notes: Vec<StatisticsNoteRow>,
    pub most_connected_notes: Vec<StatisticsNoteRow>,
    pub stalest_notes: Vec<StatisticsNoteRow>,
    pub most_changed_notes: Vec<GitNoteActivityRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsExamsSection {
    pub summary: StatisticsExamsSummary,
    pub score_history: Vec<StatisticsExamPoint>,
    pub attempt_history: Vec<StatisticsValuePoint>,
    pub verdict_mix: Vec<StatisticsCountBucket>,
    pub mastery_distribution: Vec<StatisticsCountBucket>,
    pub recent_exams: Vec<StatisticsExamPoint>,
    pub weakest_attempts: Vec<StatisticsExamPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsAiSection {
    pub summary: StatisticsAiSummary,
    pub history: Vec<StatisticsSnapshotPoint>,
    pub status_breakdown: Vec<StatisticsCountBucket>,
    pub failed_notes: Vec<StatisticsNoteRow>,
    pub stale_notes: Vec<StatisticsNoteRow>,
    pub course_rows: Vec<CourseStatisticsRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsOutputsSection {
    pub summary: StatisticsOutputsSummary,
    pub history: Vec<StatisticsSnapshotPoint>,
    pub output_mix: Vec<StatisticsCountBucket>,
    pub latest_flashcards: FlashcardSummary,
    pub latest_revision: RevisionSummary,
    pub course_rows: Vec<CourseStatisticsRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsVaultActivitySection {
    pub summary: VaultActivitySummary,
    pub activity_buckets: Vec<VaultActivityBucket>,
    pub recent_notes: Vec<StatisticsNoteRow>,
    pub course_activity: Vec<CourseStatisticsRow>,
    pub git_timeline: Vec<GitTimelinePoint>,
    pub git_course_activity: Vec<GitCourseActivityRow>,
    pub git_top_notes: Vec<GitNoteActivityRow>,
    pub recent_commits: Vec<GitCommitItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsGitSection {
    pub summary: GitSummary,
    pub commit_timeline: Vec<GitTimelinePoint>,
    pub churn_timeline: Vec<GitTimelinePoint>,
    pub course_activity: Vec<GitCourseActivityRow>,
    pub top_notes: Vec<GitNoteActivityRow>,
    pub recent_commits: Vec<GitCommitItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsResponse {
    pub scope: StatisticsScope,
    pub generated_at: String,
    pub course_id: Option<String>,
    pub course_name: Option<String>,
    pub git_available: bool,
    pub git_error: Option<String>,
    pub overview: StatisticsOverviewSection,
    pub knowledge: StatisticsKnowledgeSection,
    pub notes: StatisticsNotesSection,
    pub exams: StatisticsExamsSection,
    pub ai: StatisticsAiSection,
    pub outputs: StatisticsOutputsSection,
    pub vault_activity: StatisticsVaultActivitySection,
    pub git: Option<StatisticsGitSection>,
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
    pub ai_status: String,
    pub ai_error: Option<String>,
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
pub struct AiCourseSummary {
    pub status: String,
    pub total_notes: usize,
    pub ready_notes: usize,
    pub pending_notes: usize,
    pub failed_notes: usize,
    pub stale_notes: usize,
    pub missing_notes: usize,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub updated_at: Option<String>,
    pub model: Option<String>,
    pub summary: Option<String>,
    pub revision_priorities: Vec<String>,
    pub weak_spots: Vec<String>,
    pub next_actions: Vec<String>,
    pub last_error: Option<String>,
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
