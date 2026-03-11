export type AppView =
  | "overview"
  | "statistics"
  | "notes"
  | "formulas"
  | "ai"
  | "exams"
  | "chat"
  | "logs"
  | "courses"
  | "outputs"
  | "settings";

export type NoteFilter = "all" | "weak" | "selected";

export const APP_VIEWS: Array<{
  id: AppView;
  label: string;
  description: string;
}> = [
  {
    id: "overview",
    label: "Overview",
    description: "Track the active course, countdown, graph health, and the next revision moves.",
  },
  {
    id: "statistics",
    label: "Statistics",
    description: "Review course and vault graphs for coverage, graph health, exam progress, and vault activity.",
  },
  {
    id: "notes",
    label: "Notes",
    description: "Review indexed notes, queue them for flashcards, and inspect extracted structure.",
  },
  {
    id: "formulas",
    label: "Formulas",
    description: "Browse the current course formula library, open note context, and generate an AI brief per formula.",
  },
  {
    id: "ai",
    label: "AI",
    description: "Run AI enrichment, review the course brief, and track which notes still need support.",
  },
  {
    id: "exams",
    label: "Exams",
    description: "Queue generated exams, take them in app, and turn results into a real review queue.",
  },
  {
    id: "chat",
    label: "Chat",
    description: "Ask grounded questions against the current course or the whole vault, with note citations.",
  },
  {
    id: "logs",
    label: "Logs",
    description: "Review the live activity stream for scans, AI runs, exports, and runtime actions.",
  },
  {
    id: "outputs",
    label: "Outputs",
    description: "Generate flashcards and revision notes, then keep the latest exports within reach.",
  },
  {
    id: "courses",
    label: "Courses",
    description: "Define course folders, exam dates, and output targets for the vault.",
  },
  {
    id: "settings",
    label: "Setup",
    description: "Connect the vault, control runtime behavior, and configure optional AI refinement.",
  },
];

export function getViewMeta(view: AppView) {
  return APP_VIEWS.find((entry) => entry.id === view) ?? APP_VIEWS[0];
}
