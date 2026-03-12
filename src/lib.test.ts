import { describe, expect, it } from "vitest";

import {
  DEFAULT_EXAM_DEFAULTS,
  clampPercent,
  createCourseDraft,
  createExamBuilderInput,
  getErrorMessage,
  shortenPath,
} from "./lib";

describe("createExamBuilderInput", () => {
  it("copies the selected defaults into the builder payload", () => {
    expect(createExamBuilderInput("course-1", DEFAULT_EXAM_DEFAULTS)).toEqual({
      courseId: "course-1",
      preset: "sprint",
      multipleChoiceCount: 6,
      shortAnswerCount: 2,
      difficulty: "mixed",
      timeLimitMinutes: 10,
      generateCount: 1,
    });
  });
});

describe("createCourseDraft", () => {
  it("returns empty defaults when no course is provided", () => {
    expect(createCourseDraft()).toEqual({
      name: "",
      folder: "",
      examDate: "",
      revisionFolder: "Revision",
      flashcardsFolder: "Flashcards",
    });
  });

  it("preserves an existing course configuration", () => {
    expect(
      createCourseDraft({
        id: "course-2",
        name: "Algorithms",
        folder: "Semester 2/Algorithms",
        examDate: "2026-06-01",
        revisionFolder: "Revision Sheets",
        flashcardsFolder: "Cards",
        noteCount: 12,
        conceptCount: 7,
        formulaCount: 4,
        coverage: 82,
        weakNoteCount: 3,
      }),
    ).toEqual({
      id: "course-2",
      name: "Algorithms",
      folder: "Semester 2/Algorithms",
      examDate: "2026-06-01",
      revisionFolder: "Revision Sheets",
      flashcardsFolder: "Cards",
    });
  });
});

describe("clampPercent", () => {
  it("limits values to the 0-100 range and rounds them", () => {
    expect(clampPercent(-10)).toBe(0);
    expect(clampPercent(42.4)).toBe(42);
    expect(clampPercent(42.5)).toBe(43);
    expect(clampPercent(120)).toBe(100);
  });
});

describe("shortenPath", () => {
  it("keeps short paths untouched", () => {
    expect(shortenPath("Vault\\Semester 2\\Notes")).toBe("Vault\\Semester 2\\Notes");
  });

  it("collapses long paths to the first and last segments", () => {
    expect(shortenPath("Vault\\Semester 2\\Algorithms\\Week 1\\Notes.md")).toBe(
      "Vault\\Semester 2\\…\\Week 1\\Notes.md",
    );
  });
});

describe("getErrorMessage", () => {
  it("returns plain Error messages", () => {
    expect(getErrorMessage(new Error("Something went wrong"))).toBe("Something went wrong");
  });

  it("prettifies provider rate-limit errors", () => {
    const message =
      'provider rejected request (429): {"error":{"message":"Upstream timeout","metadata":{"provider_name":"OpenRouter","raw":"Rate limit exceeded."}}}';

    expect(getErrorMessage(message)).toBe("OpenRouter rate-limited this request. Rate limit exceeded.");
  });

  it("falls back to a generic message for unknown values", () => {
    expect(getErrorMessage({ code: 500 })).toBe("Unexpected runtime error.");
  });
});