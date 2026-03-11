import { useEffect, useState } from "react";
import { formatDateTime, shortenPath } from "../lib";
import type {
  CourseConfig,
  FormulaBrief,
  FormulaDetails,
  FormulaLinkedNote,
  FormulaSummary,
  FormulaWorkspaceSnapshot,
  NoteChunkPreview,
} from "../types";
import { MathFormula } from "./MathFormula";

type FormulaWorkspaceProps = {
  aiEnabled: boolean;
  busyAction: string | null;
  formulaDetails: FormulaDetails | null;
  formulaWorkspace: FormulaWorkspaceSnapshot | null;
  selectedCourse: CourseConfig | null;
  selectedFormulaId: string | null;
  onGenerateBrief: (formulaId: string) => void;
  onOpenNote: (noteId: string) => void;
  onSelectFormula: (formulaId: string | null) => void;
};

type ReaderMode = "split" | "maximized";
type SortMode = "coverage" | "alphabetical";
type BriefTab = "coach" | "practice" | "derivation";

export function FormulaWorkspace({
  aiEnabled,
  busyAction,
  formulaDetails,
  formulaWorkspace,
  selectedCourse,
  selectedFormulaId,
  onGenerateBrief,
  onOpenNote,
  onSelectFormula,
}: FormulaWorkspaceProps) {
  const [readerMode, setReaderMode] = useState<ReaderMode>("split");
  const [sortMode, setSortMode] = useState<SortMode>("coverage");
  const [search, setSearch] = useState("");
  const [briefTab, setBriefTab] = useState<BriefTab>("coach");

  useEffect(() => {
    if (!formulaWorkspace?.formulas.length) {
      if (selectedFormulaId) {
        onSelectFormula(null);
      }
      return;
    }

    if (!selectedFormulaId || !formulaWorkspace.formulas.some((formula) => formula.id === selectedFormulaId)) {
      onSelectFormula(formulaWorkspace.formulas[0]?.id ?? null);
    }
  }, [formulaWorkspace, onSelectFormula, selectedFormulaId]);

  useEffect(() => {
    setBriefTab("coach");
  }, [formulaDetails?.id]);

  if (!selectedCourse) {
    return (
      <div className="page-stack">
        <section className="surface surface--hero">
          <span className="surface__eyebrow">Formulas</span>
          <h3>Select a course first</h3>
          <p>Choose a course to build a dedicated formula library for that workspace.</p>
        </section>
      </div>
    );
  }

  const filteredFormulas = (formulaWorkspace?.formulas ?? [])
    .filter((formula) => {
      if (!search.trim()) {
        return true;
      }
      const query = search.toLowerCase();
      return (
        formula.latex.toLowerCase().includes(query) ||
        formula.sourceNoteTitles.some((title) => title.toLowerCase().includes(query))
      );
    })
    .sort((left, right) =>
      sortMode === "alphabetical"
        ? left.latex.localeCompare(right.latex)
        : right.noteCount - left.noteCount || left.latex.localeCompare(right.latex),
    );

  return (
    <div className="page-stack">
      <section className="surface surface--hero">
        <div className="surface__header">
          <div>
            <span className="surface__eyebrow">Formulas</span>
            <h3>{selectedCourse.name} formula library</h3>
          </div>
          <div className="button-row">
            <button
              className={`button button--ghost ${readerMode === "maximized" ? "button--active" : ""}`}
              onClick={() => setReaderMode((current) => (current === "split" ? "maximized" : "split"))}
              type="button"
            >
              {readerMode === "split" ? "Maximize reader" : "Split view"}
            </button>
          </div>
        </div>
        <p className="surface__summary">
          Browse every extracted formula in the current course, inspect where it appears, and generate a cached AI explanation bundle when you need one.
        </p>
        <div className="metric-strip">
          <Metric label="Unique formulas" value={String(formulaWorkspace?.summary.formulaCount ?? 0)} />
          <Metric label="Formula mentions" value={String(formulaWorkspace?.summary.formulaMentions ?? 0)} />
          <Metric label="Notes with math" value={String(formulaWorkspace?.summary.notesWithFormulas ?? 0)} />
          <Metric label="AI briefs" value={String(formulaWorkspace?.summary.briefedCount ?? 0)} />
        </div>
      </section>

      <section className={`surface formula-workspace formula-workspace--${readerMode}`}>
        <div className={`formula-workspace__rail ${readerMode === "maximized" ? "formula-workspace__rail--hidden" : ""}`}>
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Library</span>
              <h3>Course formulas</h3>
            </div>
          </div>
          <div className="formula-controls">
            <label className="field">
              <span>Search</span>
              <input
                onChange={(event) => setSearch(event.target.value)}
                placeholder="Search LaTeX or note title"
                type="text"
                value={search}
              />
            </label>
            <label className="field">
              <span>Sort</span>
              <select
                onChange={(event) => setSortMode(event.target.value as SortMode)}
                value={sortMode}
              >
                <option value="coverage">Most used</option>
                <option value="alphabetical">Alphabetical</option>
              </select>
            </label>
          </div>
          {filteredFormulas.length ? (
            <div className="row-list row-list--compact formula-list">
              {filteredFormulas.map((formula) => (
                <FormulaListRow
                  key={formula.id}
                  formula={formula}
                  isActive={selectedFormulaId === formula.id}
                  onSelect={() => onSelectFormula(formula.id)}
                />
              ))}
            </div>
          ) : (
            <EmptyState
              title="No formulas in this view"
              description="Change the search term or run a scan to refresh the extracted math."
            />
          )}
        </div>

        <div className="formula-workspace__main">
          {formulaDetails ? (
            <>
              <section className="formula-detail-card">
                <div className="surface__header">
                  <div>
                    <span className="surface__eyebrow">Selected formula</span>
                    <h3>Context and explanation</h3>
                  </div>
                  <div className="button-row">
                    <button
                      className="button button--ghost"
                      onClick={() => void navigator.clipboard.writeText(formulaDetails.latex)}
                      type="button"
                    >
                      Copy LaTeX
                    </button>
                    <button
                      className="button button--subtle"
                      disabled={!aiEnabled || busyAction !== null}
                      onClick={() => onGenerateBrief(formulaDetails.id)}
                      type="button"
                    >
                      {busyAction === "Formula brief generation failed"
                        ? "Generating..."
                        : formulaDetails.brief
                          ? "Refresh AI brief"
                          : "Generate AI brief"}
                    </button>
                  </div>
                </div>
                <div className="formula-display">
                  <MathFormula
                    className="formula-display__math"
                    latex={formulaDetails.latex}
                    showSource
                    sourceClassName="math-formula__source formula-display__source"
                  />
                </div>
                <dl className="definition-grid">
                  <Definition label="Appears in notes" value={String(formulaDetails.noteCount)} />
                  <Definition label="Related concepts" value={String(formulaDetails.relatedConcepts.length)} />
                  <Definition label="Heading anchors" value={String(formulaDetails.headings.length)} />
                  <Definition
                    label="AI brief"
                    value={
                      formulaDetails.brief
                        ? `${formulaDetails.brief.model} · ${formatDateTime(formulaDetails.brief.generatedAt)}`
                        : aiEnabled
                          ? "Not generated yet"
                          : "AI disabled"
                    }
                  />
                </dl>
              </section>

              <section className="surface surface--split">
                <div>
                  <div className="surface__header">
                    <div>
                      <span className="surface__eyebrow">Linked notes</span>
                      <h3>Where this formula lives</h3>
                    </div>
                  </div>
                  <div className="line-list">
                    {formulaDetails.linkedNotes.map((note) => (
                      <FormulaNoteRow key={note.noteId} note={note} onOpenNote={onOpenNote} />
                    ))}
                  </div>
                </div>
                <div>
                  <div className="surface__header">
                    <div>
                      <span className="surface__eyebrow">Source chunks</span>
                      <h3>Reader context</h3>
                    </div>
                  </div>
                  <div className="formula-chunk-list">
                    {formulaDetails.chunks.map((chunk) => (
                      <FormulaChunkCard key={chunk.chunkId} chunk={chunk} onOpenNote={onOpenNote} />
                    ))}
                  </div>
                </div>
              </section>

              <section className="surface">
                <div className="surface__header">
                  <div>
                    <span className="surface__eyebrow">AI brief</span>
                    <h3>Coach, practice, and derivation</h3>
                  </div>
                </div>
                {formulaDetails.brief ? (
                  <>
                    <div className="toolbar">
                      {(["coach", "practice", "derivation"] as BriefTab[]).map((tab) => (
                        <button
                          key={tab}
                          className={`toolbar__item ${briefTab === tab ? "toolbar__item--active" : ""}`}
                          onClick={() => setBriefTab(tab)}
                          type="button"
                        >
                          {capitalize(tab)}
                        </button>
                      ))}
                    </div>
                    <FormulaBriefPanel brief={formulaDetails.brief} tab={briefTab} />
                  </>
                ) : (
                  <EmptyState
                    title={aiEnabled ? "No AI brief yet" : "AI is disabled"}
                    description={
                      aiEnabled
                        ? "Generate the brief to get a structured explanation, practice prompts, and a derivation outline."
                        : "Enable AI in Setup to unlock per-formula coaching."
                    }
                  />
                )}
              </section>
            </>
          ) : (
            <EmptyState
              title="Choose a formula"
              description="Select a formula from the library to inspect its note context and AI brief."
            />
          )}
        </div>
      </section>
    </div>
  );
}

function FormulaListRow({
  formula,
  isActive,
  onSelect,
}: {
  formula: FormulaSummary;
  isActive: boolean;
  onSelect: () => void;
}) {
  return (
    <article className={`row-item formula-list-row ${isActive ? "row-item--active formula-list-row--active" : ""}`}>
      <button className="row-item__main" onClick={onSelect} type="button">
        <div className="row-item__title-row">
          <MathFormula
            className="line-item__math formula-list-row__math"
            latex={formula.latex}
            showSource={false}
            sourceClassName="math-formula__source line-item__title line-item__code"
          />
          <span className="soft-badge">{formula.noteCount} notes</span>
        </div>
        <span className="row-item__subtitle">{formula.sourceNoteTitles.slice(0, 3).join(" · ")}</span>
      </button>
    </article>
  );
}

function FormulaNoteRow({
  note,
  onOpenNote,
}: {
  note: FormulaLinkedNote;
  onOpenNote: (noteId: string) => void;
}) {
  return (
    <button className="line-item formula-note-card" onClick={() => onOpenNote(note.noteId)} type="button">
      <span className="line-item__title">{note.title}</span>
      <span className="line-item__subtitle">{shortenPath(note.relativePath)}</span>
      <p className="formula-note-card__summary">{formatPreviewText(note.excerpt, 220)}</p>
      <div className="formula-chip-list">
        {note.headings.slice(0, 3).map((heading) => (
          <span key={`${note.noteId}-${heading}`} className="formula-chip">
            {heading}
          </span>
        ))}
        {note.relatedConcepts.slice(0, 2).map((concept) => (
          <span key={`${note.noteId}-${concept}`} className="formula-chip formula-chip--muted">
            {concept}
          </span>
        ))}
      </div>
      <span className="line-item__meta formula-note-card__meta">{note.formulaCount} formulas in this note</span>
    </button>
  );
}

function FormulaChunkCard({
  chunk,
  onOpenNote,
}: {
  chunk: NoteChunkPreview;
  onOpenNote: (noteId: string) => void;
}) {
  return (
    <article className="formula-chunk">
      <div className="surface__header">
        <div>
          <span className="surface__eyebrow">{chunk.headingPath}</span>
          <h3>{chunk.noteTitle}</h3>
        </div>
        <button className="button button--ghost" onClick={() => onOpenNote(chunk.noteId)} type="button">
          Open note
        </button>
      </div>
      <p className="formula-chunk__body">{formatPreviewText(chunk.text, 340)}</p>
      <div className="formula-chip-list">
        <span className="formula-chip">{chunk.headingPath}</span>
        <span className="formula-chip formula-chip--muted">Chunk {chunk.ordinal + 1}</span>
      </div>
      <span className="line-item__meta">{shortenPath(chunk.relativePath)}</span>
    </article>
  );
}

function FormulaBriefPanel({ brief, tab }: { brief: FormulaBrief; tab: BriefTab }) {
  if (tab === "coach") {
    return (
      <div className="insight-stack">
        <p className="inspector-copy">{brief.coach.meaning}</p>
        <InsightList items={brief.coach.symbolBreakdown} title="Symbol breakdown" />
        <InsightList items={brief.coach.useCases} title="Use cases" />
        <InsightList items={brief.coach.pitfalls} title="Pitfalls" />
      </div>
    );
  }

  if (tab === "practice") {
    return (
      <div className="insight-stack">
        <InsightList items={brief.practice.recallPrompts} title="Recall prompts" />
        <InsightList items={brief.practice.shortAnswerDrills} title="Short-answer drills" />
        <InsightList items={brief.practice.multipleChoiceChecks} title="MCQ prompts" />
      </div>
    );
  }

  return (
    <div className="insight-stack">
      <p className="inspector-copy">{brief.derivation.intuition}</p>
      <InsightList items={brief.derivation.assumptions} title="Assumptions" />
      <InsightList items={brief.derivation.outline} title="Outline" />
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

function capitalize(value: string) {
  return value.slice(0, 1).toUpperCase() + value.slice(1);
}

function formatPreviewText(value: string, maxLength: number) {
  const cleaned = value
    .replace(/\[\[([^\]]+)\]\]/g, "$1")
    .replace(/#+\s*/g, "")
    .replace(/\*\*/g, "")
    .replace(/__+/g, "")
    .replace(/\$[^$]+\$/g, " formula ")
    .replace(/-{3,}/g, " ")
    .replace(/\s+/g, " ")
    .trim();

  if (cleaned.length <= maxLength) {
    return cleaned;
  }

  return `${cleaned.slice(0, maxLength).trim()}...`;
}
