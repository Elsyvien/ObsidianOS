import type {
  AiSettingsInput,
  CourseConfig,
  WorkspaceSnapshot,
} from "./types";

export type BannerTone = "neutral" | "success" | "error";

export type BannerState = {
  tone: BannerTone;
  title: string;
  detail?: string;
} | null;

export type CourseDraft = {
  id?: string;
  name: string;
  folder: string;
  examDate: string;
  revisionFolder: string;
  flashcardsFolder: string;
};

export const EMPTY_WORKSPACE: WorkspaceSnapshot = {
  vault: null,
  aiSettings: null,
  courses: [],
  selectedCourseId: null,
  dashboard: null,
  scanStatus: null,
};

export const DEFAULT_AI_SETTINGS: AiSettingsInput = {
  baseUrl: "https://openrouter.ai/api/v1",
  model: "openrouter/free",
  apiKey: "",
  enabled: false,
  timeoutMs: 30000,
};

export function createCourseDraft(course?: CourseConfig): CourseDraft {
  return {
    id: course?.id,
    name: course?.name ?? "",
    folder: course?.folder ?? "",
    examDate: course?.examDate ?? "",
    revisionFolder: course?.revisionFolder ?? "Revision",
    flashcardsFolder: course?.flashcardsFolder ?? "Flashcards",
  };
}

export function clampPercent(value: number) {
  return Math.max(0, Math.min(100, Math.round(value)));
}

export function formatDate(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(new Date(value));
}

export function formatDateTime(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value));
}

export function shortenPath(path: string) {
  const segments = path.split(/[\\/]/).filter(Boolean);
  if (segments.length <= 3) {
    return path;
  }

  return `${segments.slice(0, 2).join("\\")}\\…\\${segments.slice(-2).join("\\")}`;
}

export function getErrorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === "string") {
    return error;
  }

  return "Unexpected runtime error.";
}
