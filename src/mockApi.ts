import type {
  AiCourseSummary,
  AiNoteInsight,
  AiSettingsInput,
  CourseConfig,
  CourseConfigInput,
  DashboardData,
  FlashcardGenerationRequest,
  FlashcardGenerationResult,
  NoteDetails,
  NoteSummary,
  RevisionNoteRequest,
  RevisionNoteResult,
  ScanReport,
  ValidationResult,
  WorkspaceSnapshot,
} from "./types";

type MockState = {
  workspace: WorkspaceSnapshot;
  notesByCourse: Record<string, NoteDetails[]>;
  flashcardRuns: Record<string, FlashcardGenerationResult[]>;
  revisionRuns: Record<string, RevisionNoteResult[]>;
};

type DemoBlueprint = {
  coverage: DashboardData["coverage"];
  formulas: DashboardData["formulas"];
  graph: DashboardData["graph"];
  notes: NoteDetails[];
  topConcepts: DashboardData["topConcepts"];
};

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
  state.workspace.selectedCourseId = id;
  state.workspace.dashboard = buildDashboard(id);
  return clone(state.workspace);
}

export async function deleteCourseMock(courseId: string) {
  state.workspace.courses = state.workspace.courses.filter((course) => course.id !== courseId);
  delete state.notesByCourse[courseId];
  delete state.flashcardRuns[courseId];
  delete state.revisionRuns[courseId];
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
    notes: noteSummaries,
  };
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
