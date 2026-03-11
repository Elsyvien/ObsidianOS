export type VaultConfig = {
  vaultPath: string;
  connectedAt: string;
};

export type AiSettings = {
  baseUrl: string;
  model: string;
  enabled: boolean;
  timeoutMs: number;
  hasApiKey: boolean;
};

export type AiSettingsInput = {
  baseUrl: string;
  model: string;
  apiKey?: string;
  enabled: boolean;
  timeoutMs: number;
};

export type CourseConfig = {
  id: string;
  name: string;
  folder: string;
  examDate: string | null;
  revisionFolder: string;
  flashcardsFolder: string;
  noteCount: number;
  conceptCount: number;
  formulaCount: number;
  coverage: number;
  weakNoteCount: number;
};

export type CourseConfigInput = {
  id?: string;
  name: string;
  folder: string;
  examDate?: string | null;
  revisionFolder?: string;
  flashcardsFolder?: string;
};

export type WeakNote = {
  noteId: string;
  title: string;
  relativePath: string;
  score: number;
  suggestions: string[];
};

export type ConceptMetric = {
  name: string;
  noteCount: number;
  supportScore: number;
};

export type FormulaMetric = {
  latex: string;
  noteCount: number;
};

export type FlashcardSummary = {
  setCount: number;
  totalCards: number;
  lastGeneratedAt: string | null;
  exportPath: string | null;
};

export type RevisionSummary = {
  lastGeneratedAt: string | null;
  notePath: string | null;
  itemCount: number;
};

export type ExamPreset = "sprint" | "mock" | "final";

export type ExamDifficulty = "easy" | "mixed" | "hard";

export type ExamStatus = "queued" | "generating" | "ready" | "failed";

export type ExamQuestionType = "multiple-choice" | "short-answer";

export type NoteMasteryState = "active" | "review" | "mastered";

export type ExamDefaults = {
  preset: ExamPreset;
  multipleChoiceCount: number;
  shortAnswerCount: number;
  difficulty: ExamDifficulty;
  timeLimitMinutes: number;
  generateCount: number;
};

export type ExamBuilderInput = {
  courseId: string;
  preset: ExamPreset;
  multipleChoiceCount: number;
  shortAnswerCount: number;
  difficulty: ExamDifficulty;
  timeLimitMinutes: number;
  generateCount: number;
  title?: string | null;
};

export type ExamSourceNote = {
  noteId: string;
  title: string;
  relativePath: string;
  aiStatus: string;
  masteryState: NoteMasteryState;
  lastAccuracy: number | null;
  conceptCount: number;
  formulaCount: number;
};

export type ExamSummary = {
  id: string;
  courseId: string;
  title: string;
  preset: ExamPreset;
  status: ExamStatus;
  difficulty: ExamDifficulty;
  questionCount: number;
  sourceNoteCount: number;
  multipleChoiceCount: number;
  shortAnswerCount: number;
  timeLimitMinutes: number;
  createdAt: string;
  updatedAt: string;
  generatedAt: string | null;
  latestScorePercent: number | null;
  latestAttemptedAt: string | null;
  attemptCount: number;
  lastError: string | null;
};

export type ExamQuestion = {
  id: string;
  examId: string;
  index: number;
  type: ExamQuestionType;
  prompt: string;
  options: string[];
  sourceNoteId: string;
  sourceNoteTitle: string;
  expectedAnswer: string | null;
  explanation: string | null;
  userAnswer: string | string[] | null;
  isCorrect: boolean | null;
  feedback: string | null;
};

export type ExamDetails = {
  id: string;
  courseId: string;
  title: string;
  preset: ExamPreset;
  status: ExamStatus;
  difficulty: ExamDifficulty;
  timeLimitMinutes: number;
  questionCount: number;
  multipleChoiceCount: number;
  shortAnswerCount: number;
  createdAt: string;
  updatedAt: string;
  generatedAt: string | null;
  instructions: string;
  summary: string;
  questions: ExamQuestion[];
  sourceNotes: ExamSourceNote[];
  lastError: string | null;
};

export type ExamAnswerInput = {
  questionId: string;
  answer: string | string[];
};

export type ExamSubmissionRequest = {
  examId: string;
  answers: ExamAnswerInput[];
};

export type ExamQuestionResult = {
  questionId: string;
  index: number;
  type: ExamQuestionType;
  prompt: string;
  options: string[];
  sourceNoteId: string;
  sourceNoteTitle: string;
  userAnswer: string | string[];
  verdict: "correct" | "partial" | "incorrect";
  isCorrect: boolean;
  expectedAnswer: string;
  explanation: string;
  feedback: string;
};

export type ExamReviewSuggestion = {
  noteId: string;
  title: string;
  relativePath: string;
  currentState: NoteMasteryState;
  recommendedState: NoteMasteryState;
  accuracy: number;
  reason: string;
  currentlyInSourceQueue: boolean;
};

export type ExamAttemptResult = {
  examId: string;
  attemptId: string;
  submittedAt: string;
  scorePercent: number;
  correctCount: number;
  partialCount: number;
  incorrectCount: number;
  overallFeedback: string;
  questionResults: ExamQuestionResult[];
  noteSuggestions: ExamReviewSuggestion[];
};

export type ExamReviewAction = {
  noteId: string;
  nextState: NoteMasteryState;
  addToExamQueue: boolean;
};

export type ApplyExamReviewActionsRequest = {
  attemptId: string;
  actions: ExamReviewAction[];
};

export type ExamAttemptSummary = {
  id: string;
  examId: string;
  examTitle: string;
  submittedAt: string;
  scorePercent: number;
  correctCount: number;
  partialCount: number;
  incorrectCount: number;
};

export type ExamWorkspaceSummary = {
  sourceQueueCount: number;
  queuedCount: number;
  generatingCount: number;
  readyCount: number;
  failedCount: number;
  reviewCount: number;
  masteredCount: number;
  latestAttemptedAt: string | null;
};

export type ExamWorkspaceSnapshot = {
  courseId: string;
  defaults: ExamDefaults;
  sourceQueue: ExamSourceNote[];
  queuedExams: ExamSummary[];
  readyExams: ExamSummary[];
  failedExams: ExamSummary[];
  history: ExamAttemptSummary[];
  reviewNotes: ExamSourceNote[];
  masteredNotes: ExamSourceNote[];
  summary: ExamWorkspaceSummary;
};

export type NoteSummary = {
  id: string;
  title: string;
  relativePath: string;
  linkCount: number;
  conceptCount: number;
  formulaCount: number;
  strength: number;
  aiStatus: string;
};

export type AiNoteInsight = {
  noteId: string;
  summary: string;
  takeaways: string[];
  examQuestions: string[];
  connectionOpportunities: string[];
  generatedAt: string;
  model: string;
};

export type NoteDetails = {
  id: string;
  title: string;
  relativePath: string;
  excerpt: string;
  headings: string[];
  links: string[];
  tags: string[];
  concepts: string[];
  formulas: string[];
  suggestions: string[];
  aiStatus: string;
  aiError: string | null;
  aiInsight: AiNoteInsight | null;
};

export type AiCourseSummary = {
  status: string;
  totalNotes: number;
  readyNotes: number;
  pendingNotes: number;
  failedNotes: number;
  staleNotes: number;
  missingNotes: number;
  startedAt: string | null;
  finishedAt: string | null;
  updatedAt: string | null;
  model: string | null;
  summary: string | null;
  revisionPriorities: string[];
  weakSpots: string[];
  nextActions: string[];
  lastError: string | null;
};

export type ScanStatus = {
  lastScanAt: string | null;
  noteCount: number;
  changedCount: number;
  removedCount: number;
};

export type ScanReport = {
  scannedNotes: number;
  changedNotes: number;
  unchangedNotes: number;
  removedNotes: number;
  generatedEdges: number;
  generatedWeakLinks: number;
  scannedAt: string;
};

export type FlashcardGenerationRequest = {
  courseId: string;
  noteIds: string[];
  flashcardsFolder?: string | null;
  exportCsv?: boolean;
};

export type FlashcardGenerationResult = {
  markdownPath: string;
  csvPath: string | null;
  cardCount: number;
  generatedAt: string;
};

export type RevisionNoteRequest = {
  courseId: string;
  revisionFolder?: string | null;
};

export type RevisionNoteResult = {
  notePath: string;
  generatedAt: string;
  itemCount: number;
};

export type DashboardData = {
  generatedAt: string;
  vaultPath: string;
  selectedCourseId: string | null;
  countdown: {
    examDate: string | null;
    daysRemaining: number | null;
    label: string;
  };
  coverage: {
    totalConcepts: number;
    coveredConcepts: number;
    percentage: number;
  };
  graph: {
    noteCount: number;
    edgeCount: number;
    strongLinks: number;
    inferredLinks: number;
    isolatedNotes: number;
  };
  weakNotes: WeakNote[];
  topConcepts: ConceptMetric[];
  formulas: FormulaMetric[];
  flashcards: FlashcardSummary;
  revision: RevisionSummary;
  exams: ExamWorkspaceSummary;
  notes: NoteSummary[];
  ai: AiCourseSummary;
};

export type WorkspaceSnapshot = {
  vault: VaultConfig | null;
  aiSettings: AiSettings | null;
  courses: CourseConfig[];
  selectedCourseId: string | null;
  dashboard: DashboardData | null;
  scanStatus: ScanStatus | null;
};

export type ValidationResult = {
  ok: boolean;
  message: string;
};

export type ActivityLogEntry = {
  id: string;
  timestamp: string;
  scope: string;
  title: string;
  detail: string;
  tone: "neutral" | "success" | "error";
};
