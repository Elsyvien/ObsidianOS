import { invoke, isTauri } from "@tauri-apps/api/core";
import type {
  ApplyExamReviewActionsRequest,
  AiCourseSummary,
  AiNoteInsight,
  AiSettingsInput,
  CourseConfigInput,
  DashboardData,
  ExamAttemptResult,
  ExamBuilderInput,
  ExamDetails,
  ExamSubmissionRequest,
  ExamWorkspaceSnapshot,
  FlashcardGenerationRequest,
  FlashcardGenerationResult,
  NoteDetails,
  RevisionNoteRequest,
  RevisionNoteResult,
  ScanReport,
  ValidationResult,
  WorkspaceSnapshot,
} from "./types";
import {
  addExamSourceNotesMock,
  applyExamReviewActionsMock,
  clearExamSourceQueueMock,
  connectVaultMock,
  deleteCourseMock,
  disconnectVaultMock,
  getExamDetailsMock,
  getExamWorkspaceMock,
  generateFlashcardsMock,
  generateNoteAiInsightMock,
  generateRevisionNoteMock,
  queueExamsMock,
  removeExamSourceNotesMock,
  getDashboardMock,
  getNoteDetailsMock,
  loadWorkspaceMock,
  runScanMock,
  saveAiSettingsMock,
  saveCourseConfigMock,
  startAiEnrichmentMock,
  submitExamAttemptMock,
  validateAiSettingsMock,
} from "./mockApi";

type RunScanResponse = {
  workspace: WorkspaceSnapshot;
  report: ScanReport;
};

const isTauriRuntime = () => isTauri();

export const getRuntimeMode = () => (isTauriRuntime() ? "tauri" : "browser-preview");
export const runtimeMode = getRuntimeMode();

export function loadWorkspace() {
  if (!isTauriRuntime()) return loadWorkspaceMock();
  return invoke<WorkspaceSnapshot>("load_workspace");
}

export function connectVault(vaultPath: string) {
  if (!isTauriRuntime()) return connectVaultMock(vaultPath);
  return invoke<WorkspaceSnapshot>("connect_vault", { vaultPath });
}

export async function chooseVaultDirectory() {
  if (!isTauriRuntime()) {
    throw new Error("Vault browsing is only available in the desktop app. The browser preview uses demo data.");
  }
  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({
    directory: true,
    multiple: false,
    title: "Select Obsidian Vault",
  });

  return typeof selected === "string" ? selected : null;
}

export function disconnectVault() {
  if (!isTauriRuntime()) return disconnectVaultMock();
  return invoke<WorkspaceSnapshot>("disconnect_vault");
}

export function saveCourseConfig(input: CourseConfigInput) {
  if (!isTauriRuntime()) return saveCourseConfigMock(input);
  return invoke<WorkspaceSnapshot>("save_course_config", { input });
}

export function deleteCourse(courseId: string) {
  if (!isTauriRuntime()) return deleteCourseMock(courseId);
  return invoke<WorkspaceSnapshot>("delete_course", { courseId });
}

export function runScan() {
  if (!isTauriRuntime()) return runScanMock();
  return invoke<RunScanResponse>("run_scan");
}

export function getDashboard(courseId: string | null) {
  if (!isTauriRuntime()) return getDashboardMock(courseId);
  return invoke<DashboardData | null>("get_dashboard", { courseId });
}

export function getNoteDetails(noteId: string) {
  if (!isTauriRuntime()) return getNoteDetailsMock(noteId);
  return invoke<NoteDetails>("get_note_details", { noteId });
}

export function generateNoteAiInsight(noteId: string) {
  if (!isTauriRuntime()) return generateNoteAiInsightMock(noteId);
  return invoke<AiNoteInsight>("generate_note_ai_insight", { noteId });
}

export function startAiEnrichment(courseId: string, force = false) {
  if (!isTauriRuntime()) return startAiEnrichmentMock(courseId, force);
  return invoke<AiCourseSummary>("start_ai_enrichment", { courseId, force });
}

export function saveAiSettings(input: AiSettingsInput) {
  if (!isTauriRuntime()) return saveAiSettingsMock(input);
  return invoke<WorkspaceSnapshot>("save_ai_settings", { input });
}

export function validateAiSettings(input: AiSettingsInput) {
  if (!isTauriRuntime()) return validateAiSettingsMock(input);
  return invoke<ValidationResult>("validate_ai_settings", { input });
}

export function generateFlashcards(request: FlashcardGenerationRequest) {
  if (!isTauriRuntime()) return generateFlashcardsMock(request);
  return invoke<FlashcardGenerationResult>("generate_flashcards", { request });
}

export function generateRevisionNote(request: RevisionNoteRequest) {
  if (!isTauriRuntime()) return generateRevisionNoteMock(request);
  return invoke<RevisionNoteResult>("generate_revision_note", { request });
}

export function getExamWorkspace(courseId: string | null) {
  if (!isTauriRuntime()) return getExamWorkspaceMock(courseId);
  return invoke<ExamWorkspaceSnapshot | null>("get_exam_workspace", { courseId });
}

export function addExamSourceNotes(courseId: string, noteIds: string[]) {
  if (!isTauriRuntime()) return addExamSourceNotesMock(courseId, noteIds);
  return invoke<ExamWorkspaceSnapshot>("add_exam_source_notes", { courseId, noteIds });
}

export function removeExamSourceNotes(courseId: string, noteIds: string[]) {
  if (!isTauriRuntime()) return removeExamSourceNotesMock(courseId, noteIds);
  return invoke<ExamWorkspaceSnapshot>("remove_exam_source_notes", { courseId, noteIds });
}

export function clearExamSourceQueue(courseId: string) {
  if (!isTauriRuntime()) return clearExamSourceQueueMock(courseId);
  return invoke<ExamWorkspaceSnapshot>("clear_exam_source_queue", { courseId });
}

export function queueExams(request: ExamBuilderInput) {
  if (!isTauriRuntime()) return queueExamsMock(request);
  return invoke<ExamWorkspaceSnapshot>("queue_exams", { request });
}

export function getExamDetails(examId: string) {
  if (!isTauriRuntime()) return getExamDetailsMock(examId);
  return invoke<ExamDetails>("get_exam_details", { examId });
}

export function submitExamAttempt(request: ExamSubmissionRequest) {
  if (!isTauriRuntime()) return submitExamAttemptMock(request);
  return invoke<ExamAttemptResult>("submit_exam_attempt", { request });
}

export function applyExamReviewActions(request: ApplyExamReviewActionsRequest) {
  if (!isTauriRuntime()) return applyExamReviewActionsMock(request);
  return invoke<ExamWorkspaceSnapshot>("apply_exam_review_actions", { request });
}
