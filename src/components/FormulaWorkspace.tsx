import { useCallback, useEffect, useMemo, useRef, useState, type UIEvent } from "react";
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
import { MarkdownContent } from "./MarkdownContent";
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

const FORMULA_BATCH_SIZE = 24;

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
  const [visibleFormulaCount, setVisibleFormulaCount] = useState(FORMULA_BATCH_SIZE);
  const formulaListRef = useRef<HTMLDivElement | null>(null);

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

  const filteredFormulas = useMemo(
    () =>
      (formulaWorkspace?.formulas ?? [])
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
        ),
    [formulaWorkspace?.formulas, search, sortMode],
  );

  const loadMoreFormulas = useCallback(() => {
    setVisibleFormulaCount((current) => {
      if (current >= filteredFormulas.length) {
        return current;
      }

      return Math.min(current + FORMULA_BATCH_SIZE, filteredFormulas.length);
    });
  }, [filteredFormulas.length]);

  useEffect(() => {
    setVisibleFormulaCount(Math.min(FORMULA_BATCH_SIZE, filteredFormulas.length || FORMULA_BATCH_SIZE));

    if (formulaListRef.current) {
      formulaListRef.current.scrollTop = 0;
    }
  }, [filteredFormulas]);

  useEffect(() => {
    if (!selectedFormulaId) {
      return;
    }

    const selectedIndex = filteredFormulas.findIndex((formula) => formula.id === selectedFormulaId);
    if (selectedIndex === -1) {
      return;
    }

    setVisibleFormulaCount((current) => Math.max(current, Math.ceil((selectedIndex + 1) / FORMULA_BATCH_SIZE) * FORMULA_BATCH_SIZE));
  }, [filteredFormulas, selectedFormulaId]);

  useEffect(() => {
    if (visibleFormulaCount >= filteredFormulas.length) {
      return;
    }

    const frame = window.requestAnimationFrame(() => {
      const list = formulaListRef.current;
      if (!list) {
        return;
      }

      if (list.scrollHeight <= list.clientHeight + 24) {
        loadMoreFormulas();
      }
    });

    return () => window.cancelAnimationFrame(frame);
  }, [filteredFormulas.length, loadMoreFormulas, visibleFormulaCount]);

  const visibleFormulas = useMemo(
    () => filteredFormulas.slice(0, visibleFormulaCount),
    [filteredFormulas, visibleFormulaCount],
  );

  const handleFormulaListScroll = useCallback(
    (event: UIEvent<HTMLDivElement>) => {
      const list = event.currentTarget;
      const remainingDistance = list.scrollHeight - list.scrollTop - list.clientHeight;

      if (remainingDistance <= 240) {
        loadMoreFormulas();
      }
    },
    [loadMoreFormulas],
  );

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
            <>
              <div
                ref={formulaListRef}
                className="row-list row-list--compact formula-list"
                onScroll={handleFormulaListScroll}
              >
                {visibleFormulas.map((formula) => (
                  <FormulaListRow
                    key={formula.id}
                    formula={formula}
                    isActive={selectedFormulaId === formula.id}
                    onSelect={() => onSelectFormula(formula.id)}
                  />
                ))}
              </div>
            </>
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

              <section className="surface formula-context-grid">
                <div className="formula-context-column">
                  <div className="surface__header">
                    <div>
                      <span className="surface__eyebrow">Linked notes</span>
                      <h3>Where this formula lives</h3>
                    </div>
                  </div>
                  <div className="line-list formula-note-list">
                    {formulaDetails.linkedNotes.map((note) => (
                      <FormulaNoteRow key={note.noteId} note={note} onOpenNote={onOpenNote} />
                    ))}
                  </div>
                </div>
                <div className="formula-context-column">
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
  const headingChips = note.headings.slice(0, 2).map((heading) => compactLabel(heading, 32));
  const conceptChips = note.relatedConcepts.slice(0, 1).map((concept) => compactLabel(concept, 24));

  return (
    <button className="line-item formula-note-card" onClick={() => onOpenNote(note.noteId)} type="button">
      <span className="line-item__title">{note.title}</span>
      <span className="line-item__subtitle">{shortenPath(note.relativePath)}</span>
      <MarkdownContent className="formula-note-card__summary" text={formatMarkdownPreview(note.excerpt, 220)} />
      <div className="formula-chip-list">
        {headingChips.map((heading) => (
          <span key={`${note.noteId}-${heading}`} className="formula-chip">
            {heading}
          </span>
        ))}
        {conceptChips.map((concept) => (
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
  const headingLabel = compactLabel(formatHeadingPath(chunk.headingPath), 52);

  return (
    <article className="formula-chunk">
      <div className="surface__header">
        <div>
          <span className="surface__eyebrow">{headingLabel}</span>
          <h3>{chunk.noteTitle}</h3>
        </div>
        <button className="button button--ghost" onClick={() => onOpenNote(chunk.noteId)} type="button">
          Open note
        </button>
      </div>
      <MarkdownContent className="formula-chunk__body" text={formatMarkdownPreview(chunk.text, 340)} />
      <div className="formula-chip-list">
        <span className="formula-chip">{headingLabel}</span>
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
        <MarkdownContent className="inspector-copy" text={brief.coach.meaning} />
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
      <MarkdownContent className="inspector-copy" text={brief.derivation.intuition} />
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
          <li key={`${title}-${item}`}>
            <MarkdownContent text={item} />
          </li>
        ))}
      </ul>
    </div>
  );
}

function capitalize(value: string) {
  return value.slice(0, 1).toUpperCase() + value.slice(1);
}

function formatMarkdownPreview(value: string, maxLength: number) {
  const mathPattern = /\$\$[\s\S]*?\$\$|\\\[[\s\S]*?\\\]|\\\([^\n]+?\\\)|\$[^$\n]+\$/g;
  const mathTokens = value.match(mathPattern) ?? [];
  let mathIndex = 0;

  const normalized = value
    .replace(mathPattern, () => `__OBSIDIAN_OS_MATH_${mathIndex++}__`)
    .replace(/\[\[([^\]]+)\]\]/g, "$1")
    .replace(/^(#{1,6})\s+/gm, "")
    .replace(/\r/g, "")
    .replace(/\n{2,}/g, " ")
    .replace(/\n/g, " ")
    .replace(/\s+/g, " ")
    .trim();

  let output = "";
  let visibleLength = 0;
  let index = 0;

  while (index < normalized.length && visibleLength < maxLength) {
    const placeholderMatch = normalized.slice(index).match(/^__OBSIDIAN_OS_MATH_(\d+)__/);

    if (placeholderMatch) {
      const mathToken = mathTokens[Number(placeholderMatch[1])] ?? "";
      const normalizedMath = normalizeMathToken(mathToken);

      if (normalizedMath) {
        output = appendPreviewSegment(output, normalizedMath);
        visibleLength += getMathVisibleLength(normalizedMath);
      }

      index += placeholderMatch[0].length;
      continue;
    }

    output += normalized[index];
    visibleLength += 1;
    index += 1;
  }

  const trimmed = output.replace(/\s+/g, " ").trim();
  if (!trimmed) {
    return fallbackMathPreview(mathTokens, maxLength);
  }

  return index < normalized.length ? `${trimmed}...` : trimmed;
}

function normalizeMathToken(value: string) {
  if (!value) {
    return "";
  }

  if (value.startsWith("$$") && value.endsWith("$$")) {
    const body = value.slice(2, -2).replace(/\s+/g, " ").trim();
    return body ? `$$${body}$$` : "";
  }

  if (value.startsWith("\\[") && value.endsWith("\\]")) {
    const body = value.slice(2, -2).replace(/\s+/g, " ").trim();
    return body ? `$$${body}$$` : "";
  }

  if (value.startsWith("\\(") && value.endsWith("\\)")) {
    const body = value.slice(2, -2).replace(/\s+/g, " ").trim();
    return body ? `$${body}$` : "";
  }

  const body = value.slice(1, -1).replace(/\s+/g, " ").trim();
  return body ? `$${body}$` : "";
}

function getMathVisibleLength(value: string) {
  return value.replace(/^\$\$?|\$\$?$/g, "").trim().length;
}

function appendPreviewSegment(output: string, segment: string) {
  if (!output) {
    return segment;
  }

  if (output.endsWith(" ") || segment.startsWith(" ")) {
    return `${output}${segment}`;
  }

  return `${output} ${segment}`;
}

function fallbackMathPreview(mathTokens: string[], maxLength: number) {
  const fallback = mathTokens.map((token) => normalizeMathToken(token)).filter(Boolean).join(" ").trim();

  if (!fallback) {
    return "";
  }

  if (fallback.length <= maxLength) {
    return fallback;
  }

  return `${fallback.slice(0, maxLength).trim()}...`;
}

function formatHeadingPath(value: string) {
  const parts = value
    .split("/")
    .map((part) => part.trim())
    .filter(Boolean);

  if (parts.length <= 2) {
    return parts.join(" · ");
  }

  return `${parts[0]} · ${parts[parts.length - 1] ?? ""}`;
}

function compactLabel(value: string, maxLength: number) {
  if (value.length <= maxLength) {
    return value;
  }

  return `${value.slice(0, maxLength - 1).trim()}...`;
}
