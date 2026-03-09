import { invoke, isTauri } from "@tauri-apps/api/core";
import type {
  AiCourseSummary,
  AiNoteInsight,
  AiSettingsInput,
  CourseConfigInput,
  DashboardData,
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
  connectVaultMock,
  deleteCourseMock,
  disconnectVaultMock,
  generateFlashcardsMock,
  generateNoteAiInsightMock,
  generateRevisionNoteMock,
  getDashboardMock,
  getNoteDetailsMock,
  loadWorkspaceMock,
  runScanMock,
  saveAiSettingsMock,
  saveCourseConfigMock,
  startAiEnrichmentMock,
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
