import type { ExamBuilderInput, ExamPreset } from "./types";

export const EXAM_PRESETS: Record<
  ExamPreset,
  Omit<ExamBuilderInput, "courseId" | "generateCount" | "title" | "preset">
> = {
  sprint: {
    multipleChoiceCount: 6,
    shortAnswerCount: 2,
    difficulty: "mixed",
    timeLimitMinutes: 10,
  },
  mock: {
    multipleChoiceCount: 14,
    shortAnswerCount: 6,
    difficulty: "mixed",
    timeLimitMinutes: 25,
  },
  final: {
    multipleChoiceCount: 24,
    shortAnswerCount: 16,
    difficulty: "hard",
    timeLimitMinutes: 45,
  },
};

export function createExamBuilderDraft(courseId: string, preset: ExamPreset = "sprint"): ExamBuilderInput {
  return {
    courseId,
    preset,
    ...EXAM_PRESETS[preset],
    generateCount: 1,
    title: null,
  };
}

export function applyExamPreset(input: ExamBuilderInput, preset: ExamPreset): ExamBuilderInput {
  return {
    ...input,
    preset,
    ...EXAM_PRESETS[preset],
  };
}

export function examQuestionCount(input: Pick<ExamBuilderInput, "multipleChoiceCount" | "shortAnswerCount">) {
  return input.multipleChoiceCount + input.shortAnswerCount;
}

export function clampGenerateCount(value: number) {
  return Math.max(1, Math.min(5, Math.round(value || 1)));
}
