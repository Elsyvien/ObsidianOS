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

export type FormulaSummary = {
  id: string;
  latex: string;
  normalizedLatex: string;
  noteCount: number;
  sourceNoteIds: string[];
  sourceNoteTitles: string[];
};

export type FormulaWorkspaceSummary = {
  formulaCount: number;
  notesWithFormulas: number;
  formulaMentions: number;
  briefedCount: number;
};

export type FormulaWorkspaceSnapshot = {
  courseId: string;
  courseName: string;
  generatedAt: string;
  formulas: FormulaSummary[];
  summary: FormulaWorkspaceSummary;
};

export type FormulaLinkedNote = {
  noteId: string;
  title: string;
  relativePath: string;
  excerpt: string;
  headings: string[];
  relatedConcepts: string[];
  formulaCount: number;
};

export type NoteChunkPreview = {
  chunkId: string;
  noteId: string;
  noteTitle: string;
  relativePath: string;
  headingPath: string;
  text: string;
  ordinal: number;
};

export type FormulaCoach = {
  meaning: string;
  symbolBreakdown: string[];
  useCases: string[];
  pitfalls: string[];
};

export type FormulaPractice = {
  recallPrompts: string[];
  shortAnswerDrills: string[];
  multipleChoiceChecks: string[];
};

export type FormulaDerivation = {
  assumptions: string[];
  intuition: string;
  outline: string[];
};

export type FormulaBrief = {
  formulaId: string;
  coach: FormulaCoach;
  practice: FormulaPractice;
  derivation: FormulaDerivation;
  generatedAt: string;
  model: string;
  sourceSignature: string;
};

export type FormulaDetails = {
  courseId: string;
  id: string;
  latex: string;
  normalizedLatex: string;
  noteCount: number;
  sourceNoteIds: string[];
  sourceNoteTitles: string[];
  linkedNotes: FormulaLinkedNote[];
  chunks: NoteChunkPreview[];
  relatedConcepts: string[];
  headings: string[];
  brief: FormulaBrief | null;
};

export type GenerateFormulaBriefRequest = {
  courseId: string;
  formulaId: string;
  force?: boolean | null;
};

export type ChatScope = "course" | "vault";

export type ChatMessageRole = "user" | "assistant";

export type ChatCitation = {
  chunkId: string;
  noteId: string;
  noteTitle: string;
  relativePath: string;
  headingPath: string;
  excerpt: string;
  courseId: string;
  courseName: string;
  relevance: number;
};

export type ChatMessage = {
  id: string;
  threadId: string;
  role: ChatMessageRole;
  content: string;
  createdAt: string;
  citations: ChatCitation[];
  usedFallback: boolean;
  fallbackReason: string | null;
};

export type ChatThreadSummary = {
  id: string;
  scope: ChatScope;
  courseId: string | null;
  courseName: string | null;
  title: string;
  createdAt: string;
  updatedAt: string;
  messageCount: number;
  lastMessagePreview: string | null;
};

export type ChatThreadDetails = {
  id: string;
  scope: ChatScope;
  courseId: string | null;
  courseName: string | null;
  title: string;
  createdAt: string;
  updatedAt: string;
  messages: ChatMessage[];
};

export type CreateChatThreadRequest = {
  scope: ChatScope;
  courseId?: string | null;
  title?: string | null;
};

export type SendChatMessageRequest = {
  threadId: string;
  content: string;
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

export type StatisticsScope = "course" | "vault";

export type StatisticsOverview = {
  noteCount: number;
  totalConcepts: number;
  coveredConcepts: number;
  coveragePercentage: number;
  edgeCount: number;
  strongLinks: number;
  inferredLinks: number;
  isolatedNotes: number;
  weakNoteCount: number;
  formulaCount: number;
  notesWithFormulas: number;
  averageNoteStrength: number;
  flashcardSetCount: number;
  flashcardTotalCards: number;
  revisionRunCount: number;
  latestRevisionItemCount: number;
  aiReadyNotes: number;
  aiPendingNotes: number;
  aiFailedNotes: number;
  aiStaleNotes: number;
  aiMissingNotes: number;
  examAttemptCount: number;
  latestExamScore: number | null;
  averageExamScore: number | null;
};

export type StatisticsSnapshotPoint = {
  capturedAt: string;
  noteCount: number;
  totalConcepts: number;
  coveredConcepts: number;
  coveragePercentage: number;
  edgeCount: number;
  strongLinks: number;
  inferredLinks: number;
  isolatedNotes: number;
  weakNoteCount: number;
  formulaCount: number;
  notesWithFormulas: number;
  averageNoteStrength: number;
  flashcardSetCount: number;
  flashcardTotalCards: number;
  revisionRunCount: number;
  latestRevisionItemCount: number;
  aiReadyNotes: number;
  aiPendingNotes: number;
  aiFailedNotes: number;
  aiStaleNotes: number;
  aiMissingNotes: number;
  examAttemptCount: number;
  latestExamScore: number | null;
  averageExamScore: number | null;
};

export type VaultActivityBucket = {
  label: string;
  noteCount: number;
};

export type StatisticsCountBucket = {
  label: string;
  count: number;
};

export type StatisticsValuePoint = {
  label: string;
  value: number;
};

export type StatisticsHighlight = {
  label: string;
  value: string;
  tone: string;
};

export type StatisticsExamPoint = {
  submittedAt: string;
  examId: string;
  examTitle: string;
  scorePercent: number;
  courseId: string | null;
  courseName: string | null;
};

export type CourseStatisticsRow = {
  courseId: string;
  courseName: string;
  noteCount: number;
  coveragePercentage: number;
  edgeCount: number;
  weakNoteCount: number;
  formulaCount: number;
  averageNoteStrength: number;
  flashcardTotalCards: number;
  revisionRunCount: number;
  aiReadyNotes: number;
};

export type StatisticsNoteRow = {
  noteId: string;
  title: string;
  relativePath: string;
  courseId: string | null;
  courseName: string | null;
  aiStatus: string;
  strength: number;
  linkCount: number;
  conceptCount: number;
  formulaCount: number;
  modifiedAt: string | null;
};

export type StatisticsKnowledgeSummary = {
  totalConcepts: number;
  coveredConcepts: number;
  coveragePercentage: number;
  formulaCount: number;
  notesWithFormulas: number;
};

export type StatisticsNotesSummary = {
  noteCount: number;
  averageNoteStrength: number;
  weakNoteCount: number;
  isolatedNotes: number;
  staleNoteCount: number;
};

export type StatisticsExamsSummary = {
  attemptCount: number;
  latestScore: number | null;
  averageScore: number | null;
  reviewCount: number;
  masteredCount: number;
};

export type StatisticsAiSummary = {
  readyNotes: number;
  pendingNotes: number;
  failedNotes: number;
  staleNotes: number;
  missingNotes: number;
};

export type StatisticsOutputsSummary = {
  flashcardSetCount: number;
  flashcardTotalCards: number;
  revisionRunCount: number;
  latestRevisionItemCount: number;
  latestFlashcardExport: string | null;
  latestRevisionNote: string | null;
};

export type VaultActivitySummary = {
  totalNotes: number;
  recentNotes: number;
  staleNotes: number;
  unknownNotes: number;
  mostRecentModifiedAt: string | null;
};

export type GitTimelinePoint = {
  bucket: string;
  commitCount: number;
  changedNotes: number;
};

export type GitCourseActivityRow = {
  courseId: string | null;
  courseName: string;
  folder: string;
  commitCount: number;
  changedNotes: number;
  lastCommitAt: string | null;
};

export type GitNoteActivityRow = {
  noteId: string | null;
  title: string;
  relativePath: string;
  courseId: string | null;
  courseName: string | null;
  changeCount: number;
  lastCommitAt: string | null;
};

export type GitCommitItem = {
  sha: string;
  summary: string;
  authorName: string;
  committedAt: string;
  changedNotes: number;
};

export type GitSummary = {
  repoRoot: string;
  totalMarkdownCommits: number;
  totalMarkdownFileChanges: number;
  lastCommitAt: string | null;
  recentCommitCount: number;
  activeDays30: number;
};

export type StatisticsOverviewSection = {
  summary: StatisticsOverview;
  history: StatisticsSnapshotPoint[];
  courseRows: CourseStatisticsRow[];
  highlights: StatisticsHighlight[];
};

export type StatisticsKnowledgeSection = {
  summary: StatisticsKnowledgeSummary;
  history: StatisticsSnapshotPoint[];
  topConcepts: ConceptMetric[];
  topFormulas: FormulaMetric[];
  formulaDensityBuckets: StatisticsCountBucket[];
  courseRows: CourseStatisticsRow[];
};

export type StatisticsNotesSection = {
  summary: StatisticsNotesSummary;
  history: StatisticsSnapshotPoint[];
  strengthBuckets: StatisticsCountBucket[];
  activityBuckets: VaultActivityBucket[];
  weakestNotes: StatisticsNoteRow[];
  mostConnectedNotes: StatisticsNoteRow[];
  stalestNotes: StatisticsNoteRow[];
  mostChangedNotes: GitNoteActivityRow[];
};

export type StatisticsExamsSection = {
  summary: StatisticsExamsSummary;
  scoreHistory: StatisticsExamPoint[];
  attemptHistory: StatisticsValuePoint[];
  verdictMix: StatisticsCountBucket[];
  masteryDistribution: StatisticsCountBucket[];
  recentExams: StatisticsExamPoint[];
  weakestAttempts: StatisticsExamPoint[];
};

export type StatisticsAiSection = {
  summary: StatisticsAiSummary;
  history: StatisticsSnapshotPoint[];
  statusBreakdown: StatisticsCountBucket[];
  failedNotes: StatisticsNoteRow[];
  staleNotes: StatisticsNoteRow[];
  courseRows: CourseStatisticsRow[];
};

export type StatisticsOutputsSection = {
  summary: StatisticsOutputsSummary;
  history: StatisticsSnapshotPoint[];
  outputMix: StatisticsCountBucket[];
  latestFlashcards: FlashcardSummary;
  latestRevision: RevisionSummary;
  courseRows: CourseStatisticsRow[];
};

export type StatisticsVaultActivitySection = {
  summary: VaultActivitySummary;
  activityBuckets: VaultActivityBucket[];
  recentNotes: StatisticsNoteRow[];
  courseActivity: CourseStatisticsRow[];
  gitTimeline: GitTimelinePoint[];
  gitCourseActivity: GitCourseActivityRow[];
  gitTopNotes: GitNoteActivityRow[];
  recentCommits: GitCommitItem[];
};

export type StatisticsGitSection = {
  summary: GitSummary;
  commitTimeline: GitTimelinePoint[];
  churnTimeline: GitTimelinePoint[];
  courseActivity: GitCourseActivityRow[];
  topNotes: GitNoteActivityRow[];
  recentCommits: GitCommitItem[];
};

export type StatisticsResponse = {
  scope: StatisticsScope;
  generatedAt: string;
  courseId: string | null;
  courseName: string | null;
  gitAvailable: boolean;
  gitError: string | null;
  overview: StatisticsOverviewSection;
  knowledge: StatisticsKnowledgeSection;
  notes: StatisticsNotesSection;
  exams: StatisticsExamsSection;
  ai: StatisticsAiSection;
  outputs: StatisticsOutputsSection;
  vaultActivity: StatisticsVaultActivitySection;
  git: StatisticsGitSection | null;
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
