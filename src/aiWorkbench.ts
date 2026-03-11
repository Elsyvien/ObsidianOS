import type { DashboardData, NoteDetails, NoteSummary } from "./types";

export type AiWorkbenchPreset = {
  id: "sprint" | "repair" | "flashcards" | "oral";
  title: string;
  description: string;
  stat: string;
  buttonLabel: string;
  noteIds: string[];
  mode: "append" | "replace";
};

export type AiWorkbenchLane = {
  id: "stabilize" | "connect" | "drill";
  title: string;
  summary: string;
  buttonLabel: string;
  noteIds: string[];
  tone: "neutral" | "warning" | "success";
  items: Array<{
    noteId: string;
    title: string;
    detail: string;
    status: string;
  }>;
};

export type AiWorkbenchSignals = {
  readinessScore: number;
  examMode: string;
  cadence: string;
  checkpoint: string;
  flashcardPotential: string;
  insightDensity: string;
};

export type NoteCoach = {
  nextMove: string;
  recallPrompts: string[];
  flashcardSeeds: string[];
  bridgeTargets: string[];
  memoryAnchors: string[];
};

export type AiWorkbench = {
  presets: AiWorkbenchPreset[];
  lanes: AiWorkbenchLane[];
  signals: AiWorkbenchSignals;
};

export function buildAiWorkbench(dashboard: DashboardData | null): AiWorkbench | null {
  if (!dashboard) {
    return null;
  }

  const weakNoteIds = new Set(dashboard.weakNotes.map((note) => note.noteId));
  const readyNotes = dashboard.notes.filter((note) => note.aiStatus === "complete");
  const repairNotes = dashboard.notes.filter((note) =>
    ["missing", "stale", "failed"].includes(note.aiStatus),
  );
  const liveQueue = dashboard.notes.filter((note) =>
    ["queued", "running"].includes(note.aiStatus),
  );
  const conceptSorted = [...dashboard.notes].sort(byStudyWeight);
  const readySorted = [...readyNotes].sort(byStudyWeight);
  const repairSorted = [...repairNotes].sort(byRepairUrgency);

  const sprintIds = uniqueNoteIds([
    ...repairSorted.slice(0, 2).map((note) => note.id),
    ...Array.from(weakNoteIds).slice(0, 1),
    ...readySorted.slice(0, 1).map((note) => note.id),
  ]).slice(0, 4);

  const repairIds = uniqueNoteIds([
    ...repairSorted.map((note) => note.id),
    ...Array.from(weakNoteIds),
  ]).slice(0, 5);

  const flashcardIds = readySorted.slice(0, 4).map((note) => note.id);
  const oralIds = conceptSorted
    .filter((note) => note.aiStatus === "complete" || note.aiStatus === "stale")
    .slice(0, 3)
    .map((note) => note.id);

  const presets: AiWorkbenchPreset[] = [
    {
      id: "sprint",
      title: "Exam Sprint",
      description: "One short session that mixes repair work with one strong note to end on recall.",
      stat: `${sprintIds.length} notes`,
      buttonLabel: "Queue sprint",
      noteIds: sprintIds,
      mode: "replace",
    },
    {
      id: "repair",
      title: "Recovery Run",
      description: "Pull weak, stale, and missing notes into one repair stack before the next scan.",
      stat: `${repairIds.length} notes`,
      buttonLabel: "Queue repair lane",
      noteIds: repairIds,
      mode: "replace",
    },
    {
      id: "flashcards",
      title: "Flashcard Batch",
      description: "Take the strongest AI-ready notes and turn them into exportable study material.",
      stat: `${flashcardIds.length} notes`,
      buttonLabel: "Queue ready notes",
      noteIds: flashcardIds,
      mode: "replace",
    },
    {
      id: "oral",
      title: "Oral Drill",
      description: "Use concept-dense notes for answer-out-loud practice instead of passive rereading.",
      stat: `${oralIds.length} notes`,
      buttonLabel: "Queue drill pack",
      noteIds: oralIds,
      mode: "append",
    },
  ];

  const lanes: AiWorkbenchLane[] = [
    {
      id: "stabilize",
      title: "Stabilize the weak edge",
      summary: liveQueue.length
        ? `${liveQueue.length} notes are still being processed, so repair the failed or missing ones next.`
        : "Clear failed, stale, and missing notes first so the course brief stays trustworthy.",
      buttonLabel: "Queue stabilize lane",
      noteIds: repairIds,
      tone: "warning",
      items: repairSorted.slice(0, 4).map((note) => ({
        noteId: note.id,
        title: note.title,
        detail: getRepairReason(note, weakNoteIds.has(note.id)),
        status: getAiRecommendationLabel(note, weakNoteIds.has(note.id)),
      })),
    },
    {
      id: "connect",
      title: "Reconnect weak structure",
      summary: "These notes have content, but the graph still wants stronger anchors and backlinks.",
      buttonLabel: "Queue connection lane",
      noteIds: Array.from(weakNoteIds).slice(0, 4),
      tone: "neutral",
      items: dashboard.weakNotes.slice(0, 4).map((note) => ({
        noteId: note.noteId,
        title: note.title,
        detail: note.suggestions[0] ?? "Add one stronger bridge note from surrounding theory.",
        status: "Reconnect",
      })),
    },
    {
      id: "drill",
      title: "Exploit the ready notes",
      summary: "These notes are dense enough for oral drills, flashcards, and last-minute active recall.",
      buttonLabel: "Queue drill lane",
      noteIds: uniqueNoteIds([...oralIds, ...flashcardIds]).slice(0, 5),
      tone: "success",
      items: readySorted.slice(0, 4).map((note) => ({
        noteId: note.id,
        title: note.title,
        detail: `${note.conceptCount} concepts · ${note.linkCount} links · strength ${note.strength.toFixed(1)}`,
        status: note.conceptCount >= 3 ? "Drill" : "Flashcards",
      })),
    },
  ];

  return {
    presets,
    lanes,
    signals: buildSignals(dashboard, readyNotes.length),
  };
}

export function buildNoteCoach(noteDetails: NoteDetails | null): NoteCoach | null {
  if (!noteDetails) {
    return null;
  }

  const recallPrompts =
    noteDetails.aiInsight?.examQuestions.slice(0, 3) ??
    noteDetails.concepts.slice(0, 2).map((concept) => `Explain ${concept} from memory.`) ??
    [];

  const flashcardSeeds = uniqueStrings([
    ...noteDetails.concepts.slice(0, 3).map((concept) => `${concept}: definition + one example`),
    ...noteDetails.formulas.slice(0, 2).map((formula) => `${formula}: name, meaning, and use case`),
  ]).slice(0, 4);

  const bridgeTargets = uniqueStrings([
    ...(noteDetails.aiInsight?.connectionOpportunities ?? []),
    ...noteDetails.suggestions,
    ...noteDetails.links.map((link) => `Reconnect this note to ${link}`),
  ]).slice(0, 4);

  const memoryAnchors = uniqueStrings([
    noteDetails.title,
    ...noteDetails.headings.slice(0, 2),
    ...noteDetails.concepts.slice(0, 2),
  ]).slice(0, 4);

  return {
    nextMove: getNoteNextMove(noteDetails),
    recallPrompts,
    flashcardSeeds,
    bridgeTargets,
    memoryAnchors,
  };
}

export function getAiRecommendationLabel(note: NoteSummary, isWeak: boolean) {
  if (note.aiStatus === "failed") return "Repair";
  if (note.aiStatus === "stale") return "Refresh";
  if (note.aiStatus === "missing") return "Generate";
  if (isWeak) return "Reconnect";
  if (note.aiStatus === "complete" && note.conceptCount >= 3) return "Drill";
  if (note.aiStatus === "complete") return "Flashcards";
  return "Watch";
}

function buildSignals(dashboard: DashboardData, readyCount: number): AiWorkbenchSignals {
  const total = Math.max(1, dashboard.ai.totalNotes);
  const rawScore =
    readyCount / total -
    dashboard.ai.failedNotes * 0.18 / total -
    dashboard.ai.staleNotes * 0.12 / total -
    dashboard.ai.missingNotes * 0.08 / total;
  const readinessScore = Math.max(0, Math.min(100, Math.round(rawScore * 100)));
  const daysRemaining = dashboard.countdown.daysRemaining;

  let examMode = "Foundation mode";
  let cadence = "Refresh strong notes twice a week and keep repairing weak structure in the background.";
  let checkpoint = "Aim to convert the best notes into flashcards before the brief goes stale.";

  if (daysRemaining !== null && daysRemaining <= 7) {
    examMode = "Final push";
    cadence = "Run two short active-recall blocks per day and stop introducing brand-new note clusters.";
    checkpoint = "Anything still missing AI support should be repaired today, not later.";
  } else if (daysRemaining !== null && daysRemaining <= 21) {
    examMode = "Compression mode";
    cadence = "Alternate repair sessions with drill sessions so the course brief stays aligned with recall.";
    checkpoint = "Queue one recovery run and one flashcard batch every few days.";
  } else if (daysRemaining !== null && daysRemaining <= 45) {
    examMode = "Build recall";
    cadence = "Use mixed sessions: one concept-dense note, one weak note, one flashcard-ready note.";
    checkpoint = "The next milestone is reducing missing and stale notes below the repair count.";
  }

  return {
    readinessScore,
    examMode,
    cadence,
    checkpoint,
    flashcardPotential: `${Math.min(readyCount, 4)} notes are ready to become output`,
    insightDensity: `${readyCount}/${dashboard.ai.totalNotes} notes already have AI coverage`,
  };
}

function getRepairReason(note: NoteSummary, isWeak: boolean) {
  if (note.aiStatus === "failed") {
    return "The last AI pass failed, so this note should be checked before trusting the course brief.";
  }
  if (note.aiStatus === "stale") {
    return "The note changed since the last brief and needs a clean refresh.";
  }
  if (note.aiStatus === "missing") {
    return isWeak
      ? "It is both underlinked and missing an AI brief, so repair its structure first."
      : "No note brief exists yet, which leaves a hole in the course synthesis.";
  }

  return isWeak
    ? "The note still needs stronger anchors in the graph."
    : "Keep this note under review while enrichment finishes.";
}

function getNoteNextMove(noteDetails: NoteDetails) {
  switch (noteDetails.aiStatus) {
    case "complete":
      return noteDetails.aiInsight
        ? "Use the summary as a spoken explanation, then turn one takeaway into a flashcard."
        : "The note is ready, but the brief is missing from the current pane. Refresh it before drilling.";
    case "stale":
      return "Refresh the note brief after checking the changed section headings and formulas.";
    case "failed":
      return "Repair the note structure or AI settings first, then retry the brief generation.";
    case "queued":
    case "running":
      return "Let enrichment finish, then convert the generated exam questions into oral drills.";
    default:
      return "Generate the note brief once the note has enough structure to support active recall.";
  }
}

function byStudyWeight(left: NoteSummary, right: NoteSummary) {
  return scoreNote(right) - scoreNote(left);
}

function byRepairUrgency(left: NoteSummary, right: NoteSummary) {
  return scoreRepair(right) - scoreRepair(left);
}

function scoreNote(note: NoteSummary) {
  return note.conceptCount * 2 + note.linkCount + note.formulaCount * 1.5 + note.strength;
}

function scoreRepair(note: NoteSummary) {
  const statusWeight =
    note.aiStatus === "failed" ? 5 : note.aiStatus === "stale" ? 4 : note.aiStatus === "missing" ? 3 : 1;
  return statusWeight * 10 + note.conceptCount * 2 + note.linkCount;
}

function uniqueNoteIds(noteIds: string[]) {
  return Array.from(new Set(noteIds.filter(Boolean)));
}

function uniqueStrings(items: string[]) {
  return Array.from(new Set(items.filter(Boolean)));
}
