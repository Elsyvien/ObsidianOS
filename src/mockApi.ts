import type {
  ApplyExamReviewActionsRequest,
  AiCourseSummary,
  AiNoteInsight,
  AiSettingsInput,
  ChatCitation,
  ChatMessage,
  ChatScope,
  ChatThreadDetails,
  ChatThreadSummary,
  CourseConfig,
  CourseConfigInput,
  CreateChatThreadRequest,
  DashboardData,
  ExamAttemptResult,
  ExamAttemptSummary,
  ExamBuilderInput,
  ExamDefaults,
  ExamDetails,
  ExamQuestion,
  ExamQuestionResult,
  ExamReviewSuggestion,
  ExamSourceNote,
  ExamSubmissionRequest,
  ExamSummary,
  ExamWorkspaceSnapshot,
  FlashcardGenerationRequest,
  FlashcardGenerationResult,
  GitCourseActivityRow,
  GitTimelinePoint,
  FormulaBrief,
  FormulaDetails,
  FormulaLinkedNote,
  FormulaSummary,
  FormulaWorkspaceSnapshot,
  GenerateFormulaBriefRequest,
  NoteChunkPreview,
  NoteMasteryState,
  NoteDetails,
  NoteSummary,
  RevisionNoteRequest,
  RevisionNoteResult,
  ScanReport,
  SendChatMessageRequest,
  StatisticsCountBucket,
  StatisticsExamPoint,
  StatisticsOverview,
  StatisticsNoteRow,
  StatisticsResponse,
  StatisticsScope,
  StatisticsSnapshotPoint,
  StatisticsValuePoint,
  ValidationResult,
  VaultActivityBucket,
  WorkspaceSnapshot,
} from "./types";

type MockState = {
  workspace: WorkspaceSnapshot;
  notesByCourse: Record<string, NoteDetails[]>;
  flashcardRuns: Record<string, FlashcardGenerationResult[]>;
  revisionRuns: Record<string, RevisionNoteResult[]>;
  examSourceQueue: Record<string, string[]>;
  examsByCourse: Record<string, MockExamRecord[]>;
  noteMastery: Record<string, NoteMasteryState>;
  noteAccuracy: Record<string, number | null>;
  formulaBriefs: Record<string, FormulaBrief>;
  chatThreads: MockChatThread[];
};

type MockExamRecord = ExamDetails & {
  latestScorePercent: number | null;
  attempts: ExamAttemptResult[];
  sourceNoteIds: string[];
  gradingKeys: Record<
    string,
    {
      expectedAnswer: string;
      keyword: string;
      explanation: string;
    }
  >;
};

type DemoBlueprint = {
  coverage: DashboardData["coverage"];
  formulas: DashboardData["formulas"];
  graph: DashboardData["graph"];
  notes: NoteDetails[];
  topConcepts: DashboardData["topConcepts"];
};

type MockChatThread = ChatThreadDetails;

const PREVIEW_VAULT_PATH = "Preview\\Sample Workspace";
const now = () => new Date().toISOString();

const state: MockState = {
  workspace: {
    vault: null,
    aiSettings: null,
    courses: [],
    selectedCourseId: null,
    dashboard: null,
    scanStatus: null,
  },
  notesByCourse: {},
  flashcardRuns: {},
  revisionRuns: {},
  examSourceQueue: {},
  examsByCourse: {},
  noteMastery: {},
  noteAccuracy: {},
  formulaBriefs: {},
  chatThreads: [],
};

const EXAM_PRESET_DEFAULTS: Record<ExamDefaults["preset"], Omit<ExamDefaults, "preset">> = {
  sprint: {
    multipleChoiceCount: 6,
    shortAnswerCount: 2,
    difficulty: "mixed",
    timeLimitMinutes: 10,
    generateCount: 1,
  },
  mock: {
    multipleChoiceCount: 14,
    shortAnswerCount: 6,
    difficulty: "mixed",
    timeLimitMinutes: 25,
    generateCount: 1,
  },
  final: {
    multipleChoiceCount: 24,
    shortAnswerCount: 16,
    difficulty: "hard",
    timeLimitMinutes: 45,
    generateCount: 1,
  },
};

const DEFAULT_EXAM_DEFAULTS: ExamDefaults = {
  preset: "sprint",
  ...EXAM_PRESET_DEFAULTS.sprint,
};

export async function loadWorkspaceMock() {
  ensureDemoWorkspace();
  return clone(state.workspace);
}

export async function connectVaultMock(vaultPath: string) {
  ensureDemoWorkspace();
  state.workspace.vault = { vaultPath, connectedAt: now() };
  state.workspace.dashboard = state.workspace.selectedCourseId
    ? buildDashboard(state.workspace.selectedCourseId)
    : null;
  return clone(state.workspace);
}

export async function disconnectVaultMock() {
  state.workspace = {
    vault: null,
    aiSettings: state.workspace.aiSettings,
    courses: [],
    selectedCourseId: null,
    dashboard: null,
    scanStatus: null,
  };
  state.notesByCourse = {};
  state.flashcardRuns = {};
  state.revisionRuns = {};
  state.examSourceQueue = {};
  state.examsByCourse = {};
  state.noteMastery = {};
  state.noteAccuracy = {};
  state.formulaBriefs = {};
  state.chatThreads = [];
  return clone(state.workspace);
}

export async function saveCourseConfigMock(input: CourseConfigInput) {
  ensureDemoWorkspace();
  const id = input.id ?? `course-${crypto.randomUUID()}`;
  const previous = state.workspace.courses.find((course) => course.id === id);
  const blueprint = buildBlueprint({
    id,
    name: input.name,
    folder: input.folder,
    examDate: input.examDate ?? null,
    revisionFolder: input.revisionFolder ?? "Revision",
    flashcardsFolder: input.flashcardsFolder ?? "Flashcards",
    noteCount: previous?.noteCount ?? 0,
    conceptCount: previous?.conceptCount ?? 0,
    formulaCount: previous?.formulaCount ?? 0,
    coverage: previous?.coverage ?? 0,
    weakNoteCount: previous?.weakNoteCount ?? 0,
  });

  const nextCourse: CourseConfig = {
    id,
    name: input.name,
    folder: input.folder,
    examDate: input.examDate ?? null,
    revisionFolder: input.revisionFolder ?? "Revision",
    flashcardsFolder: input.flashcardsFolder ?? "Flashcards",
    noteCount: blueprint.notes.length,
    conceptCount: blueprint.coverage.totalConcepts,
    formulaCount: blueprint.formulas.length,
    coverage: blueprint.coverage.percentage,
    weakNoteCount: Math.min(2, blueprint.notes.length),
  };

  state.workspace.courses = [
    ...state.workspace.courses.filter((course) => course.id !== id),
    nextCourse,
  ].sort((left, right) => left.name.localeCompare(right.name));
  state.notesByCourse[id] = blueprint.notes;
  seedExamQueue(id);
  state.workspace.selectedCourseId = id;
  state.workspace.dashboard = buildDashboard(id);
  return clone(state.workspace);
}

export async function deleteCourseMock(courseId: string) {
  for (const note of state.notesByCourse[courseId] ?? []) {
    delete state.noteMastery[note.id];
    delete state.noteAccuracy[note.id];
  }
  state.workspace.courses = state.workspace.courses.filter((course) => course.id !== courseId);
  delete state.notesByCourse[courseId];
  delete state.flashcardRuns[courseId];
  delete state.revisionRuns[courseId];
  delete state.examSourceQueue[courseId];
  delete state.examsByCourse[courseId];
  state.chatThreads = state.chatThreads.filter((thread) => thread.courseId !== courseId);
  state.formulaBriefs = Object.fromEntries(
    Object.entries(state.formulaBriefs).filter(([key]) => !key.startsWith(`${courseId}:`)),
  );
  state.workspace.selectedCourseId = state.workspace.courses[0]?.id ?? null;
  state.workspace.dashboard = state.workspace.selectedCourseId
    ? buildDashboard(state.workspace.selectedCourseId)
    : null;
  return clone(state.workspace);
}

export async function runScanMock() {
  ensureDemoWorkspace();
  const courseId = state.workspace.selectedCourseId ?? state.workspace.courses[0]?.id;
  if (!courseId) {
    throw new Error("Add a course before running a scan.");
  }

  const course = state.workspace.courses.find((entry) => entry.id === courseId);
  if (!course) {
    throw new Error("Selected course not found.");
  }

  const seededNotes = buildBlueprint(course).notes;
  state.notesByCourse[courseId] = seededNotes;
  seedExamQueue(courseId);
  const dashboard = buildDashboard(courseId);
  state.workspace.dashboard = dashboard;
  state.workspace.scanStatus = {
    lastScanAt: now(),
    noteCount: dashboard.notes.length,
    changedCount: dashboard.notes.length,
    removedCount: 0,
  };
  state.workspace.courses = state.workspace.courses.map((entry) =>
    entry.id === courseId
      ? {
          ...entry,
          noteCount: dashboard.graph.noteCount,
          conceptCount: dashboard.coverage.totalConcepts,
          formulaCount: dashboard.formulas.length,
          coverage: dashboard.coverage.percentage,
          weakNoteCount: dashboard.weakNotes.length,
        }
      : entry,
  );

  const report: ScanReport = {
    scannedNotes: dashboard.notes.length,
    changedNotes: dashboard.notes.length,
    unchangedNotes: 0,
    removedNotes: 0,
    generatedEdges: dashboard.graph.edgeCount,
    generatedWeakLinks: dashboard.weakNotes.length,
    scannedAt: now(),
  };

  return { workspace: clone(state.workspace), report };
}

export async function getDashboardMock(courseId: string | null) {
  ensureDemoWorkspace();
  const selectedCourseId = courseId ?? state.workspace.selectedCourseId ?? state.workspace.courses[0]?.id ?? null;
  if (!selectedCourseId) return null;
  state.workspace.selectedCourseId = selectedCourseId;
  state.workspace.dashboard = buildDashboard(selectedCourseId);
  return clone(state.workspace.dashboard);
}

export async function getStatisticsMock(scope: StatisticsScope, courseId?: string | null) {
  ensureDemoWorkspace();

  if (scope === "course") {
    const selectedCourseId = courseId ?? state.workspace.selectedCourseId ?? state.workspace.courses[0]?.id ?? null;
    if (!selectedCourseId) return null;
    return clone(buildCourseStatisticsResponse(selectedCourseId));
  }

  return clone(buildVaultStatisticsResponse());
}

export async function getNoteDetailsMock(noteId: string) {
  ensureDemoWorkspace();
  for (const notes of Object.values(state.notesByCourse)) {
    const note = notes.find((entry) => entry.id === noteId);
    if (note) return clone(note);
  }
  throw new Error("Note not found.");
}

export async function generateNoteAiInsightMock(noteId: string) {
  ensureDemoWorkspace();
  for (const [courseId, notes] of Object.entries(state.notesByCourse)) {
    const note = notes.find((entry) => entry.id === noteId);
    if (!note) {
      continue;
    }

    const insight: AiNoteInsight = {
      noteId,
      summary: `${note.title} is a core study note. Focus on the definition, why it matters, and how it connects to the surrounding theory.`,
      takeaways: note.concepts.slice(0, 3).map((concept) => `Be able to explain ${concept} without looking at the note.`),
      examQuestions: [
        `What is the main idea behind ${note.title}?`,
        `Which typical exam exercise relies on ${note.title}?`,
        `How does ${note.title} connect to ${note.links[0] ?? "the next related topic"}?`,
      ],
      connectionOpportunities: note.suggestions.slice(0, 3),
      generatedAt: now(),
      model: "preview-demo",
    };

    note.aiStatus = "complete";
    note.aiError = null;
    note.aiInsight = insight;
    state.workspace.dashboard = buildDashboard(courseId);
    return clone(insight);
  }

  throw new Error("Note not found.");
}

export async function startAiEnrichmentMock(courseId: string, force = false) {
  ensureDemoWorkspace();
  const notes = state.notesByCourse[courseId];
  if (!notes) {
    throw new Error("Course not found.");
  }

  state.notesByCourse[courseId] = notes.map((note) => {
    if (!force && note.aiInsight) {
      return { ...note, aiStatus: "complete", aiError: null };
    }

    return {
      ...note,
      aiStatus: "complete",
      aiError: null,
      aiInsight: {
        noteId: note.id,
        summary: `${note.title} is ready for revision. Focus on the definition, the standard exercise pattern, and the link to the next chapter.`,
        takeaways: note.concepts
          .slice(0, 3)
          .map((concept) => `Own ${concept} well enough to explain it from memory.`),
        examQuestions: [
          `State the core result in ${note.title}.`,
          `Show a standard application of ${note.title}.`,
          `Explain how ${note.title} connects to ${note.links[0] ?? "the next related note"}.`,
        ],
        connectionOpportunities: note.suggestions.slice(0, 3),
        generatedAt: now(),
        model: "preview-demo",
      },
    };
  });

  state.workspace.dashboard = buildDashboard(courseId);
  return clone(state.workspace.dashboard!.ai);
}

export async function saveAiSettingsMock(input: AiSettingsInput) {
  ensureDemoWorkspace();
  state.workspace.aiSettings = {
    baseUrl: input.baseUrl,
    model: input.model,
    enabled: input.enabled,
    timeoutMs: input.timeoutMs,
    hasApiKey: Boolean(input.apiKey),
  };
  return clone(state.workspace);
}

export async function validateAiSettingsMock(input: AiSettingsInput): Promise<ValidationResult> {
  return {
    ok: Boolean(input.baseUrl && input.model),
    message:
      input.baseUrl && input.model
        ? "Browser preview: settings shape looks valid."
        : "Base URL and model are required.",
  };
}

export async function generateFlashcardsMock(request: FlashcardGenerationRequest) {
  ensureDemoWorkspace();
  const stamp = new Date().toISOString().slice(0, 19).replace(/[T:]/g, "-");
  const basePath = state.workspace.vault?.vaultPath ?? PREVIEW_VAULT_PATH;
  const result: FlashcardGenerationResult = {
    markdownPath: `${basePath}\\${request.flashcardsFolder ?? "Flashcards"}\\flashcards-${stamp}.md`,
    csvPath: `${basePath}\\exports\\flashcards-${stamp}.csv`,
    cardCount: Math.max(request.noteIds.length * 4, 4),
    generatedAt: now(),
  };

  state.flashcardRuns[request.courseId] = [result, ...(state.flashcardRuns[request.courseId] ?? [])];
  state.workspace.dashboard = buildDashboard(request.courseId);
  return result;
}

export async function generateRevisionNoteMock(request: RevisionNoteRequest) {
  ensureDemoWorkspace();
  const stamp = new Date().toISOString().slice(0, 19).replace(/[T:]/g, "-");
  const basePath = state.workspace.vault?.vaultPath ?? PREVIEW_VAULT_PATH;
  const result: RevisionNoteResult = {
    notePath: `${basePath}\\${request.revisionFolder ?? "Revision"}\\revision-${stamp}.md`,
    generatedAt: now(),
    itemCount: 8,
  };

  state.revisionRuns[request.courseId] = [result, ...(state.revisionRuns[request.courseId] ?? [])];
  state.workspace.dashboard = buildDashboard(request.courseId);
  return result;
}

export async function getExamWorkspaceMock(courseId: string | null) {
  ensureDemoWorkspace();
  const selectedCourseId = courseId ?? state.workspace.selectedCourseId ?? state.workspace.courses[0]?.id ?? null;
  if (!selectedCourseId) {
    return null;
  }

  ensureExamCourseState(selectedCourseId);
  return clone(buildExamWorkspace(selectedCourseId));
}

export async function addExamSourceNotesMock(courseId: string, noteIds: string[]) {
  ensureDemoWorkspace();
  ensureExamCourseState(courseId);
  const availableNoteIds = new Set((state.notesByCourse[courseId] ?? []).map((note) => note.id));
  const nextQueue = new Set(state.examSourceQueue[courseId] ?? []);
  for (const noteId of noteIds) {
    if (availableNoteIds.has(noteId)) {
      nextQueue.add(noteId);
    }
  }
  state.examSourceQueue[courseId] = Array.from(nextQueue);
  return clone(buildExamWorkspace(courseId));
}

export async function removeExamSourceNotesMock(courseId: string, noteIds: string[]) {
  ensureDemoWorkspace();
  ensureExamCourseState(courseId);
  const removal = new Set(noteIds);
  state.examSourceQueue[courseId] = (state.examSourceQueue[courseId] ?? []).filter((noteId) => !removal.has(noteId));
  return clone(buildExamWorkspace(courseId));
}

export async function clearExamSourceQueueMock(courseId: string) {
  ensureDemoWorkspace();
  ensureExamCourseState(courseId);
  state.examSourceQueue[courseId] = [];
  return clone(buildExamWorkspace(courseId));
}

export async function queueExamsMock(request: ExamBuilderInput) {
  ensureDemoWorkspace();
  ensureExamCourseState(request.courseId);

  if (!state.workspace.aiSettings?.enabled) {
    throw new Error("Enable AI in Setup before generating exams.");
  }

  const sourceQueue = state.examSourceQueue[request.courseId] ?? [];
  if (!sourceQueue.length) {
    throw new Error("Add one or more notes to the exam source queue first.");
  }

  const presetDefaults = EXAM_PRESET_DEFAULTS[request.preset];
  const normalizedRequest: ExamBuilderInput = {
    ...request,
    multipleChoiceCount: Math.max(1, request.multipleChoiceCount || presetDefaults.multipleChoiceCount),
    shortAnswerCount: Math.max(1, request.shortAnswerCount || presetDefaults.shortAnswerCount),
    difficulty: request.difficulty || presetDefaults.difficulty,
    timeLimitMinutes: Math.max(5, request.timeLimitMinutes || presetDefaults.timeLimitMinutes),
    generateCount: Math.max(1, Math.min(5, request.generateCount || presetDefaults.generateCount)),
  };
  const generateCount = normalizedRequest.generateCount;
  const exams = state.examsByCourse[request.courseId] ?? [];

  for (let index = 0; index < generateCount; index += 1) {
    exams.unshift(createQueuedExamRecord(request.courseId, normalizedRequest, exams.length + index));
  }

  state.examsByCourse[request.courseId] = exams;
  startExamGenerationLoop(request.courseId);
  return clone(buildExamWorkspace(request.courseId));
}

export async function getExamDetailsMock(examId: string) {
  ensureDemoWorkspace();
  const record = findExamRecord(examId);
  if (!record) {
    throw new Error("Exam not found.");
  }
  return clone(stripExamRecord(record));
}

export async function submitExamAttemptMock(request: ExamSubmissionRequest) {
  ensureDemoWorkspace();
  const record = findExamRecord(request.examId);
  if (!record || record.status !== "ready") {
    throw new Error("Exam is not ready yet.");
  }

  const answers = new Map(request.answers.map((entry) => [entry.questionId, entry.answer]));
  const noteScores = new Map<string, { correct: number; total: number }>();
  const questionResults: ExamQuestionResult[] = record.questions.map((question) => {
    const key = record.gradingKeys[question.id];
    const userAnswer = answers.get(question.id) ?? (question.type === "multiple-choice" ? "" : "");
    const verdict = scoreExamAnswer(question, userAnswer, key.keyword, key.expectedAnswer);
    const result: ExamQuestionResult = {
      questionId: question.id,
      index: question.index,
      type: question.type,
      prompt: question.prompt,
      options: question.options,
      sourceNoteId: question.sourceNoteId,
      sourceNoteTitle: question.sourceNoteTitle,
      userAnswer,
      verdict,
      isCorrect: verdict === "correct",
      expectedAnswer: key.expectedAnswer,
      explanation: key.explanation,
      feedback:
        verdict === "correct"
          ? "Correct. Keep this idea in your active recall rotation."
          : verdict === "partial"
            ? "Partly right. Tighten the wording and the exact definition."
            : "Incorrect. Bring this note back into focused review.",
    };

    const noteScore = noteScores.get(question.sourceNoteId) ?? { correct: 0, total: 0 };
    noteScore.total += 1;
    if (verdict === "correct") {
      noteScore.correct += 1;
    } else if (verdict === "partial") {
      noteScore.correct += 0.5;
    }
    noteScores.set(question.sourceNoteId, noteScore);
    return result;
  });

  const correctCount = questionResults.filter((result) => result.verdict === "correct").length;
  const partialCount = questionResults.filter((result) => result.verdict === "partial").length;
  const incorrectCount = questionResults.length - correctCount - partialCount;
  const scorePercent = Math.round(((correctCount + partialCount * 0.5) / Math.max(questionResults.length, 1)) * 100);

  const noteSuggestions = Array.from(noteScores.entries()).map(([noteId, score]) => {
    const accuracy = Math.round((score.correct / Math.max(score.total, 1)) * 100);
    state.noteAccuracy[noteId] = accuracy;
    const recommendedState: NoteMasteryState =
      accuracy >= 80 ? "mastered" : accuracy < 60 ? "review" : "active";
    const note = lookupNote(noteId);

    return {
      noteId,
      title: note?.title ?? "Unknown note",
      relativePath: note?.relativePath ?? "",
      currentState: state.noteMastery[noteId] ?? "active",
      recommendedState,
      accuracy,
      reason:
        recommendedState === "mastered"
          ? "High accuracy in this attempt. You can safely put this note away for now."
          : recommendedState === "review"
            ? "Low accuracy in this attempt. Bring this note back into the learning queue."
            : "Mixed performance. Keep this note active until recall stabilizes.",
      currentlyInSourceQueue: (state.examSourceQueue[record.courseId] ?? []).includes(noteId),
    } satisfies ExamReviewSuggestion;
  });

  const attempt: ExamAttemptResult = {
    examId: record.id,
    attemptId: `attempt-${crypto.randomUUID()}`,
    submittedAt: now(),
    scorePercent,
    correctCount,
    partialCount,
    incorrectCount,
    overallFeedback:
      scorePercent >= 80
        ? "Strong result. You can retire the clean notes and queue one harder mixed exam next."
        : scorePercent >= 60
          ? "Decent base, but the weaker notes still need another pass before you compress the material."
          : "This exam exposed real gaps. Move the missed notes into review and retry a shorter exam.",
    questionResults,
    noteSuggestions,
  };

  record.attempts.unshift(attempt);
  record.latestScorePercent = attempt.scorePercent;
  record.updatedAt = attempt.submittedAt;
  return clone(attempt);
}

export async function applyExamReviewActionsMock(request: ApplyExamReviewActionsRequest) {
  ensureDemoWorkspace();
  const record = findExamRecordByAttempt(request.attemptId);
  if (!record) {
    throw new Error("Exam attempt not found.");
  }

  ensureExamCourseState(record.courseId);
  const sourceQueue = new Set(state.examSourceQueue[record.courseId] ?? []);
  for (const action of request.actions) {
    state.noteMastery[action.noteId] = action.nextState;
    if (action.addToExamQueue) {
      sourceQueue.add(action.noteId);
    } else {
      sourceQueue.delete(action.noteId);
    }
  }
  state.examSourceQueue[record.courseId] = Array.from(sourceQueue);
  return clone(buildExamWorkspace(record.courseId));
}

export async function getFormulaWorkspaceMock(courseId: string | null) {
  ensureDemoWorkspace();
  const selectedCourseId = courseId ?? state.workspace.selectedCourseId ?? state.workspace.courses[0]?.id ?? null;
  if (!selectedCourseId) {
    return null;
  }

  return clone(buildFormulaWorkspace(selectedCourseId));
}

export async function getFormulaDetailsMock(formulaId: string, courseId: string) {
  ensureDemoWorkspace();
  const details = buildFormulaDetails(courseId, formulaId);
  if (!details) {
    throw new Error("Formula not found.");
  }

  return clone(details);
}

export async function generateFormulaBriefMock(request: GenerateFormulaBriefRequest) {
  ensureDemoWorkspace();
  const details = buildFormulaDetails(request.courseId, request.formulaId);
  if (!details) {
    throw new Error("Formula not found.");
  }

  const brief = buildFormulaBrief(details);
  state.formulaBriefs[formulaCacheKey(request.courseId, request.formulaId)] = brief;
  return clone(brief);
}

export async function listChatThreadsMock(scope: ChatScope, courseId?: string | null) {
  ensureDemoWorkspace();
  return clone(
    state.chatThreads
      .filter((thread) => thread.scope === scope)
      .filter((thread) => (scope === "course" ? thread.courseId === (courseId ?? null) : true))
      .map((thread) => summarizeChatThread(thread))
      .sort((left, right) => right.updatedAt.localeCompare(left.updatedAt)),
  );
}

export async function createChatThreadMock(request: CreateChatThreadRequest) {
  ensureDemoWorkspace();
  if (request.scope === "course" && !request.courseId) {
    throw new Error("Choose a course before starting a course chat.");
  }

  const course = request.courseId
    ? state.workspace.courses.find((entry) => entry.id === request.courseId) ?? null
    : null;
  const timestamp = now();
  const thread: MockChatThread = {
    id: `chat-${crypto.randomUUID()}`,
    title:
      request.title?.trim() ||
      (request.scope === "course" ? `${course?.name ?? "Course"} chat` : "Vault chat"),
    scope: request.scope,
    courseId: request.scope === "course" ? request.courseId ?? null : null,
    courseName: request.scope === "course" ? course?.name ?? null : null,
    createdAt: timestamp,
    updatedAt: timestamp,
    messages: [],
  };

  state.chatThreads.unshift(thread);
  return clone(thread);
}

export async function getChatThreadMock(threadId: string) {
  ensureDemoWorkspace();
  const thread = state.chatThreads.find((entry) => entry.id === threadId);
  if (!thread) {
    throw new Error("Chat thread not found.");
  }

  return clone(thread);
}

export async function sendChatMessageMock(request: SendChatMessageRequest) {
  ensureDemoWorkspace();
  const thread = state.chatThreads.find((entry) => entry.id === request.threadId);
  if (!thread) {
    throw new Error("Chat thread not found.");
  }
  if (!state.workspace.aiSettings?.enabled) {
    throw new Error("Enable AI in Setup before using grounded chat.");
  }

  const prompt = request.content.trim();
  if (!prompt) {
    throw new Error("Enter a question before sending.");
  }

  const userMessage: ChatMessage = {
    id: `message-${crypto.randomUUID()}`,
    threadId: thread.id,
    role: "user",
    content: prompt,
    createdAt: now(),
    citations: [],
    usedFallback: false,
    fallbackReason: null,
  };

  thread.messages.push(userMessage);
  if (thread.messages.filter((message) => message.role === "user").length === 1) {
    thread.title = buildChatThreadTitle(prompt);
  }

  const assistantMessage = buildChatAssistantMessage(thread, prompt);
  thread.messages.push(assistantMessage);
  thread.updatedAt = assistantMessage.createdAt;

  return clone(thread);
}

export async function deleteChatThreadMock(threadId: string) {
  ensureDemoWorkspace();
  state.chatThreads = state.chatThreads.filter((thread) => thread.id !== threadId);
}

function buildDashboard(courseId: string): DashboardData {
  const course = state.workspace.courses.find((entry) => entry.id === courseId);
  if (!course) {
    throw new Error("Course not found.");
  }

  const blueprint = buildBlueprint(course);
  const notes = state.notesByCourse[courseId] ?? blueprint.notes;
  const flashcards = state.flashcardRuns[courseId]?.[0] ?? null;
  const revision = state.revisionRuns[courseId]?.[0] ?? null;
  const noteSummaries: NoteSummary[] = notes.map((note, index) => ({
    id: note.id,
    title: note.title,
    relativePath: note.relativePath,
    linkCount: note.links.length,
    conceptCount: note.concepts.length,
    formulaCount: note.formulas.length,
    strength: Number((3.6 - index * 0.4).toFixed(1)),
    aiStatus: note.aiStatus,
  }));

  return {
    generatedAt: now(),
    vaultPath: state.workspace.vault?.vaultPath ?? PREVIEW_VAULT_PATH,
    selectedCourseId: courseId,
    countdown: {
      examDate: course.examDate,
      daysRemaining: course.examDate
        ? Math.max(2, Math.round((new Date(course.examDate).getTime() - Date.now()) / 86400000))
        : null,
      label: course.examDate
        ? `${Math.max(2, Math.round((new Date(course.examDate).getTime() - Date.now()) / 86400000))} days until exam`
        : "No exam date configured",
    },
    coverage: blueprint.coverage,
    graph: {
      ...blueprint.graph,
      noteCount: notes.length,
    },
    ai: buildAiSummary(notes),
    weakNotes: notes.slice(-2).map((note, index) => ({
      noteId: note.id,
      title: note.title,
      relativePath: note.relativePath,
      score: Number((0.58 + index * 0.09).toFixed(2)),
      suggestions: note.suggestions,
    })),
    topConcepts: blueprint.topConcepts,
    formulas: blueprint.formulas,
    flashcards: {
      setCount: state.flashcardRuns[courseId]?.length ?? 0,
      totalCards: state.flashcardRuns[courseId]?.reduce((sum, run) => sum + run.cardCount, 0) ?? 0,
      lastGeneratedAt: flashcards?.generatedAt ?? null,
      exportPath: flashcards?.csvPath ?? null,
    },
    revision: {
      lastGeneratedAt: revision?.generatedAt ?? null,
      notePath: revision?.notePath ?? null,
      itemCount: revision?.itemCount ?? 0,
    },
    exams: buildExamWorkspace(courseId).summary,
    notes: noteSummaries,
  };
}

function buildCourseStatisticsResponse(courseId: string): StatisticsResponse {
  const course = state.workspace.courses.find((entry) => entry.id === courseId);
  if (!course) {
    throw new Error("Course not found.");
  }

  const dashboard = buildDashboard(courseId);
  const summary = buildStatisticsOverviewFromDashboard(dashboard);
  const history = buildStatisticsHistory(summary, course.name.length + 1);
  const examHistory = buildStatisticsExamHistory(courseId);
  const activityBuckets = buildStatisticsActivityBuckets(courseId);
  const noteRows = buildStatisticsNoteRows(courseId);
  const git = buildMockGitAnalytics("course", courseId);

  return {
    scope: "course",
    generatedAt: now(),
    courseId: course.id,
    courseName: course.name,
    gitAvailable: Boolean(git),
    gitError: null,
    overview: {
      summary,
      history,
      courseRows: [],
      highlights: buildOverviewHighlights([], git?.courseActivity ?? []),
    },
    knowledge: {
      summary: {
        totalConcepts: summary.totalConcepts,
        coveredConcepts: summary.coveredConcepts,
        coveragePercentage: summary.coveragePercentage,
        formulaCount: summary.formulaCount,
        notesWithFormulas: summary.notesWithFormulas,
      },
      history,
      topConcepts: dashboard.topConcepts,
      topFormulas: dashboard.formulas,
      formulaDensityBuckets: buildFormulaDensityBuckets(noteRows),
      courseRows: [],
    },
    notes: {
      summary: {
        noteCount: summary.noteCount,
        averageNoteStrength: summary.averageNoteStrength,
        weakNoteCount: summary.weakNoteCount,
        isolatedNotes: summary.isolatedNotes,
        staleNoteCount: activityBuckets.find((bucket) => bucket.label === "90+ days")?.noteCount ?? 0,
      },
      history,
      strengthBuckets: buildStrengthBuckets(noteRows),
      activityBuckets,
      weakestNotes: [...noteRows].sort((left, right) => left.strength - right.strength).slice(0, 8),
      mostConnectedNotes: [...noteRows].sort((left, right) => right.linkCount - left.linkCount).slice(0, 8),
      stalestNotes: [...noteRows].sort((left, right) => (left.modifiedAt ?? "").localeCompare(right.modifiedAt ?? "")).slice(0, 8),
      mostChangedNotes: git?.topNotes ?? [],
    },
    exams: {
      summary: {
        attemptCount: summary.examAttemptCount,
        latestScore: summary.latestExamScore,
        averageScore: summary.averageExamScore,
        reviewCount: buildExamWorkspace(courseId).summary.reviewCount,
        masteredCount: buildExamWorkspace(courseId).summary.masteredCount,
      },
      scoreHistory: examHistory,
      attemptHistory: buildAttemptHistory(examHistory),
      verdictMix: buildVerdictMix(courseId),
      masteryDistribution: buildMasteryDistribution(courseId),
      recentExams: [...examHistory].reverse().slice(0, 8),
      weakestAttempts: [...examHistory].sort((left, right) => left.scorePercent - right.scorePercent).slice(0, 8),
    },
    ai: {
      summary: {
        readyNotes: summary.aiReadyNotes,
        pendingNotes: summary.aiPendingNotes,
        failedNotes: summary.aiFailedNotes,
        staleNotes: summary.aiStaleNotes,
        missingNotes: summary.aiMissingNotes,
      },
      history,
      statusBreakdown: buildAiBreakdown(summary),
      failedNotes: noteRows.filter((note) => note.aiStatus === "failed").slice(0, 8),
      staleNotes: noteRows.filter((note) => note.aiStatus === "stale" || note.aiStatus === "missing").slice(0, 8),
      courseRows: [],
    },
    outputs: {
      summary: {
        flashcardSetCount: summary.flashcardSetCount,
        flashcardTotalCards: summary.flashcardTotalCards,
        revisionRunCount: summary.revisionRunCount,
        latestRevisionItemCount: summary.latestRevisionItemCount,
        latestFlashcardExport: dashboard.flashcards.exportPath,
        latestRevisionNote: dashboard.revision.notePath,
      },
      history,
      outputMix: [
        { label: "Flashcard sets", count: summary.flashcardSetCount },
        { label: "Cards", count: summary.flashcardTotalCards },
        { label: "Revision runs", count: summary.revisionRunCount },
      ],
      latestFlashcards: dashboard.flashcards,
      latestRevision: dashboard.revision,
      courseRows: [],
    },
    vaultActivity: {
      summary: {
        totalNotes: noteRows.length,
        recentNotes: activityBuckets.find((bucket) => bucket.label === "0-7 days")?.noteCount ?? 0,
        staleNotes: activityBuckets.find((bucket) => bucket.label === "90+ days")?.noteCount ?? 0,
        unknownNotes: activityBuckets.find((bucket) => bucket.label === "Unknown")?.noteCount ?? 0,
        mostRecentModifiedAt: noteRows[0]?.modifiedAt ?? null,
      },
      activityBuckets,
      recentNotes: [...noteRows].slice(0, 8),
      courseActivity: [],
      gitTimeline: git?.commitTimeline ?? [],
      gitCourseActivity: git?.courseActivity ?? [],
      gitTopNotes: git?.topNotes ?? [],
      recentCommits: git?.recentCommits ?? [],
    },
    git,
  };
}

function buildVaultStatisticsResponse(): StatisticsResponse {
  const courseStats = state.workspace.courses.map((course) => {
    const dashboard = buildDashboard(course.id);
    const summary = buildStatisticsOverviewFromDashboard(dashboard);

    return {
      course,
      dashboard,
      summary,
      notes: buildStatisticsNoteRows(course.id),
    };
  });

  const totalConcepts = courseStats.reduce((sum, entry) => sum + entry.summary.totalConcepts, 0);
  const coveredConcepts = courseStats.reduce((sum, entry) => sum + entry.summary.coveredConcepts, 0);
  const summary: StatisticsOverview = {
    noteCount: courseStats.reduce((sum, entry) => sum + entry.summary.noteCount, 0),
    totalConcepts,
    coveredConcepts,
    coveragePercentage: totalConcepts ? Number(((coveredConcepts / totalConcepts) * 100).toFixed(1)) : 0,
    edgeCount: courseStats.reduce((sum, entry) => sum + entry.summary.edgeCount, 0),
    strongLinks: courseStats.reduce((sum, entry) => sum + entry.summary.strongLinks, 0),
    inferredLinks: courseStats.reduce((sum, entry) => sum + entry.summary.inferredLinks, 0),
    isolatedNotes: courseStats.reduce((sum, entry) => sum + entry.summary.isolatedNotes, 0),
    weakNoteCount: courseStats.reduce((sum, entry) => sum + entry.summary.weakNoteCount, 0),
    formulaCount: courseStats.reduce((sum, entry) => sum + entry.summary.formulaCount, 0),
    notesWithFormulas: courseStats.reduce((sum, entry) => sum + entry.summary.notesWithFormulas, 0),
    averageNoteStrength: Number(
      (
        courseStats.reduce((sum, entry) => sum + entry.summary.averageNoteStrength, 0) /
        Math.max(courseStats.length, 1)
      ).toFixed(1),
    ),
    flashcardSetCount: courseStats.reduce((sum, entry) => sum + entry.summary.flashcardSetCount, 0),
    flashcardTotalCards: courseStats.reduce((sum, entry) => sum + entry.summary.flashcardTotalCards, 0),
    revisionRunCount: courseStats.reduce((sum, entry) => sum + entry.summary.revisionRunCount, 0),
    latestRevisionItemCount: Math.max(...courseStats.map((entry) => entry.summary.latestRevisionItemCount), 0),
    aiReadyNotes: courseStats.reduce((sum, entry) => sum + entry.summary.aiReadyNotes, 0),
    aiPendingNotes: courseStats.reduce((sum, entry) => sum + entry.summary.aiPendingNotes, 0),
    aiFailedNotes: courseStats.reduce((sum, entry) => sum + entry.summary.aiFailedNotes, 0),
    aiStaleNotes: courseStats.reduce((sum, entry) => sum + entry.summary.aiStaleNotes, 0),
    aiMissingNotes: courseStats.reduce((sum, entry) => sum + entry.summary.aiMissingNotes, 0),
    examAttemptCount: courseStats.reduce((sum, entry) => sum + entry.summary.examAttemptCount, 0),
    latestExamScore: (() => {
      const latest = courseStats
        .flatMap((entry) => buildStatisticsExamHistory(entry.course.id))
        .sort((left, right) => left.submittedAt.localeCompare(right.submittedAt))
        .pop();
      return latest?.scorePercent ?? null;
    })(),
    averageExamScore: buildAverageScore(courseStats.flatMap((entry) => buildStatisticsExamHistory(entry.course.id))),
  };
  const history = buildStatisticsHistory(summary, 9);
  const examHistory = buildVaultExamHistory();
  const activityBuckets = buildStatisticsActivityBuckets();
  const courseRows = courseStats
    .map(({ course, summary }) => ({
      courseId: course.id,
      courseName: course.name,
      noteCount: summary.noteCount,
      coveragePercentage: summary.coveragePercentage,
      edgeCount: summary.edgeCount,
      weakNoteCount: summary.weakNoteCount,
      formulaCount: summary.formulaCount,
      averageNoteStrength: summary.averageNoteStrength,
      flashcardTotalCards: summary.flashcardTotalCards,
      revisionRunCount: summary.revisionRunCount,
      aiReadyNotes: summary.aiReadyNotes,
    }))
    .sort((left, right) => left.courseName.localeCompare(right.courseName));
  const noteRows = courseStats.flatMap((entry) => entry.notes);
  const git = buildMockGitAnalytics("vault");
  const topConcepts = courseStats.flatMap((entry) => entry.dashboard.topConcepts).slice(0, 8);
  const topFormulas = courseStats.flatMap((entry) => entry.dashboard.formulas).slice(0, 6);

  return {
    scope: "vault",
    generatedAt: now(),
    courseId: null,
    courseName: null,
    gitAvailable: true,
    gitError: null,
    overview: {
      summary,
      history,
      courseRows,
      highlights: buildOverviewHighlights(courseRows, git?.courseActivity ?? []),
    },
    knowledge: {
      summary: {
        totalConcepts: summary.totalConcepts,
        coveredConcepts: summary.coveredConcepts,
        coveragePercentage: summary.coveragePercentage,
        formulaCount: summary.formulaCount,
        notesWithFormulas: summary.notesWithFormulas,
      },
      history,
      topConcepts,
      topFormulas,
      formulaDensityBuckets: buildFormulaDensityBuckets(noteRows),
      courseRows,
    },
    notes: {
      summary: {
        noteCount: summary.noteCount,
        averageNoteStrength: summary.averageNoteStrength,
        weakNoteCount: summary.weakNoteCount,
        isolatedNotes: summary.isolatedNotes,
        staleNoteCount: activityBuckets.find((bucket) => bucket.label === "90+ days")?.noteCount ?? 0,
      },
      history,
      strengthBuckets: buildStrengthBuckets(noteRows),
      activityBuckets,
      weakestNotes: [...noteRows].sort((left, right) => left.strength - right.strength).slice(0, 8),
      mostConnectedNotes: [...noteRows].sort((left, right) => right.linkCount - left.linkCount).slice(0, 8),
      stalestNotes: [...noteRows].sort((left, right) => (left.modifiedAt ?? "").localeCompare(right.modifiedAt ?? "")).slice(0, 8),
      mostChangedNotes: git?.topNotes ?? [],
    },
    exams: {
      summary: {
        attemptCount: summary.examAttemptCount,
        latestScore: summary.latestExamScore,
        averageScore: summary.averageExamScore,
        reviewCount: courseStats.reduce((sum, entry) => sum + entry.dashboard.exams.reviewCount, 0),
        masteredCount: courseStats.reduce((sum, entry) => sum + entry.dashboard.exams.masteredCount, 0),
      },
      scoreHistory: examHistory,
      attemptHistory: buildAttemptHistory(examHistory),
      verdictMix: buildVerdictMix(),
      masteryDistribution: buildMasteryDistribution(),
      recentExams: [...examHistory].reverse().slice(0, 8),
      weakestAttempts: [...examHistory].sort((left, right) => left.scorePercent - right.scorePercent).slice(0, 8),
    },
    ai: {
      summary: {
        readyNotes: summary.aiReadyNotes,
        pendingNotes: summary.aiPendingNotes,
        failedNotes: summary.aiFailedNotes,
        staleNotes: summary.aiStaleNotes,
        missingNotes: summary.aiMissingNotes,
      },
      history,
      statusBreakdown: buildAiBreakdown(summary),
      failedNotes: noteRows.filter((note) => note.aiStatus === "failed").slice(0, 8),
      staleNotes: noteRows.filter((note) => note.aiStatus === "stale" || note.aiStatus === "missing").slice(0, 8),
      courseRows,
    },
    outputs: {
      summary: {
        flashcardSetCount: summary.flashcardSetCount,
        flashcardTotalCards: summary.flashcardTotalCards,
        revisionRunCount: summary.revisionRunCount,
        latestRevisionItemCount: summary.latestRevisionItemCount,
        latestFlashcardExport:
          courseStats.find((entry) => entry.dashboard.flashcards.exportPath)?.dashboard.flashcards.exportPath ?? null,
        latestRevisionNote:
          courseStats.find((entry) => entry.dashboard.revision.notePath)?.dashboard.revision.notePath ?? null,
      },
      history,
      outputMix: [
        { label: "Flashcard sets", count: summary.flashcardSetCount },
        { label: "Cards", count: summary.flashcardTotalCards },
        { label: "Revision runs", count: summary.revisionRunCount },
      ],
      latestFlashcards: {
        setCount: summary.flashcardSetCount,
        totalCards: summary.flashcardTotalCards,
        lastGeneratedAt:
          courseStats.find((entry) => entry.dashboard.flashcards.lastGeneratedAt)?.dashboard.flashcards.lastGeneratedAt ?? null,
        exportPath:
          courseStats.find((entry) => entry.dashboard.flashcards.exportPath)?.dashboard.flashcards.exportPath ?? null,
      },
      latestRevision: {
        lastGeneratedAt:
          courseStats.find((entry) => entry.dashboard.revision.lastGeneratedAt)?.dashboard.revision.lastGeneratedAt ?? null,
        notePath:
          courseStats.find((entry) => entry.dashboard.revision.notePath)?.dashboard.revision.notePath ?? null,
        itemCount: summary.latestRevisionItemCount,
      },
      courseRows,
    },
    vaultActivity: {
      summary: {
        totalNotes: noteRows.length,
        recentNotes: activityBuckets.find((bucket) => bucket.label === "0-7 days")?.noteCount ?? 0,
        staleNotes: activityBuckets.find((bucket) => bucket.label === "90+ days")?.noteCount ?? 0,
        unknownNotes: activityBuckets.find((bucket) => bucket.label === "Unknown")?.noteCount ?? 0,
        mostRecentModifiedAt: noteRows[0]?.modifiedAt ?? null,
      },
      activityBuckets,
      recentNotes: noteRows.slice(0, 8),
      courseActivity: courseRows,
      gitTimeline: git?.commitTimeline ?? [],
      gitCourseActivity: git?.courseActivity ?? [],
      gitTopNotes: git?.topNotes ?? [],
      recentCommits: git?.recentCommits ?? [],
    },
    git,
  };
}

function buildStatisticsOverviewFromDashboard(dashboard: DashboardData): StatisticsOverview {
  const examHistory = buildStatisticsExamHistory(dashboard.selectedCourseId ?? "");
  const latestExam = examHistory.length > 0 ? examHistory[examHistory.length - 1] : null;
  const averageNoteStrength = dashboard.notes.length
    ? Number((dashboard.notes.reduce((sum, note) => sum + note.strength, 0) / dashboard.notes.length).toFixed(1))
    : 0;
  const notesWithFormulas = dashboard.notes.filter((note) => note.formulaCount > 0).length;
  return {
    noteCount: dashboard.graph.noteCount,
    totalConcepts: dashboard.coverage.totalConcepts,
    coveredConcepts: dashboard.coverage.coveredConcepts,
    coveragePercentage: dashboard.coverage.percentage,
    edgeCount: dashboard.graph.edgeCount,
    strongLinks: dashboard.graph.strongLinks,
    inferredLinks: dashboard.graph.inferredLinks,
    isolatedNotes: dashboard.graph.isolatedNotes,
    weakNoteCount: dashboard.weakNotes.length,
    formulaCount: dashboard.formulas.length,
    notesWithFormulas,
    averageNoteStrength,
    flashcardSetCount: dashboard.flashcards.setCount,
    flashcardTotalCards: dashboard.flashcards.totalCards,
    revisionRunCount: dashboard.revision.lastGeneratedAt ? 1 : 0,
    latestRevisionItemCount: dashboard.revision.itemCount,
    aiReadyNotes: dashboard.ai.readyNotes,
    aiPendingNotes: dashboard.ai.pendingNotes,
    aiFailedNotes: dashboard.ai.failedNotes,
    aiStaleNotes: dashboard.ai.staleNotes,
    aiMissingNotes: dashboard.ai.missingNotes,
    examAttemptCount: examHistory.length,
    latestExamScore: latestExam?.scorePercent ?? null,
    averageExamScore: buildAverageScore(examHistory),
  };
}

function buildStatisticsHistory(
  current: StatisticsOverview,
  seed: number,
): StatisticsSnapshotPoint[] {
  return Array.from({ length: 6 }, (_, index) => {
    const progress = (index + 1) / 6;
    const noteCount = Math.max(1, Math.round(current.noteCount * (0.55 + progress * 0.45)));
    const totalConcepts = Math.max(1, Math.round(current.totalConcepts * (0.7 + progress * 0.3)));
    const coveredConcepts = Math.min(
      totalConcepts,
      Math.max(0, Math.round(current.coveredConcepts * (0.35 + progress * 0.65))),
    );
    const edgeCount = Math.max(0, Math.round(current.edgeCount * (0.4 + progress * 0.6)));
    const strongLinks = Math.max(0, Math.round(current.strongLinks * (0.35 + progress * 0.65)));
    const inferredLinks = Math.max(0, Math.round(current.inferredLinks * (0.5 + progress * 0.5)));
    const weakNoteCount = Math.max(
      0,
      Math.round(current.weakNoteCount * (1.15 - progress * 0.35 + ((seed + index) % 2) * 0.03)),
    );
    const formulaCount = Math.max(0, Math.round(current.formulaCount * (0.5 + progress * 0.5)));
    const coveragePercentage = totalConcepts
      ? Number(((coveredConcepts / totalConcepts) * 100).toFixed(1))
      : 0;
    const examAttemptCount = Math.max(0, Math.round(current.examAttemptCount * progress));
    const averageExamScore =
      current.averageExamScore === null ? null : Number((58 + progress * 24 + (seed % 5)).toFixed(1));
    const latestExamScore =
      current.latestExamScore === null ? null : Number((60 + progress * 22 + ((seed + index) % 4)).toFixed(1));

    return {
      capturedAt: new Date(Date.now() - (5 - index) * 6 * 86400000).toISOString(),
      noteCount,
      totalConcepts,
      coveredConcepts,
      coveragePercentage,
      edgeCount,
      strongLinks,
      inferredLinks,
      isolatedNotes: Math.max(0, Math.round(current.isolatedNotes * (1.25 - progress * 0.4))),
      weakNoteCount,
      formulaCount,
      notesWithFormulas: Math.max(0, Math.round(current.notesWithFormulas * (0.6 + progress * 0.4))),
      averageNoteStrength: Number((Math.max(0.4, current.averageNoteStrength * (0.75 + progress * 0.25))).toFixed(1)),
      flashcardSetCount: Math.max(0, Math.round(current.flashcardSetCount * progress)),
      flashcardTotalCards: Math.max(0, Math.round(current.flashcardTotalCards * progress)),
      revisionRunCount: Math.max(0, Math.round(current.revisionRunCount * progress)),
      latestRevisionItemCount: Math.max(0, Math.round(current.latestRevisionItemCount * (0.7 + progress * 0.3))),
      aiReadyNotes: Math.max(0, Math.round(current.aiReadyNotes * progress)),
      aiPendingNotes: Math.max(0, Math.round(current.aiPendingNotes * (1.2 - progress * 0.5))),
      aiFailedNotes: Math.max(0, Math.round(current.aiFailedNotes * (1.15 - progress * 0.35))),
      aiStaleNotes: Math.max(0, Math.round(current.aiStaleNotes * (1.15 - progress * 0.45))),
      aiMissingNotes: Math.max(0, Math.round(current.aiMissingNotes * (1.2 - progress * 0.6))),
      examAttemptCount,
      latestExamScore,
      averageExamScore,
    };
  });
}

function buildStatisticsExamHistory(courseId: string): StatisticsExamPoint[] {
  const exams = state.examsByCourse[courseId] ?? [];
  const attempts = exams
    .flatMap((exam) =>
      exam.attempts.map((attempt) => ({
        submittedAt: attempt.submittedAt,
        examId: exam.id,
        examTitle: exam.title,
        scorePercent: attempt.scorePercent,
        courseId,
        courseName: state.workspace.courses.find((course) => course.id === courseId)?.name ?? null,
      })),
    )
    .sort((left, right) => left.submittedAt.localeCompare(right.submittedAt));

  if (attempts.length > 0) {
    return attempts;
  }

  const courseName = state.workspace.courses.find((course) => course.id === courseId)?.name ?? "Course";
  return Array.from({ length: 4 }, (_, index) => ({
    submittedAt: new Date(Date.now() - (3 - index) * 8 * 86400000).toISOString(),
    examId: `${courseId}-exam-${index + 1}`,
    examTitle: `${courseName} Check ${index + 1}`,
    scorePercent: 58 + index * 9,
    courseId,
    courseName,
  }));
}

function buildVaultExamHistory(): StatisticsExamPoint[] {
  return state.workspace.courses
    .flatMap((course) => buildStatisticsExamHistory(course.id))
    .sort((left, right) => left.submittedAt.localeCompare(right.submittedAt));
}

function buildStatisticsActivityBuckets(courseId?: string): VaultActivityBucket[] {
  const noteCount = courseId
    ? (state.notesByCourse[courseId] ?? []).length
    : Object.values(state.notesByCourse).reduce((sum, notes) => sum + notes.length, 0);
  const recent = Math.max(1, Math.round(noteCount * (courseId ? 0.28 : 0.22)));
  const medium = Math.max(0, Math.round(noteCount * (courseId ? 0.32 : 0.3)));
  const stale = Math.max(0, Math.round(noteCount * (courseId ? 0.22 : 0.25)));
  const old = Math.max(0, Math.round(noteCount * (courseId ? 0.12 : 0.17)));
  const assigned = recent + medium + stale + old;
  const unknown = Math.max(0, noteCount - assigned);

  return [
    { label: "0-7 days", noteCount: recent },
    { label: "8-30 days", noteCount: medium },
    { label: "31-90 days", noteCount: stale },
    { label: "90+ days", noteCount: old },
    { label: "Unknown", noteCount: unknown },
  ];
}

function buildAverageScore(points: StatisticsExamPoint[]) {
  if (points.length === 0) {
    return null;
  }

  return Number((points.reduce((sum, point) => sum + point.scorePercent, 0) / points.length).toFixed(1));
}

function buildStatisticsNoteRows(courseId?: string): StatisticsNoteRow[] {
  const courseIds = courseId ? [courseId] : state.workspace.courses.map((course) => course.id);
  const rows = courseIds.flatMap((id) => {
    const course = state.workspace.courses.find((entry) => entry.id === id);
    const notes = state.notesByCourse[id] ?? [];
    return notes.map((note, index) => ({
      noteId: note.id,
      title: note.title,
      relativePath: note.relativePath,
      courseId: id,
      courseName: course?.name ?? null,
      aiStatus: note.aiStatus,
      strength: Number((3.8 - index * 0.42).toFixed(1)),
      linkCount: note.links.length,
      conceptCount: note.concepts.length,
      formulaCount: note.formulas.length,
      modifiedAt: new Date(Date.now() - (index + 1) * 5 * 86400000).toISOString(),
    }));
  });
  return rows.sort((left, right) => (right.modifiedAt ?? "").localeCompare(left.modifiedAt ?? ""));
}

function buildAttemptHistory(examHistory: StatisticsExamPoint[]): StatisticsValuePoint[] {
  const grouped = new Map<string, number>();
  for (const point of examHistory) {
    const label = point.submittedAt.slice(0, 10);
    grouped.set(label, (grouped.get(label) ?? 0) + 1);
  }
  return Array.from(grouped.entries()).map(([label, count]) => ({ label, value: count }));
}

function buildVerdictMix(courseId?: string): StatisticsCountBucket[] {
  const attempts = (courseId ? state.examsByCourse[courseId] ?? [] : Object.values(state.examsByCourse).flat()).flatMap(
    (exam) => exam.attempts,
  );
  const correct = attempts.reduce((sum, attempt) => sum + attempt.correctCount, 0);
  const partial = attempts.reduce((sum, attempt) => sum + attempt.partialCount, 0);
  const incorrect = attempts.reduce((sum, attempt) => sum + attempt.incorrectCount, 0);
  return [
    { label: "Correct", count: correct },
    { label: "Partial", count: partial },
    { label: "Incorrect", count: incorrect },
  ];
}

function buildMasteryDistribution(courseId?: string): StatisticsCountBucket[] {
  const noteIds = courseId
    ? (state.notesByCourse[courseId] ?? []).map((note) => note.id)
    : Object.values(state.notesByCourse).flat().map((note) => note.id);
  const active = noteIds.filter((id) => (state.noteMastery[id] ?? "active") === "active").length;
  const review = noteIds.filter((id) => (state.noteMastery[id] ?? "active") === "review").length;
  const mastered = noteIds.filter((id) => (state.noteMastery[id] ?? "active") === "mastered").length;
  return [
    { label: "Active", count: active },
    { label: "Review", count: review },
    { label: "Mastered", count: mastered },
  ];
}

function buildAiBreakdown(summary: StatisticsOverview): StatisticsCountBucket[] {
  return [
    { label: "Ready", count: summary.aiReadyNotes },
    { label: "Queued", count: summary.aiPendingNotes },
    { label: "Failed", count: summary.aiFailedNotes },
    { label: "Stale", count: summary.aiStaleNotes },
    { label: "Missing", count: summary.aiMissingNotes },
  ];
}

function buildOverviewHighlights(courseRows: StatisticsResponse["overview"]["courseRows"], gitRows: GitCourseActivityRow[]) {
  const highlights = [];
  const strongest = [...courseRows].sort((left, right) => right.coveragePercentage - left.coveragePercentage)[0];
  if (strongest) {
    highlights.push({
      label: "Strongest coverage",
      value: `${strongest.courseName} ${strongest.coveragePercentage}%`,
      tone: "success",
    });
  }
  const fragile = [...courseRows].sort((left, right) => right.weakNoteCount - left.weakNoteCount)[0];
  if (fragile) {
    highlights.push({
      label: "Most fragile course",
      value: `${fragile.courseName} ${fragile.weakNoteCount} weak notes`,
      tone: "warning",
    });
  }
  const edited = gitRows[0];
  if (edited) {
    highlights.push({
      label: "Most edited course",
      value: `${edited.courseName} ${edited.commitCount} commits`,
      tone: "accent",
    });
  }
  return highlights;
}

function buildStrengthBuckets(noteRows: StatisticsNoteRow[]): StatisticsCountBucket[] {
  return [
    { label: "Fragile", count: noteRows.filter((note) => note.strength < 1.5).length },
    { label: "Developing", count: noteRows.filter((note) => note.strength >= 1.5 && note.strength < 3).length },
    { label: "Stable", count: noteRows.filter((note) => note.strength >= 3 && note.strength < 5).length },
    { label: "Dense", count: noteRows.filter((note) => note.strength >= 5).length },
  ];
}

function buildFormulaDensityBuckets(noteRows: StatisticsNoteRow[]): StatisticsCountBucket[] {
  return [
    { label: "0 formulas", count: noteRows.filter((note) => note.formulaCount === 0).length },
    { label: "1 formula", count: noteRows.filter((note) => note.formulaCount === 1).length },
    { label: "2 formulas", count: noteRows.filter((note) => note.formulaCount === 2).length },
    { label: "3+ formulas", count: noteRows.filter((note) => note.formulaCount >= 3).length },
  ];
}

function buildMockGitAnalytics(scope: StatisticsScope, courseId?: string): StatisticsResponse["git"] {
  const courseRows: GitCourseActivityRow[] = state.workspace.courses.map((course, index) => ({
    courseId: course.id,
    courseName: course.name,
    folder: course.folder,
    commitCount: 6 + index * 3,
    changedNotes: 12 + index * 4,
    lastCommitAt: new Date(Date.now() - index * 3 * 86400000).toISOString(),
  }));
  const filteredCourseRows =
    scope === "course" && courseId ? courseRows.filter((row) => row.courseId === courseId) : courseRows;
  const noteRows = buildStatisticsNoteRows(courseId).slice(0, 10).map((note, index) => ({
    noteId: note.noteId,
    title: note.title,
    relativePath: note.relativePath,
    courseId: note.courseId,
    courseName: note.courseName,
    changeCount: 10 - index,
    lastCommitAt: new Date(Date.now() - index * 2 * 86400000).toISOString(),
  }));
  const timeline: GitTimelinePoint[] = Array.from({ length: 8 }, (_, index) => ({
    bucket: new Date(Date.now() - (7 - index) * 30 * 86400000).toISOString().slice(0, 7),
    commitCount: 3 + index,
    changedNotes: 5 + index * 2,
  }));
  return {
    summary: {
      repoRoot: `${state.workspace.vault?.vaultPath ?? PREVIEW_VAULT_PATH}\\.git`,
      totalMarkdownCommits: timeline.reduce((sum, point) => sum + point.commitCount, 0),
      totalMarkdownFileChanges: timeline.reduce((sum, point) => sum + point.changedNotes, 0),
      lastCommitAt: new Date().toISOString(),
      recentCommitCount: 9,
      activeDays30: 12,
    },
    commitTimeline: timeline,
    churnTimeline: timeline.map((point) => ({ ...point, changedNotes: point.changedNotes + 2 })),
    courseActivity: filteredCourseRows,
    topNotes: noteRows,
    recentCommits: Array.from({ length: 6 }, (_, index) => ({
      sha: `preview-${index + 1}`,
      summary: `Refined study notes batch ${index + 1}`,
      authorName: "Preview User",
      committedAt: new Date(Date.now() - index * 3 * 86400000).toISOString(),
      changedNotes: 2 + index,
    })),
  };
}

function buildFormulaWorkspace(courseId: string): FormulaWorkspaceSnapshot {
  const formulas = listCourseFormulaSummaries(courseId);
  const notes = state.notesByCourse[courseId] ?? [];
  const notesWithFormulas = notes.filter((note) => note.formulas.length > 0).length;
  const briefedCount = formulas.filter((formula) => state.formulaBriefs[formulaCacheKey(courseId, formula.id)]).length;

  return {
    courseId,
    courseName: state.workspace.courses.find((course) => course.id === courseId)?.name ?? "Course",
    generatedAt: now(),
    formulas,
    summary: {
      formulaCount: formulas.length,
      formulaMentions: notes.reduce((sum, note) => sum + note.formulas.length, 0),
      notesWithFormulas,
      briefedCount,
    },
  };
}

function listCourseFormulaSummaries(courseId: string): FormulaSummary[] {
  const formulas = new Map<
    string,
    {
      latex: string;
      sourceNoteIds: Set<string>;
      sourceNoteTitles: Set<string>;
    }
  >();

  for (const note of state.notesByCourse[courseId] ?? []) {
    for (const formula of note.formulas) {
      const normalizedLatex = normalizeFormula(formula);
      const entry = formulas.get(normalizedLatex) ?? {
        latex: formula,
        sourceNoteIds: new Set<string>(),
        sourceNoteTitles: new Set<string>(),
      };
      entry.sourceNoteIds.add(note.id);
      entry.sourceNoteTitles.add(note.title);
      formulas.set(normalizedLatex, entry);
    }
  }

  return Array.from(formulas.entries())
    .map(([normalizedLatex, entry]) => ({
      id: buildFormulaId(courseId, normalizedLatex),
      latex: entry.latex,
      normalizedLatex,
      noteCount: entry.sourceNoteIds.size,
      sourceNoteIds: Array.from(entry.sourceNoteIds),
      sourceNoteTitles: Array.from(entry.sourceNoteTitles),
    }))
    .sort((left, right) => right.noteCount - left.noteCount || left.latex.localeCompare(right.latex));
}

function buildFormulaDetails(courseId: string, formulaId: string): FormulaDetails | null {
  const formulaSummary = listCourseFormulaSummaries(courseId).find((formula) => formula.id === formulaId);
  if (!formulaSummary) {
    return null;
  }

  const normalizedTarget = formulaSummary.normalizedLatex;
  const matchingNotes = (state.notesByCourse[courseId] ?? []).filter((note) =>
    note.formulas.some((formula) => normalizeFormula(formula) === normalizedTarget),
  );
  const linkedNotes: FormulaLinkedNote[] = matchingNotes.map((note) => ({
    noteId: note.id,
    title: note.title,
    relativePath: note.relativePath,
    excerpt: note.excerpt,
    headings: note.headings.slice(0, 3),
    relatedConcepts: note.concepts.slice(0, 4),
    formulaCount: note.formulas.length,
  }));
  const chunks: NoteChunkPreview[] = matchingNotes.flatMap((note, noteIndex) =>
    buildFormulaChunks(note, formulaSummary.latex, noteIndex),
  );
  const relatedConcepts = Array.from(
    new Set(matchingNotes.flatMap((note) => note.concepts)),
  ).slice(0, 8);
  const headings = Array.from(
    new Set(matchingNotes.flatMap((note) => note.headings)),
  ).slice(0, 8);

  return {
    id: formulaSummary.id,
    courseId,
    latex: formulaSummary.latex,
    normalizedLatex: normalizedTarget,
    noteCount: formulaSummary.noteCount,
    sourceNoteIds: formulaSummary.sourceNoteIds,
    sourceNoteTitles: formulaSummary.sourceNoteTitles,
    linkedNotes,
    chunks,
    relatedConcepts,
    headings,
    brief: state.formulaBriefs[formulaCacheKey(courseId, formulaId)] ?? null,
  };
}

function buildFormulaChunks(note: NoteDetails, latex: string, noteIndex: number): NoteChunkPreview[] {
  const fragments = [
    note.excerpt,
    ...note.headings.map((heading) => `${heading}: ${note.excerpt}`),
  ].filter(Boolean);

  return fragments.slice(0, 3).map((text, index) => ({
    chunkId: `chunk-${note.id}-${noteIndex}-${index}`,
    noteId: note.id,
    noteTitle: note.title,
    relativePath: note.relativePath,
    headingPath: note.headings[index] ?? note.headings[0] ?? "Overview",
    text: text.includes(latex) ? text : `${text} Key formula: ${latex}.`,
    ordinal: index,
  }));
}

function buildFormulaBrief(details: FormulaDetails): FormulaBrief {
  const [leftSide, rightSide] = details.latex.split("=").map((part) => part.trim());
  const sourceSignature = `${details.normalizedLatex}:${details.sourceNoteIds.join("|")}:${details.chunks.length}`;

  return {
    formulaId: details.id,
    coach: {
      meaning: `$${details.latex}$ shows up across ${details.noteCount} note${details.noteCount === 1 ? "" : "s"} in this course and anchors ${details.relatedConcepts.slice(0, 3).join(", ") || "the surrounding topic"}.`,
      symbolBreakdown: [
        leftSide ? `Left side: $${leftSide}$` : "Left side not explicitly separated in the stored formula.",
        rightSide ? `Right side: $${rightSide}$` : "Use the linked note excerpts to unpack the right-hand side.",
        `Primary note anchors: ${details.sourceNoteTitles.slice(0, 3).join(", ")}`,
      ],
      useCases: details.linkedNotes.slice(0, 3).map((note) => `Use it in ${note.title} when the task asks for the core relation, setup, or transformation.`),
      pitfalls: [
        "Do not memorize the formula without its assumptions.",
        "Check which symbols are fixed inputs versus derived outputs.",
        "Pair the formula with one concrete worked example from the linked notes.",
      ],
    },
    practice: {
      recallPrompts: [
        `State $${details.latex}$ from memory and explain each symbol.`,
        `Name the note where $${details.latex}$ first becomes important.`,
        `Say when you would apply $${details.latex}$ in an exam setting.`,
      ],
      shortAnswerDrills: details.relatedConcepts.slice(0, 3).map((concept) => `Explain how $${details.latex}$ supports ${concept}.`),
      multipleChoiceChecks: [
        `Which assumption matters most before using $${details.latex}$?`,
        `Which term in $${details.latex}$ changes when the system setup changes?`,
      ],
    },
    derivation: {
      intuition: `Treat $${details.latex}$ as a compressed summary of the logic spread across ${details.sourceNoteTitles.slice(0, 2).join(" and ") || "the linked notes"}.`,
      assumptions: details.headings.slice(0, 4).map((heading) => `Recheck the ${heading} section before deriving or using the formula.`),
      outline: [
        "Start from the definition used in the linked note.",
        "Rewrite the relationship step by step until the target expression appears.",
        "Validate the result against one of the stored examples or excerpts.",
      ],
    },
    generatedAt: now(),
    model: "preview-demo",
    sourceSignature,
  };
}

function summarizeChatThread(thread: MockChatThread): ChatThreadSummary {
  const lastMessage = thread.messages[thread.messages.length - 1];
  return {
    id: thread.id,
    title: thread.title,
    scope: thread.scope,
    courseId: thread.courseId,
    courseName: thread.courseName,
    createdAt: thread.createdAt,
    updatedAt: thread.updatedAt,
    messageCount: thread.messages.length,
    lastMessagePreview: lastMessage?.content.slice(0, 120) ?? "No messages yet",
  };
}

function buildChatAssistantMessage(thread: MockChatThread, prompt: string): ChatMessage {
  const retrieval = retrieveChatSupport(thread.scope, thread.courseId, prompt);
  const citations = retrieval.sources.map(
    (source, index): ChatCitation => ({
      chunkId: `chunk-${thread.id}-${index}`,
      noteId: source.note.id,
      noteTitle: source.note.title,
      relativePath: source.note.relativePath,
      headingPath: source.headingPath,
      excerpt: source.quote,
      courseId: source.courseId,
      courseName: source.courseName,
      relevance: source.score,
    }),
  );

  const groundedSection = citations.length
    ? [
        `From your ${thread.scope === "course" ? "course" : "vault"} notes, the strongest matches are ${citations
          .map((citation) => citation.noteTitle)
          .join(", ")}.`,
        ...retrieval.sources.slice(0, 3).map(
          (source) =>
            `${source.note.title}: ${source.note.excerpt || source.quote}${source.note.formulas[0] ? ` Key formula: ${source.note.formulas[0]}.` : ""}`,
        ),
      ].join("\n\n")
    : "I could not find a strong note-backed answer in the current scope.";
  const usedFallback = retrieval.sources.length === 0 || retrieval.sources[0].score < 2;
  const fallbackReason = usedFallback
    ? "The vault did not contain enough high-confidence note support for a complete answer."
    : null;
  const fallbackSection = usedFallback
    ? "\n\nFallback:\nBased on general study patterns, start from the core definition, attach one example, and then connect it back to the most relevant note."
    : "";

  return {
    id: `message-${crypto.randomUUID()}`,
    threadId: thread.id,
    role: "assistant",
    content: `${groundedSection}${fallbackSection}`,
    createdAt: now(),
    citations,
    usedFallback,
    fallbackReason,
  };
}

function retrieveChatSupport(scope: ChatScope, courseId: string | null, prompt: string) {
  const tokens = tokenize(prompt);
  const notes =
    scope === "course" && courseId
      ? state.notesByCourse[courseId] ?? []
      : Object.values(state.notesByCourse).flat();
  const sources = notes
    .map((note) => {
      const searchable = [
        note.title,
        note.excerpt,
        note.headings.join(" "),
        note.concepts.join(" "),
        note.formulas.join(" "),
        note.links.join(" "),
      ].join(" ");
      const normalizedSearchable = normalizeFormula(searchable);
      const score = tokens.reduce((sum, token) => sum + (normalizedSearchable.includes(token) ? 1 : 0), 0);
      return {
        note,
        courseId: Object.entries(state.notesByCourse).find(([, notes]) => notes.some((entry) => entry.id === note.id))?.[0] ?? "",
        courseName:
          state.workspace.courses.find((course) =>
            (state.notesByCourse[course.id] ?? []).some((entry) => entry.id === note.id),
          )?.name ?? "Course",
        score:
          score +
          (note.formulas.some((formula) => normalizeFormula(prompt).includes(normalizeFormula(formula))) ? 2 : 0) +
          (normalizeFormula(note.title).includes(normalizeFormula(prompt)) ? 2 : 0),
        quote: note.excerpt,
        headingPath: note.headings[0] ?? "Overview",
      };
    })
    .filter((entry) => entry.score > 0)
    .sort((left, right) => right.score - left.score || left.note.title.localeCompare(right.note.title))
    .slice(0, 3);

  return { sources };
}

function buildChatThreadTitle(prompt: string) {
  const compact = prompt.trim().replace(/\s+/g, " ");
  const words = compact.split(" ").slice(0, 6).join(" ");
  return words.length < compact.length ? `${words}...` : words || "New chat";
}

function buildFormulaId(courseId: string, normalizedLatex: string) {
  return `formula-${courseId}-${normalizedLatex}`;
}

function formulaCacheKey(courseId: string, formulaId: string) {
  return `${courseId}:${formulaId}`;
}

function normalizeFormula(value: string) {
  return value.toLowerCase().replace(/\s+/g, "").replace(/[^\w\\]/g, "");
}

function tokenize(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9\\]+/g, " ")
    .split(/\s+/)
    .filter((token) => token.length >= 3);
}

function buildAiSummary(notes: NoteDetails[]): AiCourseSummary {
  const readyNotes = notes.filter((note) => note.aiStatus === "complete").length;
  const failedNotes = notes.filter((note) => note.aiStatus === "failed").length;
  const staleNotes = notes.filter((note) => note.aiStatus === "stale").length;
  const pendingNotes = notes.filter((note) => note.aiStatus === "queued" || note.aiStatus === "running").length;
  const missingNotes = notes.filter((note) => note.aiStatus === "missing").length;
  const status =
    pendingNotes > 0
      ? "running"
      : readyNotes === notes.length && notes.length > 0
        ? "complete"
        : failedNotes > 0
          ? "failed"
          : readyNotes > 0
            ? "partial"
            : "idle";

  return {
    status,
    totalNotes: notes.length,
    readyNotes,
    pendingNotes,
    failedNotes,
    staleNotes,
    missingNotes,
    startedAt: readyNotes ? now() : null,
    finishedAt: status === "complete" ? now() : null,
    updatedAt: now(),
    model: readyNotes ? "preview-demo" : null,
    summary:
      readyNotes > 0
        ? "Preview AI grouped the course into a few clean revision tracks and highlighted where to deepen recall."
        : null,
    revisionPriorities:
      readyNotes > 0
        ? ["Rehearse the definitions first", "Work one proof-heavy note", "Turn the strongest notes into flashcards"]
        : [],
    weakSpots:
      readyNotes > 0
        ? ["Examples need backlinks to theory", "Sparse notes need one stronger anchor note"]
        : [],
    nextActions:
      readyNotes > 0
        ? ["Refresh stale notes after the next scan", "Review failed notes before generating outputs"]
        : [],
    lastError: null,
  };
}

const activeExamGenerationCourses = new Set<string>();

function ensureExamCourseState(courseId: string) {
  if (!state.examSourceQueue[courseId]) {
    seedExamQueue(courseId);
  }
  if (!state.examsByCourse[courseId]) {
    state.examsByCourse[courseId] = [];
  }
}

function seedExamQueue(courseId: string) {
  const notes = state.notesByCourse[courseId] ?? [];
  const existing = new Set(state.examSourceQueue[courseId] ?? []);
  const preserved = notes.filter((note) => existing.has(note.id)).map((note) => note.id);
  state.examSourceQueue[courseId] =
    preserved.length > 0 ? preserved : notes.slice(0, Math.min(2, notes.length)).map((note) => note.id);
  for (const note of notes) {
    state.noteMastery[note.id] ??= "active";
    state.noteAccuracy[note.id] ??= null;
  }
}

function buildExamWorkspace(courseId: string): ExamWorkspaceSnapshot {
  const notes = state.notesByCourse[courseId] ?? [];
  const noteLookup = new Map(notes.map((note) => [note.id, note]));
  const sourceQueue = (state.examSourceQueue[courseId] ?? [])
    .map((noteId) => buildExamSourceNote(noteLookup.get(noteId)))
    .filter(Boolean) as ExamSourceNote[];
  const queuedExams = (state.examsByCourse[courseId] ?? [])
    .filter((record) => record.status === "queued" || record.status === "generating")
    .map((record) => summarizeExamRecord(record));
  const readyExams = (state.examsByCourse[courseId] ?? [])
    .filter((record) => record.status === "ready")
    .map((record) => summarizeExamRecord(record));
  const failedExams = (state.examsByCourse[courseId] ?? [])
    .filter((record) => record.status === "failed")
    .map((record) => summarizeExamRecord(record));
  const history = (state.examsByCourse[courseId] ?? [])
    .flatMap((record) =>
      record.attempts.map(
        (attempt): ExamAttemptSummary => ({
          id: attempt.attemptId,
          examId: record.id,
          examTitle: record.title,
          submittedAt: attempt.submittedAt,
          scorePercent: attempt.scorePercent,
          correctCount: attempt.correctCount,
          partialCount: attempt.partialCount,
          incorrectCount: attempt.incorrectCount,
        }),
      ),
    )
    .sort((left, right) => right.submittedAt.localeCompare(left.submittedAt));
  const reviewNotes = notes
    .filter((note) => state.noteMastery[note.id] === "review")
    .map((note) => buildExamSourceNote(note))
    .filter(Boolean) as ExamSourceNote[];
  const masteredNotes = notes
    .filter((note) => state.noteMastery[note.id] === "mastered")
    .map((note) => buildExamSourceNote(note))
    .filter(Boolean) as ExamSourceNote[];

  return {
    courseId,
    defaults: DEFAULT_EXAM_DEFAULTS,
    sourceQueue,
    queuedExams,
    readyExams,
    failedExams,
    history,
    reviewNotes,
    masteredNotes,
    summary: {
      sourceQueueCount: sourceQueue.length,
      queuedCount: queuedExams.filter((exam) => exam.status === "queued").length,
      generatingCount: queuedExams.filter((exam) => exam.status === "generating").length,
      readyCount: readyExams.length,
      failedCount: failedExams.length,
      reviewCount: reviewNotes.length,
      masteredCount: masteredNotes.length,
      latestAttemptedAt: history[0]?.submittedAt ?? null,
    },
  };
}

function buildExamSourceNote(note: NoteDetails | undefined): ExamSourceNote | null {
  if (!note) {
    return null;
  }

  return {
    noteId: note.id,
    title: note.title,
    relativePath: note.relativePath,
    aiStatus: note.aiStatus,
    masteryState: state.noteMastery[note.id] ?? "active",
    lastAccuracy: state.noteAccuracy[note.id] ?? null,
    conceptCount: note.concepts.length,
    formulaCount: note.formulas.length,
  };
}

function summarizeExamRecord(record: MockExamRecord): ExamSummary {
  return {
    id: record.id,
    courseId: record.courseId,
    title: record.title,
    preset: record.preset,
    status: record.status,
    difficulty: record.difficulty,
    questionCount: record.questionCount,
    sourceNoteCount: record.sourceNoteIds.length,
    multipleChoiceCount: record.multipleChoiceCount,
    shortAnswerCount: record.shortAnswerCount,
    timeLimitMinutes: record.timeLimitMinutes,
    createdAt: record.createdAt,
    updatedAt: record.updatedAt,
    generatedAt: record.generatedAt,
    latestScorePercent: record.latestScorePercent,
    latestAttemptedAt: record.attempts[0]?.submittedAt ?? null,
    attemptCount: record.attempts.length,
    lastError: record.lastError,
  };
}

function createQueuedExamRecord(courseId: string, request: ExamBuilderInput, sequence: number): MockExamRecord {
  const questionCount = request.multipleChoiceCount + request.shortAnswerCount;
  const queuedAt = now();
  const title = request.title?.trim() ? request.title.trim() : `${capitalize(request.preset)} Exam ${sequence + 1}`;
  const sourceNoteIds = [...(state.examSourceQueue[courseId] ?? [])];
  const sourceNotes = sourceNoteIds
    .map((noteId) => buildExamSourceNote(lookupNote(noteId) ?? undefined))
    .filter(Boolean) as ExamSourceNote[];

  return {
    id: `exam-${crypto.randomUUID()}`,
    courseId,
    title,
    preset: request.preset,
    status: "queued",
    difficulty: request.difficulty,
    timeLimitMinutes: request.timeLimitMinutes,
    questionCount,
    multipleChoiceCount: request.multipleChoiceCount,
    shortAnswerCount: request.shortAnswerCount,
    createdAt: queuedAt,
    updatedAt: queuedAt,
    generatedAt: null,
    instructions:
      "Answer the questions from memory first. Use the linked note context only when reviewing after submission.",
    summary: "Exam is queued for generation.",
    questions: [],
    sourceNotes,
    lastError: null,
    latestScorePercent: null,
    attempts: [],
    sourceNoteIds,
    gradingKeys: {},
  };
}

function startExamGenerationLoop(courseId: string) {
  if (activeExamGenerationCourses.has(courseId)) {
    return;
  }

  activeExamGenerationCourses.add(courseId);
  processNextExam(courseId);
}

function processNextExam(courseId: string) {
  const exams = state.examsByCourse[courseId] ?? [];
  const generating = exams.find((exam) => exam.status === "generating");
  const queued = exams.find((exam) => exam.status === "queued");
  const current = generating ?? queued;

  if (!current) {
    activeExamGenerationCourses.delete(courseId);
    return;
  }

  if (current.status === "queued") {
    current.status = "generating";
    current.updatedAt = now();
  }

  window.setTimeout(() => {
    try {
      populateExamRecord(current);
    } catch (error) {
      current.status = "failed";
      current.lastError = error instanceof Error ? error.message : "Mock exam generation failed.";
      current.updatedAt = now();
    }
    processNextExam(courseId);
  }, 700);
}

function populateExamRecord(record: MockExamRecord) {
  const notes = record.sourceNoteIds
    .map((noteId) => lookupNote(noteId))
    .filter(Boolean) as NoteDetails[];

  if (!notes.length) {
    throw new Error("Exam source queue is empty.");
  }

  const questions: ExamQuestion[] = [];
  const gradingKeys: MockExamRecord["gradingKeys"] = {};
  const noteTitles = notes.map((note) => note.title);

  for (let index = 0; index < record.multipleChoiceCount; index += 1) {
    const note = notes[index % notes.length];
    const concept = note.concepts[index % Math.max(note.concepts.length, 1)] ?? note.title;
    const question = buildMultipleChoiceQuestion(record.id, note, concept, noteTitles, questions.length + 1);
    questions.push(question.question);
    gradingKeys[question.question.id] = question.key;
  }

  for (let index = 0; index < record.shortAnswerCount; index += 1) {
    const note = notes[(index + record.multipleChoiceCount) % notes.length];
    const concept = note.concepts[index % Math.max(note.concepts.length, 1)] ?? note.title;
    const question = buildShortAnswerQuestion(record.id, note, concept, questions.length + 1);
    questions.push(question.question);
    gradingKeys[question.question.id] = question.key;
  }

  record.questions = questions;
  record.gradingKeys = gradingKeys;
  record.sourceNotes = notes.map((note) => buildExamSourceNote(note)).filter(Boolean) as ExamSourceNote[];
  record.summary = `Built from ${notes.length} queued notes with a ${record.difficulty} difficulty mix.`;
  record.generatedAt = now();
  record.updatedAt = record.generatedAt;
  record.status = "ready";
  record.lastError = null;
}

function buildMultipleChoiceQuestion(
  examId: string,
  note: NoteDetails,
  concept: string,
  noteTitles: string[],
  index: number,
) {
  const correctAnswer = `The note connects ${concept} to ${note.title}.`;
  const distractors = [
    `It says ${concept} is unrelated to ${note.links[0] ?? noteTitles[0] ?? "the course graph"}.`,
    `It treats ${concept} as a pure memorization term without examples.`,
    `It replaces ${concept} with ${noteTitles.find((title) => title !== note.title) ?? "an unrelated chapter"}.`,
  ];
  const options = [correctAnswer, ...distractors];

  return {
    question: {
      id: `question-${crypto.randomUUID()}`,
      examId,
      index,
      type: "multiple-choice" as const,
      prompt: `Which statement best matches the note about ${concept}?`,
      options,
      sourceNoteId: note.id,
      sourceNoteTitle: note.title,
      expectedAnswer: correctAnswer,
      explanation: note.excerpt || `Review ${concept} in ${note.title}.`,
      userAnswer: null,
      isCorrect: null,
      feedback: null,
    },
    key: {
      expectedAnswer: correctAnswer,
      keyword: concept,
      explanation: note.excerpt || `Review ${concept} in ${note.title}.`,
    },
  };
}

function buildShortAnswerQuestion(examId: string, note: NoteDetails, concept: string, index: number) {
  const formula = note.formulas[0];
  const expectedAnswer = formula
    ? `${concept} matters in ${note.title}, and the key formula is ${formula}.`
    : `${concept} is a core idea in ${note.title}. ${note.excerpt}`;

  return {
    question: {
      id: `question-${crypto.randomUUID()}`,
      examId,
      index,
      type: "short-answer" as const,
      prompt: `Explain ${concept} in the context of ${note.title}.`,
      options: [],
      sourceNoteId: note.id,
      sourceNoteTitle: note.title,
      expectedAnswer,
      explanation: note.excerpt || `Revisit ${note.title} and restate the main idea clearly.`,
      userAnswer: null,
      isCorrect: null,
      feedback: null,
    },
    key: {
      expectedAnswer,
      keyword: concept.toLowerCase(),
      explanation: note.excerpt || `Revisit ${note.title} and restate the main idea clearly.`,
    },
  };
}

function scoreExamAnswer(
  question: ExamQuestion,
  answer: string | string[],
  keyword: string,
  expectedAnswer: string,
): ExamQuestionResult["verdict"] {
  if (question.type === "multiple-choice") {
    const normalized = Array.isArray(answer) ? answer.join(" ") : answer;
    return normalized.trim() === expectedAnswer ? "correct" : "incorrect";
  }

  const normalized = (Array.isArray(answer) ? answer.join(" ") : answer).trim().toLowerCase();
  if (!normalized) {
    return "incorrect";
  }
  if (normalized.includes(keyword.toLowerCase()) && normalized.length >= Math.min(expectedAnswer.length / 2, 24)) {
    return "correct";
  }
  if (normalized.includes(keyword.toLowerCase()) || normalized.split(/\s+/).length >= 4) {
    return "partial";
  }
  return "incorrect";
}

function stripExamRecord(record: MockExamRecord): ExamDetails {
  return {
    id: record.id,
    courseId: record.courseId,
    title: record.title,
    preset: record.preset,
    status: record.status,
    difficulty: record.difficulty,
    timeLimitMinutes: record.timeLimitMinutes,
    questionCount: record.questionCount,
    multipleChoiceCount: record.multipleChoiceCount,
    shortAnswerCount: record.shortAnswerCount,
    createdAt: record.createdAt,
    updatedAt: record.updatedAt,
    generatedAt: record.generatedAt,
    instructions: record.instructions,
    summary: record.summary,
    questions: record.questions.map((question) => ({
      ...question,
      expectedAnswer: null,
      explanation: null,
      userAnswer: null,
      isCorrect: null,
      feedback: null,
    })),
    sourceNotes: record.sourceNotes,
    lastError: record.lastError,
  };
}

function findExamRecord(examId: string) {
  for (const exams of Object.values(state.examsByCourse)) {
    const record = exams.find((exam) => exam.id === examId);
    if (record) {
      return record;
    }
  }
  return null;
}

function findExamRecordByAttempt(attemptId: string) {
  for (const exams of Object.values(state.examsByCourse)) {
    const record = exams.find((exam) => exam.attempts.some((attempt) => attempt.attemptId === attemptId));
    if (record) {
      return record;
    }
  }
  return null;
}

function lookupNote(noteId: string) {
  for (const notes of Object.values(state.notesByCourse)) {
    const note = notes.find((entry) => entry.id === noteId);
    if (note) {
      return note;
    }
  }
  return null;
}

function capitalize(value: string) {
  return value.slice(0, 1).toUpperCase() + value.slice(1);
}

function ensureDemoWorkspace() {
  if (state.workspace.vault && state.workspace.courses.length > 0) {
    return;
  }

  state.workspace.vault = {
    vaultPath: PREVIEW_VAULT_PATH,
    connectedAt: now(),
  };
  state.workspace.courses = [
    {
      id: "course-demo-mathe-1",
      name: "Mathe 1",
      folder: "Mathe 1",
      examDate: "2026-07-14",
      revisionFolder: "Revision",
      flashcardsFolder: "Flashcards",
      noteCount: 3,
      conceptCount: 15,
      formulaCount: 2,
      coverage: 71,
      weakNoteCount: 2,
    },
    {
      id: "course-demo-pi1",
      name: "PI1",
      folder: "PI1",
      examDate: "2026-06-30",
      revisionFolder: "Revision",
      flashcardsFolder: "Flashcards",
      noteCount: 3,
      conceptCount: 14,
      formulaCount: 2,
      coverage: 63,
      weakNoteCount: 2,
    },
    {
      id: "course-demo-ti1",
      name: "TI1",
      folder: "TI1",
      examDate: "2026-06-24",
      revisionFolder: "Revision",
      flashcardsFolder: "Flashcards",
      noteCount: 3,
      conceptCount: 18,
      formulaCount: 2,
      coverage: 67,
      weakNoteCount: 2,
    },
  ];
  state.workspace.selectedCourseId = state.workspace.courses[0]?.id ?? null;

  for (const course of state.workspace.courses) {
    state.notesByCourse[course.id] = buildBlueprint(course).notes;
    seedExamQueue(course.id);
    state.examsByCourse[course.id] = [];
  }

  state.workspace.dashboard = state.workspace.selectedCourseId
    ? buildDashboard(state.workspace.selectedCourseId)
    : null;
  state.workspace.scanStatus = {
    lastScanAt: now(),
    noteCount: 3,
    changedCount: 3,
    removedCount: 0,
  };
}

function buildBlueprint(course: CourseConfig): DemoBlueprint {
  const normalized = course.name.toLowerCase();

  if (normalized.includes("mathe")) {
    return {
      coverage: {
        totalConcepts: 15,
        coveredConcepts: 11,
        percentage: 71,
      },
      graph: {
        noteCount: 3,
        edgeCount: 11,
        strongLinks: 7,
        inferredLinks: 4,
        isolatedNotes: 1,
      },
      topConcepts: [
        { name: "Grenzwerte", noteCount: 3, supportScore: 4.8 },
        { name: "Folgen", noteCount: 2, supportScore: 4.1 },
        { name: "Stetigkeit", noteCount: 2, supportScore: 3.7 },
      ],
      formulas: [
        { latex: "lim_{x \\to a} f(x) = L", noteCount: 2 },
        { latex: "f'(x) = lim_{h \\to 0} (f(x+h)-f(x))/h", noteCount: 1 },
      ],
      notes: [
        {
          id: `${course.id}-grenzwerte`,
          title: "Grenzwerte",
          relativePath: `${course.folder}\\Grenzwerte.md`,
          excerpt: "Definitionen, Rechenregeln, and common exam patterns around limits and continuity.",
          headings: ["Definition", "Rechenregeln", "Beispiele"],
          links: ["Folgen", "Stetigkeit"],
          tags: ["analysis", "exam"],
          concepts: ["Grenzwerte", "Stetigkeit", "Rechenregeln"],
          formulas: ["lim_{x \\to a} f(x) = L"],
          suggestions: ["Link examples back to Folgen", "Add a bridge note to Stetigkeit"],
          aiStatus: "missing",
          aiError: null,
          aiInsight: null,
        },
        {
          id: `${course.id}-folgen`,
          title: "Folgen",
          relativePath: `${course.folder}\\Folgen.md`,
          excerpt: "Covers sequence convergence tests, boundedness, and the intuition needed for fast recall.",
          headings: ["Konvergenz", "Monotonie", "Beschraenktheit"],
          links: ["Grenzwerte"],
          tags: ["analysis"],
          concepts: ["Folgen", "Konvergenz", "Monotonie"],
          formulas: ["a_n \\to a"],
          suggestions: ["Reference Grenzwerte examples", "Link monotonic sequences to continuity notes"],
          aiStatus: "missing",
          aiError: null,
          aiInsight: null,
        },
        {
          id: `${course.id}-ableitungen`,
          title: "Ableitungen",
          relativePath: `${course.folder}\\Ableitungen.md`,
          excerpt: "Derivative rules and worked examples, but still weakly connected to the foundational notes.",
          headings: ["Definition", "Ableitungsregeln", "Kurvendiskussion"],
          links: ["Grenzwerte"],
          tags: ["calculus"],
          concepts: ["Ableitung", "Differenzenquotient", "Kurvendiskussion"],
          formulas: ["f'(x) = lim_{h \\to 0} (f(x+h)-f(x))/h"],
          suggestions: ["Connect derivative rules to Grenzwerte", "Add backlinks from worked examples"],
          aiStatus: "missing",
          aiError: null,
          aiInsight: null,
        },
      ],
    };
  }

  if (normalized.includes("pi")) {
    return {
      coverage: {
        totalConcepts: 14,
        coveredConcepts: 9,
        percentage: 63,
      },
      graph: {
        noteCount: 3,
        edgeCount: 9,
        strongLinks: 5,
        inferredLinks: 4,
        isolatedNotes: 1,
      },
      topConcepts: [
        { name: "Binaere Darstellung", noteCount: 2, supportScore: 4.3 },
        { name: "Aussagenlogik", noteCount: 2, supportScore: 3.9 },
        { name: "Codierung", noteCount: 2, supportScore: 3.5 },
      ],
      formulas: [
        { latex: "x = \\sum_{i=0}^{n} b_i 2^i", noteCount: 2 },
        { latex: "p \\to q \\equiv \\neg p \\lor q", noteCount: 1 },
      ],
      notes: [
        {
          id: `${course.id}-zahlensysteme`,
          title: "Zahlensysteme",
          relativePath: `${course.folder}\\Zahlensysteme.md`,
          excerpt: "Binary, decimal, and hexadecimal conversion patterns with fast exam-ready rules.",
          headings: ["Binaer", "Hexadezimal", "Umrechnung"],
          links: ["Codierung", "Aussagenlogik"],
          tags: ["grundlagen"],
          concepts: ["Binaere Darstellung", "Hexadezimal", "Umrechnung"],
          formulas: ["x = \\sum_{i=0}^{n} b_i 2^i"],
          suggestions: ["Link conversion exercises to Codierung", "Add backlinks from chapter overview"],
          aiStatus: "missing",
          aiError: null,
          aiInsight: null,
        },
        {
          id: `${course.id}-logik`,
          title: "Aussagenlogik",
          relativePath: `${course.folder}\\Aussagenlogik.md`,
          excerpt: "Logical operators, truth tables, and equivalence laws for the core PI1 questions.",
          headings: ["Operatoren", "Aequivalenz", "Wahrheitstabellen"],
          links: ["Zahlensysteme"],
          tags: ["logik"],
          concepts: ["Aussagenlogik", "Implikation", "Aequivalenz"],
          formulas: ["p \\to q \\equiv \\neg p \\lor q"],
          suggestions: ["Connect implication rules to truth table exercises"],
          aiStatus: "missing",
          aiError: null,
          aiInsight: null,
        },
        {
          id: `${course.id}-codierung`,
          title: "Codierung",
          relativePath: `${course.folder}\\Codierung.md`,
          excerpt: "Compression and representation notes that still need stronger links back into the foundations.",
          headings: ["ASCII", "Fehlererkennung", "Kompression"],
          links: [],
          tags: ["codierung"],
          concepts: ["Codierung", "ASCII", "Fehlererkennung"],
          formulas: ["H(X) = -\\sum p(x) log_2 p(x)"],
          suggestions: ["Link codierung examples to Zahlensysteme", "Add a summary link from chapter index"],
          aiStatus: "missing",
          aiError: null,
          aiInsight: null,
        },
      ],
    };
  }

  return {
    coverage: {
      totalConcepts: 18,
      coveredConcepts: 12,
      percentage: 67,
    },
    graph: {
      noteCount: 3,
      edgeCount: 14,
      strongLinks: 8,
      inferredLinks: 6,
      isolatedNotes: 1,
    },
    topConcepts: [
      { name: "Zustandsraum", noteCount: 3, supportScore: 4.7 },
      { name: "Regelkreis", noteCount: 2, supportScore: 3.9 },
      { name: "Laplace-Transformation", noteCount: 2, supportScore: 3.5 },
    ],
    formulas: [
      { latex: "x' = Ax + Bu", noteCount: 2 },
      { latex: "G(s) = C(sI - A)^{-1}B + D", noteCount: 1 },
    ],
    notes: [
      {
        id: `${course.id}-zustandsraum`,
        title: "Zustandsraum",
        relativePath: `${course.folder}\\Zustandsraum.md`,
        excerpt: "Defines the core model, notation, and assumptions used throughout the TI1 material.",
        headings: ["Modell", "Annahmen", "Notation"],
        links: ["Stabilitaet", "Uebertragungsfunktion"],
        tags: ["core"],
        concepts: ["Zustandsraum", "Notation", "Annahmen"],
        formulas: ["x' = Ax + Bu"],
        suggestions: ["Link notation examples to Stabilitaet", "Add a backlink from summary note"],
        aiStatus: "missing",
        aiError: null,
        aiInsight: null,
      },
      {
        id: `${course.id}-stabilitaet`,
        title: "Stabilitaet",
        relativePath: `${course.folder}\\Stabilitaet.md`,
        excerpt: "Collects the intuition and criteria for reasoning about system stability under exam pressure.",
        headings: ["Lyapunov", "Eigenwerte"],
        links: ["Zustandsraum"],
        tags: ["stability"],
        concepts: ["Stabilitaet", "Eigenwerte", "Lyapunov"],
        formulas: ["\\lambda(A) < 0"],
        suggestions: ["Connect this note to worked examples"],
        aiStatus: "missing",
        aiError: null,
        aiInsight: null,
      },
      {
        id: `${course.id}-uebertragungsfunktion`,
        title: "Uebertragungsfunktion",
        relativePath: `${course.folder}\\Uebertragungsfunktion.md`,
        excerpt: "Links state models to transfer functions, but it still lacks enough backlinks into the theory notes.",
        headings: ["Herleitung", "Beispiel", "Interpretation"],
        links: [],
        tags: ["signals"],
        concepts: ["Uebertragungsfunktion", "Laplace-Transformation"],
        formulas: ["G(s) = C(sI - A)^{-1}B + D"],
        suggestions: ["Link this note to Zustandsraum", "Reference Stabilitaet from worked examples"],
        aiStatus: "missing",
        aiError: null,
        aiInsight: null,
      },
    ],
  };
}

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}
