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
