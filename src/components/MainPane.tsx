import type { ReactNode } from "react";
import { buildAiWorkbench, getAiRecommendationLabel } from "../aiWorkbench";
import { clampPercent, formatDate, formatDateTime } from "../lib";
import type {
  ActivityLogEntry,
  CourseConfig,
  FlashcardGenerationResult,
  RevisionNoteResult,
  ScanReport,
  WorkspaceSnapshot,
} from "../types";
import type { AppView, NoteFilter } from "./appShell";
import { MathFormula } from "./MathFormula";

type AiFilter = "all" | "ready" | "needs-work" | "failed";

type MainPaneProps = {
  activeView: AppView;
  activityLog: ActivityLogEntry[];
  aiFilter: AiFilter;
  busyAction: string | null;
  flashcardResult: FlashcardGenerationResult | null;
  noteFilter: NoteFilter;
  revisionResult: RevisionNoteResult | null;
  runtimeMode: "tauri" | "browser-preview";
  scanReport: ScanReport | null;
  selectedCourse: CourseConfig | null;
  selectedNoteId: string | null;
  selectedNoteIds: string[];
  workspace: WorkspaceSnapshot;
  onChangeAiFilter: (filter: AiFilter) => void;
  onChangeNoteFilter: (filter: NoteFilter) => void;
  onCreateRevisionNote: () => void;
  onGenerateCourseAi: () => void;
  onGenerateFlashcards: () => void;
  onOpenNote: (noteId: string) => void;
  onOpenAiNote: (noteId: string) => void;
  onApplyAiSelection: (noteIds: string[], label: string, mode?: "append" | "replace") => void;
  onAddNotesToExamQueue: (noteIds: string[]) => void;
  onRefreshCourseAi: () => void;
  onSelectCourse: (courseId: string) => void;
  onStartNewCourse: () => void;
  onToggleNoteSelection: (noteId: string) => void;
};

export function MainPane({
  activeView,
  activityLog,
  aiFilter,
  busyAction,
  flashcardResult,
  noteFilter,
  revisionResult,
  runtimeMode,
  scanReport,
  selectedCourse,
  selectedNoteId,
  selectedNoteIds,
  workspace,
  onChangeAiFilter,
  onChangeNoteFilter,
  onCreateRevisionNote,
  onGenerateCourseAi,
  onGenerateFlashcards,
  onOpenNote,
  onOpenAiNote,
  onApplyAiSelection,
  onAddNotesToExamQueue,
  onRefreshCourseAi,
  onSelectCourse,
  onStartNewCourse,
  onToggleNoteSelection,
}: MainPaneProps) {
  const dashboard = workspace.dashboard;
  const isPreview = runtimeMode === "browser-preview";
  const isScanning = busyAction === "Scan failed";
  const weakNoteIds = new Set(dashboard?.weakNotes.map((note) => note.noteId) ?? []);
  const filteredNotes =
    dashboard?.notes.filter((note) => {
      if (noteFilter === "weak") return weakNoteIds.has(note.id);
      if (noteFilter === "selected") return selectedNoteIds.includes(note.id);
      return true;
    }) ?? [];
  const aiNotes =
    dashboard?.notes.filter((note) => {
      if (aiFilter === "ready") return note.aiStatus === "complete";
      if (aiFilter === "failed") return note.aiStatus === "failed";
      if (aiFilter === "needs-work") {
        return note.aiStatus === "missing" || note.aiStatus === "stale" || note.aiStatus === "queued" || note.aiStatus === "running";
      }
      return true;
    }) ?? [];
  const selectedQueue = dashboard?.notes.filter((note) => selectedNoteIds.includes(note.id)) ?? [];
  const aiWorkbench = buildAiWorkbench(dashboard);
  const scanNotice = isScanning ? (
    <section className="surface surface--status">
      <span className="surface__eyebrow">Indexing</span>
      <h3>{selectedCourse ? `Scanning ${selectedCourse.name}` : "Scanning course"}</h3>
      <p>
        Reading markdown files, updating note records, and rebuilding the graph. The view refreshes when the
        scan completes.
      </p>
    </section>
  ) : null;

  if (!workspace.vault && !isPreview) {
    return (
      <div className="page-stack">
        {scanNotice}
        <section className="surface surface--hero">
          <span className="surface__eyebrow">Setup</span>
          <h3>Connect the vault first</h3>
          <p>
            The desktop runtime is ready. Open Setup, paste the Obsidian vault path, connect it,
            then run the first scan to build the course library and note graph.
          </p>
        </section>
      </div>
    );
  }

  if (activeView === "courses") {
    return (
      <div className="page-stack">
        {scanNotice}
        <section className="surface surface--hero">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Course library</span>
              <h3>Manage the course spaces inside this vault</h3>
            </div>
            <button className="button button--subtle" onClick={onStartNewCourse} type="button">
              New course
            </button>
          </div>
          <p className="surface__summary">
            Courses are the working contexts for scans, revision notes, and flashcard exports.
          </p>
        </section>

        <section className="surface">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Library</span>
              <h3>Saved courses</h3>
            </div>
          </div>
          {workspace.courses.length === 0 ? (
            <EmptyState
              title="No courses imported"
              description="After the vault is connected, top-level folders with markdown will appear here."
            />
          ) : (
            <div className="row-list">
              {workspace.courses.map((course) => (
                <article
                  key={course.id}
                  className={`row-item ${selectedCourse?.id === course.id ? "row-item--active" : ""}`}
                >
                  <button className="row-item__main" onClick={() => onSelectCourse(course.id)} type="button">
                    <div className="row-item__title-row">
                      <span className="row-item__title">{course.name}</span>
                      <span className="soft-badge">{Math.round(course.coverage)}% coverage</span>
                    </div>
                    <span className="row-item__subtitle">{course.folder}</span>
                  </button>
                  <div className="row-item__meta">
                    <span>{course.examDate ? formatDate(course.examDate) : "No exam date"}</span>
                    <span>{course.noteCount} notes</span>
                    <span>{course.weakNoteCount} weak</span>
                  </div>
                </article>
              ))}
            </div>
          )}
        </section>

        <section className="surface">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Selection</span>
              <h3>{selectedCourse?.name ?? "Choose a course"}</h3>
            </div>
          </div>
          {selectedCourse ? (
            <dl className="definition-grid">
              <Definition label="Folder" value={selectedCourse.folder} />
              <Definition
                label="Exam date"
                value={selectedCourse.examDate ? formatDate(selectedCourse.examDate) : "Not set"}
              />
              <Definition label="Revision folder" value={selectedCourse.revisionFolder} />
              <Definition label="Flashcards folder" value={selectedCourse.flashcardsFolder} />
              <Definition label="Indexed notes" value={String(selectedCourse.noteCount)} />
              <Definition label="Weak notes" value={String(selectedCourse.weakNoteCount)} />
            </dl>
          ) : (
            <EmptyState
              title="No course selected"
              description="Select a course from the list or create a new one to edit it in the detail rail."
            />
          )}
        </section>
      </div>
    );
  }

  if (activeView === "notes") {
    return (
      <div className="page-stack">
        {scanNotice}
        <section className="surface surface--hero">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Note library</span>
              <h3>{selectedCourse ? `${selectedCourse.name} note review` : "Select a course"}</h3>
            </div>
            <div className="toolbar">
              {[
                { id: "all", label: "All notes" },
                { id: "weak", label: "Weak links" },
                { id: "selected", label: "Queued" },
              ].map((filter) => (
                <button
                  key={filter.id}
                  className={`toolbar__item ${noteFilter === filter.id ? "toolbar__item--active" : ""}`}
                  onClick={() => onChangeNoteFilter(filter.id as NoteFilter)}
                  type="button"
                >
                  {filter.label}
                </button>
              ))}
              <button
                className="toolbar__item"
                disabled={selectedNoteIds.length === 0}
                onClick={() => onAddNotesToExamQueue(selectedNoteIds)}
                type="button"
              >
                Add queued to exams
              </button>
            </div>
          </div>
          <div className="metric-strip">
            <Metric label="Visible notes" value={String(filteredNotes.length)} />
            <Metric label="Queued notes" value={String(selectedNoteIds.length)} />
            <Metric label="Weak notes" value={String(dashboard?.weakNotes.length ?? 0)} />
            <Metric label="Formulas" value={String(dashboard?.formulas.length ?? 0)} />
          </div>
        </section>

        <section className="surface">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Index</span>
              <h3>Review queue</h3>
            </div>
          </div>
          {filteredNotes.length === 0 ? (
            <EmptyState
              title="No notes in this view"
              description={
                dashboard?.notes.length
                  ? "Change the filter or queue notes from the full library."
                  : "Run a scan to build the note index for the selected course."
              }
            />
          ) : (
            <div className="row-list row-list--compact">
              {filteredNotes.map((note) => (
                <article
                  key={note.id}
                  className={`row-item ${selectedNoteId === note.id ? "row-item--active" : ""}`}
                >
                  <button className="row-item__main" onClick={() => onOpenNote(note.id)} type="button">
                    <div className="row-item__title-row">
                      <span className="row-item__title">{note.title}</span>
                      {weakNoteIds.has(note.id) ? <span className="soft-badge soft-badge--warning">Weak</span> : null}
                      {workspace.aiSettings?.enabled ? (
                        <span className={`soft-badge soft-badge--${aiStatusTone(note.aiStatus)}`}>
                          {aiStatusLabel(note.aiStatus)}
                        </span>
                      ) : null}
                    </div>
                    <span className="row-item__subtitle">{note.relativePath}</span>
                  </button>
                  <div className="row-item__meta">
                    <span>{note.conceptCount} concepts</span>
                    <span>{note.formulaCount} formulas</span>
                    <span>{note.linkCount} links</span>
                    <span>Strength {note.strength.toFixed(1)}</span>
                  </div>
                  <button
                    aria-pressed={selectedNoteIds.includes(note.id)}
                    className={`button button--ghost ${selectedNoteIds.includes(note.id) ? "button--active" : ""}`}
                    onClick={() => onToggleNoteSelection(note.id)}
                    type="button"
                  >
                    {selectedNoteIds.includes(note.id) ? "Queued" : "Queue"}
                  </button>
                </article>
              ))}
            </div>
          )}
        </section>

        <section className="surface">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Flashcard queue</span>
              <h3>Notes armed for output</h3>
            </div>
          </div>
          {selectedQueue.length ? (
            <div className="row-list row-list--compact">
              {selectedQueue.map((note) => (
                <article key={note.id} className="row-item">
                  <button className="row-item__main" onClick={() => onOpenNote(note.id)} type="button">
                    <span className="row-item__title">{note.title}</span>
                    <span className="row-item__subtitle">{note.relativePath}</span>
                  </button>
                  <div className="row-item__meta">
                    <span>{note.conceptCount} concepts</span>
                    <span>{note.formulaCount} formulas</span>
                  </div>
                  <button className="button button--ghost button--active" onClick={() => onToggleNoteSelection(note.id)} type="button">
                    Remove
                  </button>
                </article>
              ))}
            </div>
          ) : (
            <EmptyState
              title="No notes queued"
              description="Queue notes from the library above to build the flashcard set."
            />
          )}
        </section>
      </div>
    );
  }

  if (activeView === "logs") {
    const successCount = activityLog.filter((entry) => entry.tone === "success").length;
    const errorCount = activityLog.filter((entry) => entry.tone === "error").length;

    return (
      <div className="page-stack">
        {scanNotice}
        <section className="surface surface--hero">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Activity log</span>
              <h3>Session event stream</h3>
            </div>
          </div>
          <p className="surface__summary">
            Every scan, save, AI run, export, and runtime warning lands here so you can trace what the app
            actually did.
          </p>
          <div className="metric-strip">
            <Metric label="Events" value={String(activityLog.length)} />
            <Metric label="Success" value={String(successCount)} />
            <Metric label="Errors" value={String(errorCount)} />
            <Metric label="Course" value={selectedCourse?.name ?? "None"} />
          </div>
        </section>

        <section className="surface">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Timeline</span>
              <h3>Latest events</h3>
            </div>
          </div>
          {activityLog.length ? (
            <div className="log-table">
              <div className="log-table__head">
                <span>Timestamp</span>
                <span>Scope</span>
                <span>Event</span>
                <span>Detail</span>
              </div>
              <div className="log-table__body">
                {activityLog.map((entry) => (
                  <article key={entry.id} className={`log-row log-row--${entry.tone}`}>
                    <span className="log-cell log-cell--timestamp">{formatDateTime(entry.timestamp)}</span>
                    <span className="log-cell log-cell--scope">{entry.scope}</span>
                    <span className="log-cell log-cell--event">{entry.title}</span>
                    <span className="log-cell log-cell--detail">{entry.detail || "No additional detail"}</span>
                  </article>
                ))}
              </div>
            </div>
          ) : (
            <EmptyState
              title="No session events yet"
              description="Start scanning, saving settings, or running AI to build the activity stream."
            />
          )}
        </section>
      </div>
    );
  }

  if (activeView === "ai") {
    const weakNoteIds = new Set(dashboard?.weakNotes.map((note) => note.noteId) ?? []);

    return (
      <div className="page-stack">
        {scanNotice}
        <section className="surface surface--hero">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">AI workspace</span>
              <h3>{selectedCourse ? `${selectedCourse.name} study copilot` : "Choose a course"}</h3>
            </div>
            <div className="toolbar">
              {[
                { id: "all", label: "All notes" },
                { id: "needs-work", label: "Needs work" },
                { id: "ready", label: "Ready" },
                { id: "failed", label: "Failed" },
              ].map((filter) => (
                <button
                  key={filter.id}
                  className={`toolbar__item ${aiFilter === filter.id ? "toolbar__item--active" : ""}`}
                  onClick={() => onChangeAiFilter(filter.id as AiFilter)}
                  type="button"
                >
                  {filter.label}
                </button>
              ))}
              <button
                className="toolbar__item"
                disabled={selectedNoteIds.length === 0}
                onClick={() => onAddNotesToExamQueue(selectedNoteIds)}
                type="button"
              >
                Add queued to exams
              </button>
            </div>
          </div>
          <p className="surface__summary">
            AI enrichment runs after scans when enabled. It builds note briefs, tracks stale notes, and turns the course into a revision plan instead of a pile of files.
          </p>
          <div className="metric-strip">
            <Metric label="Ready" value={String(dashboard?.ai.readyNotes ?? 0)} />
            <Metric label="Pending" value={String(dashboard?.ai.pendingNotes ?? 0)} />
            <Metric label="Missing" value={String(dashboard?.ai.missingNotes ?? 0)} />
            <Metric label="Failed" value={String(dashboard?.ai.failedNotes ?? 0)} />
            <Metric label="Stale" value={String(dashboard?.ai.staleNotes ?? 0)} />
          </div>
        </section>

        {aiWorkbench ? (
          <section className="surface surface--split">
            <div>
              <div className="surface__header">
                <div>
                  <span className="surface__eyebrow">AI posture</span>
                  <h3>How usable the AI layer is right now</h3>
                </div>
              </div>
              <div className="metric-strip">
                <Metric label="Readiness" value={`${aiWorkbench.signals.readinessScore}%`} />
                <Metric label="Exam mode" value={aiWorkbench.signals.examMode} />
                <Metric label="Coverage" value={aiWorkbench.signals.insightDensity} />
              </div>
              <dl className="definition-grid">
                <Definition label="Flashcard potential" value={aiWorkbench.signals.flashcardPotential} />
                <Definition label="Cadence" value={aiWorkbench.signals.cadence} />
                <Definition label="Checkpoint" value={aiWorkbench.signals.checkpoint} />
                <Definition
                  label="Queue state"
                  value={`${dashboard?.ai.pendingNotes ?? 0} pending · ${dashboard?.ai.failedNotes ?? 0} failed`}
                />
              </dl>
            </div>

            <div>
              <div className="surface__header">
                <div>
                  <span className="surface__eyebrow">Command deck</span>
                  <h3>Turn the brief into actions</h3>
                </div>
              </div>
              <div className="action-list">
                {aiWorkbench.presets.map((preset) => (
                  <ActionRow
                    key={preset.id}
                    action={
                      <button
                        className="button button--subtle"
                        disabled={busyAction !== null || preset.noteIds.length === 0}
                        onClick={() => onApplyAiSelection(preset.noteIds, preset.title, preset.mode)}
                        type="button"
                      >
                        {preset.buttonLabel}
                      </button>
                    }
                    eyebrow="Preset"
                    meta={preset.stat}
                    title={preset.title}
                  >
                    {preset.description}
                  </ActionRow>
                ))}
              </div>
            </div>
          </section>
        ) : null}

        <section className="surface">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Course brief</span>
              <h3>{dashboard?.ai.summary ? "AI revision brief" : "No AI brief yet"}</h3>
            </div>
            <div className="button-row">
              <button
                className="button button--subtle"
                disabled={!selectedCourse || busyAction !== null || dashboard?.ai.status === "running"}
                onClick={onGenerateCourseAi}
                type="button"
              >
                {dashboard?.ai.status === "running" ? "Running..." : "Run AI"}
              </button>
              <button
                className="button button--ghost"
                disabled={!selectedCourse || busyAction !== null || dashboard?.ai.status === "running"}
                onClick={onRefreshCourseAi}
                type="button"
              >
                Refresh all
              </button>
            </div>
          </div>
          {dashboard?.ai.summary ? (
            <div className="insight-stack">
              <p className="inspector-copy">{dashboard.ai.summary}</p>
              <section className="insight-list">
                <strong>Revision priorities</strong>
                <ul>
                  {dashboard.ai.revisionPriorities.map((item) => (
                    <li key={item}>{item}</li>
                  ))}
                </ul>
              </section>
              <section className="insight-list">
                <strong>Weak spots</strong>
                <ul>
                  {dashboard.ai.weakSpots.map((item) => (
                    <li key={item}>{item}</li>
                  ))}
                </ul>
              </section>
              <section className="insight-list">
                <strong>Next actions</strong>
                <ul>
                  {dashboard.ai.nextActions.map((item) => (
                    <li key={item}>{item}</li>
                  ))}
                </ul>
              </section>
            </div>
          ) : (
            <EmptyState
              title={
                workspace.aiSettings?.enabled
                  ? dashboard?.ai.status === "running"
                    ? "AI is working in the background"
                    : "Start the first AI run"
                  : "AI is not enabled"
              }
              description={
                workspace.aiSettings?.enabled
                  ? "Run AI for this course to create note briefs and a course-level revision summary."
                  : "Enable AI in Setup and save reachable provider settings to unlock course enrichment."
              }
            />
          )}
        </section>

        {aiWorkbench ? (
          <section className="surface">
            <div className="surface__header">
              <div>
                <span className="surface__eyebrow">Study lanes</span>
                <h3>Three useful ways to work the course</h3>
              </div>
            </div>
            <div className="action-list">
              {aiWorkbench.lanes.map((lane) => (
                <ActionRow
                  key={lane.id}
                  action={
                    <button
                      className="button button--ghost"
                      disabled={busyAction !== null || lane.noteIds.length === 0}
                      onClick={() => onApplyAiSelection(lane.noteIds, lane.title, "replace")}
                      type="button"
                    >
                      {lane.buttonLabel}
                    </button>
                  }
                  eyebrow="Lane"
                  meta={`${lane.items.length} notes`}
                  title={lane.title}
                >
                  {lane.summary}
                </ActionRow>
              ))}
            </div>
          </section>
        ) : null}

        <section className="surface">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Note coverage</span>
              <h3>AI note queue</h3>
            </div>
          </div>
          {aiNotes.length ? (
            <div className="row-list row-list--compact">
              {aiNotes.map((note) => (
                <article key={note.id} className={`row-item ${selectedNoteId === note.id ? "row-item--active" : ""}`}>
                  <button className="row-item__main" onClick={() => onOpenAiNote(note.id)} type="button">
                    <div className="row-item__title-row">
                      <span className="row-item__title">{note.title}</span>
                      <span className={`soft-badge soft-badge--${aiStatusTone(note.aiStatus)}`}>
                        {aiStatusLabel(note.aiStatus)}
                      </span>
                      <span className="soft-badge soft-badge--neutral">
                        {getAiRecommendationLabel(note, weakNoteIds.has(note.id))}
                      </span>
                    </div>
                    <span className="row-item__subtitle">{note.relativePath}</span>
                  </button>
                  <div className="row-item__meta">
                    <span>{note.linkCount} links</span>
                    <span>{note.conceptCount} concepts</span>
                    <span>{note.formulaCount} formulas</span>
                  </div>
                  <button
                    aria-pressed={selectedNoteIds.includes(note.id)}
                    className={`button button--ghost ${selectedNoteIds.includes(note.id) ? "button--active" : ""}`}
                    onClick={() => onToggleNoteSelection(note.id)}
                    type="button"
                  >
                    {selectedNoteIds.includes(note.id) ? "Queued" : "Queue"}
                  </button>
                </article>
              ))}
            </div>
          ) : (
            <EmptyState
              title="No notes in this AI filter"
              description="Change the AI filter or run the course enrichment to populate this queue."
            />
          )}
        </section>
      </div>
    );
  }

  if (activeView === "outputs") {
    return (
      <div className="page-stack">
        {scanNotice}
        <section className="surface surface--hero">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Generate</span>
              <h3>Produce study material from the current course</h3>
            </div>
          </div>
          <p className="surface__summary">
            Generate outputs only when you need them. Nothing writes back to the vault until you trigger an action.
          </p>
        </section>

        <section className="surface">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Actions</span>
              <h3>Study outputs</h3>
            </div>
          </div>
          <div className="action-list">
            <ActionRow
              action={
                <button
                  className="button button--subtle"
                  disabled={!selectedCourse || selectedNoteIds.length === 0 || busyAction !== null}
                  onClick={onGenerateFlashcards}
                  type="button"
                >
                  {busyAction === "Flashcard generation failed" ? "Generating..." : "Generate flashcards"}
                </button>
              }
              eyebrow="Flashcards"
              meta={`${selectedNoteIds.length} queued notes`}
              title="Build markdown and Anki CSV exports"
            >
              Uses the current note queue and writes a markdown set plus a CSV export when enabled.
            </ActionRow>
            <ActionRow
              action={
                <button
                  className="button button--subtle"
                  disabled={!selectedCourse || busyAction !== null}
                  onClick={onCreateRevisionNote}
                  type="button"
                >
                  {busyAction === "Revision note generation failed" ? "Generating..." : "Create today’s revision note"}
                </button>
              }
              eyebrow="Revision"
              meta={selectedCourse?.revisionFolder ?? "Revision"}
              title="Write the daily revision note into the vault"
            >
              Includes the countdown, weak-link topics, and a ranked revision queue for the selected course.
            </ActionRow>
          </div>
        </section>

        <section className="surface surface--split">
          <div>
            <div className="surface__header">
              <div>
                <span className="surface__eyebrow">Latest flashcards</span>
                <h3>Recent export</h3>
              </div>
            </div>
            {flashcardResult ? (
              <dl className="definition-grid">
                <Definition label="Cards" value={String(flashcardResult.cardCount)} />
                <Definition label="Generated" value={formatDateTime(flashcardResult.generatedAt)} />
                <Definition label="Markdown" value={flashcardResult.markdownPath} />
                <Definition label="CSV" value={flashcardResult.csvPath ?? "No CSV"} />
              </dl>
            ) : (
              <EmptyState
                title="No flashcard export yet"
                description="Queue notes in Notes and run the flashcard action."
              />
            )}
          </div>
          <div>
            <div className="surface__header">
              <div>
                <span className="surface__eyebrow">Latest revision note</span>
                <h3>Recent daily plan</h3>
              </div>
            </div>
            {revisionResult ? (
              <dl className="definition-grid">
                <Definition label="Items" value={String(revisionResult.itemCount)} />
                <Definition label="Generated" value={formatDateTime(revisionResult.generatedAt)} />
                <Definition label="Path" value={revisionResult.notePath} />
              </dl>
            ) : (
              <EmptyState
                title="No revision note yet"
                description="Choose a course and create the first revision note from this page."
              />
            )}
          </div>
        </section>
      </div>
    );
  }

  if (activeView === "settings") {
    return (
      <div className="page-stack">
        {scanNotice}
        <section className="surface surface--hero">
          <span className="surface__eyebrow">Runtime</span>
          <h3>{isPreview ? "Preview mode" : "Desktop runtime ready"}</h3>
          <p>
            {isPreview
              ? "The browser preview is only for UI work. Live vault access, scans, and file writes run in the Tauri desktop app."
              : "Use the detail rail to connect the Obsidian vault and configure optional AI refinement."}
          </p>
        </section>

        <section className="surface surface--split">
          <div>
            <div className="surface__header">
              <div>
                <span className="surface__eyebrow">Vault state</span>
                <h3>Current connection</h3>
              </div>
            </div>
            {workspace.vault ? (
              <dl className="definition-grid">
                <Definition label="Vault path" value={workspace.vault.vaultPath} />
                <Definition label="Connected" value={formatDateTime(workspace.vault.connectedAt)} />
                <Definition label="Imported courses" value={String(workspace.courses.length)} />
                <Definition label="Last scan" value={workspace.scanStatus?.lastScanAt ? formatDateTime(workspace.scanStatus.lastScanAt) : "Never"} />
              </dl>
            ) : (
              <EmptyState
                title="No vault connected"
                description="Paste or browse to the vault path in the detail rail to start working with live notes."
              />
            )}
          </div>
          <div>
            <div className="surface__header">
              <div>
                <span className="surface__eyebrow">AI</span>
                <h3>Optional refinement</h3>
              </div>
            </div>
            {workspace.aiSettings ? (
              <dl className="definition-grid">
                <Definition label="Model" value={workspace.aiSettings.model} />
                <Definition label="Base URL" value={workspace.aiSettings.baseUrl} />
                <Definition label="Enabled" value={workspace.aiSettings.enabled ? "Yes" : "No"} />
                <Definition label="Timeout" value={`${workspace.aiSettings.timeoutMs} ms`} />
              </dl>
            ) : (
              <EmptyState
                title="AI refinement is off"
                description="The deterministic local extraction pipeline works without any model configured."
              />
            )}
          </div>
        </section>
      </div>
    );
  }

  return (
    <div className="page-stack">
      {scanNotice}
      <section className="surface surface--hero">
        <div className="overview-hero">
          <div>
            <span className="surface__eyebrow">Active course</span>
            <h3>{selectedCourse?.name ?? "No course selected"}</h3>
            <p>
              {selectedCourse
                ? `${selectedCourse.folder} · ${dashboard?.graph.noteCount ?? 0} indexed notes`
                : "Choose a course from the sidebar to review its scan and revision state."}
            </p>
          </div>
          <div className="countdown-panel">
            <strong>{dashboard?.countdown.daysRemaining ?? "--"}</strong>
            <span>{dashboard?.countdown.label ?? "No exam scheduled"}</span>
          </div>
        </div>
      </section>

      <section className="surface">
        <div className="metric-strip">
          <Metric
            label="Concept coverage"
            value={dashboard ? `${clampPercent(dashboard.coverage.percentage)}%` : "--"}
          />
          <Metric
            label="Anchored concepts"
            value={
              dashboard
                ? `${dashboard.coverage.coveredConcepts}/${dashboard.coverage.totalConcepts}`
                : "--"
            }
          />
          <Metric
            label="Graph links"
            value={dashboard ? String(dashboard.graph.edgeCount) : "--"}
          />
          <Metric
            label="Weak notes"
            value={dashboard ? String(dashboard.weakNotes.length) : "--"}
          />
          <Metric
            label="Flashcards"
            value={dashboard ? String(dashboard.flashcards.totalCards) : "--"}
          />
          <Metric
            label="Last scan"
            value={scanReport ? formatDateTime(scanReport.scannedAt) : workspace.scanStatus?.lastScanAt ? formatDateTime(workspace.scanStatus.lastScanAt) : "Never"}
          />
        </div>
      </section>

      <section className="surface surface--split">
        <div>
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Weak links</span>
              <h3>Notes that need stronger structure</h3>
            </div>
          </div>
          {dashboard?.weakNotes.length ? (
            <div className="line-list">
              {dashboard.weakNotes.map((note) => (
                <button key={note.noteId} className="line-item" onClick={() => onOpenNote(note.noteId)} type="button">
                  <span className="line-item__title">{note.title}</span>
                  <span className="line-item__subtitle">{note.relativePath}</span>
                  <span className="line-item__meta">
                    Weakness {note.score.toFixed(2)} · {note.suggestions.join(" · ")}
                  </span>
                </button>
              ))}
            </div>
          ) : (
            <EmptyState
              title="No weak notes surfaced"
              description="The graph looks healthy for the selected course."
            />
          )}
        </div>
        <div>
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Priority topics</span>
              <h3>Concepts with the most support</h3>
            </div>
          </div>
          {dashboard?.topConcepts.length ? (
            <div className="line-list">
              {dashboard.topConcepts.map((concept) => (
                <div key={concept.name} className="line-item line-item--static">
                  <span className="line-item__title">{concept.name}</span>
                  <span className="line-item__meta">
                    {concept.noteCount} notes · support {concept.supportScore.toFixed(1)}
                  </span>
                </div>
              ))}
            </div>
          ) : (
            <EmptyState
              title="No concepts extracted"
              description="Run a scan to populate the concept view."
            />
          )}
        </div>
      </section>

      <section className="surface surface--split">
        <div>
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Recent notes</span>
              <h3>Continue review</h3>
            </div>
          </div>
          {dashboard?.notes.length ? (
            <div className="row-list row-list--compact">
              {dashboard.notes.slice(0, 6).map((note) => (
                <article key={note.id} className={`row-item ${selectedNoteId === note.id ? "row-item--active" : ""}`}>
                  <button className="row-item__main" onClick={() => onOpenNote(note.id)} type="button">
                    <div className="row-item__title-row">
                      <span className="row-item__title">{note.title}</span>
                      {selectedNoteIds.includes(note.id) ? <span className="soft-badge">Queued</span> : null}
                    </div>
                    <span className="row-item__subtitle">{note.relativePath}</span>
                  </button>
                  <div className="row-item__meta">
                    <span>{note.linkCount} links</span>
                    <span>{note.conceptCount} concepts</span>
                    <span>{note.formulaCount} formulas</span>
                  </div>
                  <button
                    className={`button button--ghost ${selectedNoteIds.includes(note.id) ? "button--active" : ""}`}
                    onClick={() => onToggleNoteSelection(note.id)}
                    type="button"
                  >
                    {selectedNoteIds.includes(note.id) ? "Queued" : "Queue"}
                  </button>
                </article>
              ))}
            </div>
          ) : (
            <EmptyState
              title="No notes indexed"
              description={isPreview ? "Refresh the preview or run a preview scan to populate demo notes." : "Run a scan to build the note library."}
            />
          )}
        </div>
        <div>
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Formula list</span>
              <h3>Most visible formulas</h3>
            </div>
          </div>
          {dashboard?.formulas.length ? (
            <div className="line-list">
              {dashboard.formulas.map((formula) => (
                <div key={formula.latex} className="line-item line-item--static">
                  <MathFormula
                    className="line-item__math"
                    latex={formula.latex}
                    showSource={false}
                    sourceClassName="math-formula__source line-item__title line-item__code"
                  />
                  <span className="line-item__meta">{formula.noteCount} notes</span>
                </div>
              ))}
            </div>
          ) : (
            <EmptyState
              title="No formulas extracted"
              description="Formulas appear here after the scan finds inline or block math."
            />
          )}
        </div>
      </section>
    </div>
  );
}

function EmptyState({ title, description }: { title: string; description: string }) {
  return (
    <div className="empty-pane">
      <strong>{title}</strong>
      <p>{description}</p>
    </div>
  );
}

function Definition({ label, value }: { label: string; value: string }) {
  return (
    <div className="definition-item">
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="metric">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function aiStatusLabel(status: string) {
  switch (status) {
    case "complete":
      return "Ready";
    case "running":
    case "queued":
      return "Running";
    case "failed":
      return "Failed";
    case "stale":
      return "Stale";
    default:
      return "Missing";
  }
}

function aiStatusTone(status: string) {
  switch (status) {
    case "complete":
      return "success";
    case "failed":
      return "warning";
    case "stale":
      return "warning";
    default:
      return "neutral";
  }
}

function ActionRow({
  action,
  children,
  eyebrow,
  meta,
  title,
}: {
  action: ReactNode;
  children: ReactNode;
  eyebrow: string;
  meta: string;
  title: string;
}) {
  return (
    <div className="action-row">
      <div className="action-row__copy">
        <span className="surface__eyebrow">{eyebrow}</span>
        <strong>{title}</strong>
        <p>{children}</p>
      </div>
      <div className="action-row__aside">
        <span>{meta}</span>
        {action}
      </div>
    </div>
  );
}
