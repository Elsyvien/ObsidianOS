import type {
  AiSettingsInput,
  CourseConfig,
  ExamBuilderInput,
  ExamDefaults,
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
  timeoutMs: 120000,
};

export const DEFAULT_EXAM_DEFAULTS: ExamDefaults = {
  preset: "sprint",
  multipleChoiceCount: 6,
  shortAnswerCount: 2,
  difficulty: "mixed",
  timeLimitMinutes: 10,
  generateCount: 1,
};

export function createExamBuilderInput(courseId: string, defaults: ExamDefaults): ExamBuilderInput {
  return {
    courseId,
    preset: defaults.preset,
    multipleChoiceCount: defaults.multipleChoiceCount,
    shortAnswerCount: defaults.shortAnswerCount,
    difficulty: defaults.difficulty,
    timeLimitMinutes: defaults.timeLimitMinutes,
    generateCount: defaults.generateCount,
  };
}

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
  const message =
    error instanceof Error
      ? error.message
      : typeof error === "string"
        ? error
        : "Unexpected runtime error.";

  return prettifyErrorMessage(message);
}

function prettifyErrorMessage(message: string) {
  const normalized = message.replace(/\s+/g, " ").trim();
  const providerMatch = normalized.match(/^provider rejected request \(([^)]+)\):\s*(.+)$/i);

  if (providerMatch) {
    const [, status, rawPayload] = providerMatch;
    const parsed = parseProviderErrorPayload(rawPayload);
    const providerName = parsed?.error?.metadata?.provider_name?.trim();
    const detail =
      parsed?.error?.metadata?.raw?.trim() ||
      parsed?.error?.message?.trim() ||
      rawPayload.trim();
    const cleanedDetail = detail.replace(/\s+/g, " ").trim();

    if (status.startsWith("429")) {
      return providerName
        ? `${providerName} rate-limited this request. ${cleanedDetail}`
        : `The AI provider rate-limited this request. ${cleanedDetail}`;
    }

    return providerName
      ? `${providerName} rejected the request (${status}). ${cleanedDetail}`
      : `Provider rejected the request (${status}). ${cleanedDetail}`;
  }

  return normalized;
}

function parseProviderErrorPayload(payload: string) {
  try {
    return JSON.parse(payload) as {
      error?: {
        message?: string;
        metadata?: {
          raw?: string;
          provider_name?: string;
        };
      };
    };
  } catch {
    return null;
  }
}
