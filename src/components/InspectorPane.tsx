import type { ReactNode } from "react";
import { formatDate, formatDateTime, shortenPath, type CourseDraft } from "../lib";
import type {
  AiSettingsInput,
  CourseConfig,
  FlashcardGenerationResult,
  NoteDetails,
  RevisionNoteResult,
  ScanReport,
  WorkspaceSnapshot,
} from "../types";
import type { AppView } from "./appShell";

type EditableCourseField = "name" | "folder" | "examDate" | "revisionFolder" | "flashcardsFolder";

type InspectorPaneProps = {
  activeView: AppView;
  aiDraft: AiSettingsInput;
  busyAction: string | null;
  courseDraft: CourseDraft;
  flashcardResult: FlashcardGenerationResult | null;
  hasSavedApiKey: boolean;
  noteDetails: NoteDetails | null;
  revisionResult: RevisionNoteResult | null;
  runtimeMode: "tauri" | "browser-preview";
  scanReport: ScanReport | null;
  selectedCourse: CourseConfig | null;
  selectedNoteId: string | null;
  selectedNoteIds: string[];
  vaultPath: string;
  workspace: WorkspaceSnapshot;
  onBrowseVault: () => void;
  onConnectVault: () => void;
  onDeleteCourse: (courseId: string) => void;
  onDisconnectVault: () => void;
  onGenerateNoteAiInsight: () => void;
  onResetCourseDraft: () => void;
  onSaveAiSettings: () => void;
  onSaveCourse: () => void;
  onUpdateAiField: <K extends keyof AiSettingsInput>(field: K, value: AiSettingsInput[K]) => void;
  onUpdateCourseField: (field: EditableCourseField, value: string) => void;
  onValidateAiSettings: () => void;
  onVaultPathChange: (value: string) => void;
};

export function InspectorPane({
  activeView,
  aiDraft,
  busyAction,
  courseDraft,
  flashcardResult,
  hasSavedApiKey,
  noteDetails,
  revisionResult,
  runtimeMode,
  scanReport,
  selectedCourse,
  selectedNoteId,
  selectedNoteIds,
  vaultPath,
  workspace,
  onBrowseVault,
  onConnectVault,
  onDeleteCourse,
  onDisconnectVault,
  onGenerateNoteAiInsight,
  onResetCourseDraft,
  onSaveAiSettings,
  onSaveCourse,
  onUpdateAiField,
  onUpdateCourseField,
  onValidateAiSettings,
  onVaultPathChange,
}: InspectorPaneProps) {
  const isPreview = runtimeMode === "browser-preview";
  const dashboard = workspace.dashboard;

  return (
    <aside className="inspector-pane">
      {activeView === "overview" ? (
        <>
          <InspectorSection eyebrow="Current focus" title={selectedCourse?.name ?? "No course selected"}>
            {selectedCourse ? (
              <dl className="inspector-grid">
                <InspectorItem label="Folder" value={selectedCourse.folder} />
                <InspectorItem
                  label="Exam"
                  value={dashboard?.countdown.examDate ? formatDate(dashboard.countdown.examDate) : "No exam"}
                />
                <InspectorItem
                  label="Coverage"
                  value={dashboard ? `${Math.round(dashboard.coverage.percentage)}%` : "No scan"}
                />
                <InspectorItem
                  label="Links"
                  value={dashboard ? `${dashboard.graph.edgeCount} edges` : "No graph"}
                />
              </dl>
            ) : (
              <p className="inspector-copy">Select a course from the sidebar to load its note graph and revision state.</p>
            )}
          </InspectorSection>

          <InspectorSection eyebrow="Recent scan" title="Index status">
            {workspace.scanStatus?.lastScanAt ? (
              <dl className="inspector-grid">
                <InspectorItem label="Scanned" value={formatDateTime(workspace.scanStatus.lastScanAt)} />
                <InspectorItem label="Notes" value={String(workspace.scanStatus.noteCount)} />
                <InspectorItem label="Changed" value={String(workspace.scanStatus.changedCount)} />
                <InspectorItem label="Removed" value={String(workspace.scanStatus.removedCount)} />
              </dl>
            ) : (
              <p className="inspector-copy">Run the first scan to populate the index, graph links, and weak-note suggestions.</p>
            )}
          </InspectorSection>
        </>
      ) : null}

      {activeView === "courses" ? (
        <>
          <InspectorSection eyebrow="Course editor" title={courseDraft.id ? "Edit course" : "Create course"}>
            <div className="form-grid">
              <Field label="Name">
                <input
                  onChange={(event) => onUpdateCourseField("name", event.target.value)}
                  placeholder="Mathe 1"
                  type="text"
                  value={courseDraft.name}
                />
              </Field>
              <Field label="Folder">
                <input
                  onChange={(event) => onUpdateCourseField("folder", event.target.value)}
                  placeholder="Mathe 1"
                  type="text"
                  value={courseDraft.folder}
                />
              </Field>
              <Field label="Exam date">
                <input
                  onChange={(event) => onUpdateCourseField("examDate", event.target.value)}
                  type="date"
                  value={courseDraft.examDate}
                />
              </Field>
              <Field label="Revision folder">
                <input
                  onChange={(event) => onUpdateCourseField("revisionFolder", event.target.value)}
                  type="text"
                  value={courseDraft.revisionFolder}
                />
              </Field>
              <Field label="Flashcards folder">
                <input
                  onChange={(event) => onUpdateCourseField("flashcardsFolder", event.target.value)}
                  type="text"
                  value={courseDraft.flashcardsFolder}
                />
              </Field>
            </div>
            <div className="button-row">
              <button className="button button--subtle" disabled={busyAction !== null} onClick={onSaveCourse} type="button">
                {busyAction === "Course save failed" ? "Saving..." : "Save course"}
              </button>
              <button className="button button--ghost" disabled={busyAction !== null} onClick={onResetCourseDraft} type="button">
                Reset
              </button>
              {courseDraft.id ? (
                <button
                  className="button button--ghost button--danger"
                  disabled={busyAction !== null}
                  onClick={() => onDeleteCourse(courseDraft.id!)}
                  type="button"
                >
                  Delete
                </button>
              ) : null}
            </div>
          </InspectorSection>

          <InspectorSection eyebrow="Selected course" title={selectedCourse?.name ?? "None selected"}>
            {selectedCourse ? (
              <dl className="inspector-grid">
                <InspectorItem label="Folder" value={selectedCourse.folder} />
                <InspectorItem
                  label="Exam"
                  value={selectedCourse.examDate ? formatDate(selectedCourse.examDate) : "No date"}
                />
                <InspectorItem label="Coverage" value={`${Math.round(selectedCourse.coverage)}%`} />
                <InspectorItem label="Weak notes" value={String(selectedCourse.weakNoteCount)} />
              </dl>
            ) : (
              <p className="inspector-copy">Choose a course from the library to edit its settings here.</p>
            )}
          </InspectorSection>
        </>
      ) : null}

      {activeView === "notes" ? (
        <>
          <InspectorSection eyebrow="Selected note" title={noteDetails?.title ?? "Choose a note"}>
            {selectedNoteId && !noteDetails ? (
              <p className="inspector-copy">Loading note details...</p>
            ) : noteDetails ? (
              <>
                <p className="inspector-copy">{noteDetails.excerpt}</p>
                <dl className="inspector-grid">
                  <InspectorItem label="Path" value={noteDetails.relativePath} />
                  <InspectorItem label="Queued" value={selectedNoteIds.includes(noteDetails.id) ? "Yes" : "No"} />
                </dl>
              </>
            ) : (
              <p className="inspector-copy">Select a note to inspect its extracted structure and relationships.</p>
            )}
          </InspectorSection>

          {noteDetails ? (
            <>
              <InspectorSection eyebrow="Structure" title="Headings">
                <TokenList items={noteDetails.headings} emptyLabel="No headings extracted" />
              </InspectorSection>
              <InspectorSection eyebrow="Knowledge" title="Concepts and formulas">
                <TokenList items={noteDetails.concepts} emptyLabel="No concepts extracted" />
                <TokenList code items={noteDetails.formulas} emptyLabel="No formulas extracted" />
              </InspectorSection>
              <InspectorSection eyebrow="Connections" title="Links and suggestions">
                <TokenList items={noteDetails.links} emptyLabel="No outgoing links" />
                <TokenList items={noteDetails.suggestions} emptyLabel="No link suggestions" />
              </InspectorSection>
              <InspectorSection eyebrow="AI study coach" title="Exam brief">
                {workspace.aiSettings?.enabled && hasSavedApiKey ? (
                  <>
                    <div className="button-row">
                      <button
                        className="button button--subtle"
                        disabled={busyAction !== null}
                        onClick={onGenerateNoteAiInsight}
                        type="button"
                      >
                        {busyAction === "AI note insight failed"
                          ? "Generating..."
                          : noteDetails.aiInsight
                            ? "Refresh AI brief"
                            : "Generate AI brief"}
                      </button>
                    </div>
                    {noteDetails.aiInsight ? (
                      <div className="insight-stack">
                        <p className="inspector-copy">{noteDetails.aiInsight.summary}</p>
                        <InsightList
                          items={noteDetails.aiInsight.takeaways}
                          title="What to remember"
                        />
                        <InsightList
                          items={noteDetails.aiInsight.examQuestions}
                          title="Exam questions"
                        />
                        <InsightList
                          items={noteDetails.aiInsight.connectionOpportunities}
                          title="Connection opportunities"
                        />
                        <p className="inspector-copy">
                          {noteDetails.aiInsight.model} · {formatDateTime(noteDetails.aiInsight.generatedAt)}
                        </p>
                      </div>
                    ) : (
                      <p className="inspector-copy">
                        Generate a note brief with takeaways, exam-style questions, and linking suggestions.
                      </p>
                    )}
                  </>
                ) : (
                  <p className="inspector-copy">
                    Enable AI in Setup and save a valid API key to use the note coach.
                  </p>
                )}
              </InspectorSection>
            </>
          ) : null}
        </>
      ) : null}

      {activeView === "outputs" ? (
        <>
          <InspectorSection eyebrow="Queue" title="Output context">
            <dl className="inspector-grid">
              <InspectorItem label="Course" value={selectedCourse?.name ?? "No course"} />
              <InspectorItem label="Queued notes" value={String(selectedNoteIds.length)} />
              <InspectorItem label="Flashcards folder" value={selectedCourse?.flashcardsFolder ?? "Flashcards"} />
              <InspectorItem label="Revision folder" value={selectedCourse?.revisionFolder ?? "Revision"} />
            </dl>
          </InspectorSection>

          <InspectorSection eyebrow="Latest files" title="Generated paths">
            <dl className="inspector-grid inspector-grid--single">
              <InspectorItem
                label="Flashcard export"
                value={shortenPath(flashcardResult?.markdownPath ?? dashboard?.flashcards.exportPath ?? "Not generated")}
              />
              <InspectorItem
                label="Revision note"
                value={shortenPath(revisionResult?.notePath ?? dashboard?.revision.notePath ?? "Not generated")}
              />
            </dl>
          </InspectorSection>
        </>
      ) : null}

      {activeView === "settings" ? (
        <>
          <InspectorSection eyebrow="Vault access" title={isPreview ? "Preview only" : "Connect local vault"}>
            <div className="form-grid">
              <Field label="Vault path">
                <input
                  onChange={(event) => onVaultPathChange(event.target.value)}
                  placeholder="C:\\Users\\you\\ObsidianVault\\Uni"
                  readOnly={isPreview}
                  type="text"
                  value={vaultPath}
                />
              </Field>
            </div>
            <div className="button-row">
              <button
                className="button button--subtle"
                disabled={busyAction !== null || isPreview}
                onClick={onConnectVault}
                type="button"
              >
                {busyAction === "Vault connection failed" ? "Connecting..." : "Connect vault"}
              </button>
              <button
                className="button button--ghost"
                disabled={isPreview || busyAction !== null}
                onClick={onBrowseVault}
                type="button"
              >
                Browse
              </button>
              <button
                className="button button--ghost"
                disabled={!workspace.vault || busyAction !== null}
                onClick={onDisconnectVault}
                type="button"
              >
                Disconnect
              </button>
            </div>
            <p className="inspector-copy">
              {isPreview
                ? "Run the Tauri desktop app to browse the filesystem and connect a real vault."
                : "After connection, the app imports top-level course folders and lets you scan them into the local index."}
            </p>
          </InspectorSection>

          <InspectorSection eyebrow="AI refinement" title="Optional model configuration">
            <div className="form-grid">
              <Field label="Base URL">
                <input
                  onChange={(event) => onUpdateAiField("baseUrl", event.target.value)}
                  type="text"
                  value={aiDraft.baseUrl}
                />
              </Field>
              <Field label="Model">
                <input
                  onChange={(event) => onUpdateAiField("model", event.target.value)}
                  type="text"
                  value={aiDraft.model}
                />
              </Field>
              <Field label="API key">
                <input
                  onChange={(event) => onUpdateAiField("apiKey", event.target.value)}
                  placeholder={hasSavedApiKey ? "Saved key on file" : "Paste key"}
                  type="password"
                  value={aiDraft.apiKey ?? ""}
                />
              </Field>
              <Field label="Timeout (ms)">
                <input
                  min={1000}
                  onChange={(event) => onUpdateAiField("timeoutMs", Number(event.target.value) || 0)}
                  type="number"
                  value={aiDraft.timeoutMs}
                />
              </Field>
              <label className="toggle">
                <input
                  checked={aiDraft.enabled}
                  onChange={(event) => onUpdateAiField("enabled", event.target.checked)}
                  type="checkbox"
                />
                <span>Enable AI refinement</span>
              </label>
            </div>
            <div className="button-row">
              <button className="button button--ghost" disabled={busyAction !== null} onClick={onValidateAiSettings} type="button">
                Validate
              </button>
              <button className="button button--subtle" disabled={busyAction !== null} onClick={onSaveAiSettings} type="button">
                Save settings
              </button>
            </div>
            <p className="inspector-copy">
              The app can use a saved key here or a local `OPENAI_API_KEY` environment variable.
            </p>
          </InspectorSection>
        </>
      ) : null}

      {scanReport ? (
        <InspectorSection eyebrow="Latest run" title="Scan report">
          <dl className="inspector-grid">
            <InspectorItem label="Scanned" value={String(scanReport.scannedNotes)} />
            <InspectorItem label="Changed" value={String(scanReport.changedNotes)} />
            <InspectorItem label="Edges" value={String(scanReport.generatedEdges)} />
            <InspectorItem label="Weak links" value={String(scanReport.generatedWeakLinks)} />
          </dl>
        </InspectorSection>
      ) : null}
    </aside>
  );
}

function InspectorSection({
  eyebrow,
  title,
  children,
}: {
  eyebrow: string;
  title: string;
  children: ReactNode;
}) {
  return (
    <section className="inspector-section">
      <span className="surface__eyebrow">{eyebrow}</span>
      <h3>{title}</h3>
      {children}
    </section>
  );
}

function InspectorItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="inspector-item">
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="field">
      <span>{label}</span>
      {children}
    </label>
  );
}

function TokenList({
  code = false,
  emptyLabel,
  items,
}: {
  code?: boolean;
  emptyLabel: string;
  items: string[];
}) {
  if (!items.length) {
    return <p className="inspector-copy">{emptyLabel}</p>;
  }

  return (
    <div className="token-list">
      {items.map((item) =>
        code ? (
          <code key={item} className="token-list__item token-list__item--code">
            {item}
          </code>
        ) : (
          <span key={item} className="token-list__item">
            {item}
          </span>
        ),
      )}
    </div>
  );
}

function InsightList({ items, title }: { items: string[]; title: string }) {
  if (!items.length) {
    return null;
  }

  return (
    <div className="insight-list">
      <strong>{title}</strong>
      <ul>
        {items.map((item) => (
          <li key={`${title}-${item}`}>{item}</li>
        ))}
      </ul>
    </div>
  );
}
