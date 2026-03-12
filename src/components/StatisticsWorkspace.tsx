import { useEffect, useState, type ReactNode } from "react";
import {
  Area,
  AreaChart,
  Bar,
  BarChart,
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { clampPercent, formatDate, formatDateTime } from "../lib";
import type {
  CourseConfig,
  GitCommitItem,
  GitNoteActivityRow,
  StatisticsCountBucket,
  StatisticsExamPoint,
  StatisticsNoteRow,
  StatisticsResponse,
  StatisticsScope,
  StatisticsValuePoint,
} from "../types";
import { looksLikeLatex, MathFormula } from "./MathFormula";

type StatisticsSection = "overview" | "knowledge" | "notes" | "exams" | "ai" | "outputs" | "vault-activity" | "git";

type StatisticsWorkspaceProps = {
  runtimeMode: "tauri" | "browser-preview";
  scope: StatisticsScope;
  statistics: StatisticsResponse | null;
  selectedCourse: CourseConfig | null;
  onChangeScope: (scope: StatisticsScope) => void;
};

const CHART_COLORS = {
  accent: "#79bbff",
  mint: "#50d7a5",
  amber: "#ffb870",
  coral: "#ff7f72",
  violet: "#c0a8ff",
  grid: "rgba(255, 255, 255, 0.08)",
  text: "rgba(245, 247, 250, 0.78)",
};

const BASE_SECTIONS: Array<{ id: StatisticsSection; label: string }> = [
  { id: "overview", label: "Overview" },
  { id: "knowledge", label: "Knowledge" },
  { id: "notes", label: "Notes" },
  { id: "exams", label: "Exams" },
  { id: "ai", label: "AI" },
  { id: "outputs", label: "Outputs" },
  { id: "vault-activity", label: "Vault Activity" },
];

export function StatisticsWorkspace({
  runtimeMode,
  scope,
  statistics,
  selectedCourse,
  onChangeScope,
}: StatisticsWorkspaceProps) {
  const isPreview = runtimeMode === "browser-preview";
  const sections = statistics?.gitAvailable ? [...BASE_SECTIONS, { id: "git" as const, label: "Git" }] : BASE_SECTIONS;
  const [activeSection, setActiveSection] = useState<StatisticsSection>("overview");

  useEffect(() => {
    if (activeSection === "git" && !statistics?.gitAvailable) {
      setActiveSection("overview");
    }
  }, [activeSection, statistics?.gitAvailable]);

  if (scope === "course" && !selectedCourse && !isPreview) {
    return (
      <div className="page-stack">
        <section className="surface surface--hero">
          <span className="surface__eyebrow">Statistics</span>
          <h3>Select a course first</h3>
          <p>Choose a course from the sidebar to unlock course-level trends, ranked note lists, and Git-aware edit history.</p>
        </section>
      </div>
    );
  }

  if (!statistics) {
    return (
      <div className="page-stack">
        <section className="surface surface--hero">
          <span className="surface__eyebrow">Statistics</span>
          <h3>No statistics available yet</h3>
          <p>Run a scan to create scan snapshots. Once a vault exists, this page will layer current state, scan history, and Git history when available.</p>
        </section>
      </div>
    );
  }

  const history = statistics.overview.history.map((point) => ({ ...point, label: formatDate(point.capturedAt) }));
  const examHistory = statistics.exams.scoreHistory.map((point) => ({ ...point, label: formatDate(point.submittedAt) }));
  const attemptHistory = statistics.exams.attemptHistory;
  const gitCommitTimeline = statistics.git?.commitTimeline ?? [];
  const gitChurnTimeline = statistics.git?.churnTimeline ?? [];

  return (
    <div className="page-stack">
      <section className="surface surface--hero stats-hero">
        <div className="surface__header">
          <div>
            <span className="surface__eyebrow">Statistics</span>
            <h3>{scope === "vault" ? "Vault analytics" : `${statistics.courseName ?? selectedCourse?.name ?? "Course"} analytics`}</h3>
          </div>
          <div className="toolbar">
            <button className={`toolbar__item ${scope === "course" ? "toolbar__item--active" : ""}`} onClick={() => onChangeScope("course")} type="button">Course</button>
            <button className={`toolbar__item ${scope === "vault" ? "toolbar__item--active" : ""}`} onClick={() => onChangeScope("vault")} type="button">Vault</button>
          </div>
        </div>
        <p className="surface__summary">
          {scope === "vault"
            ? "Learning-state evolution comes from scan snapshots. Editing evolution comes from markdown metadata and Git history when the vault is versioned."
            : "This page blends scan history, current note quality, exam performance, outputs, and optional Git activity for the selected course."}
        </p>
        <div className="metric-strip">
          <MetricCard label="Notes" value={String(statistics.overview.summary.noteCount)} />
          <MetricCard label="Coverage" value={`${clampPercent(statistics.overview.summary.coveragePercentage)}%`} />
          <MetricCard label="Avg note strength" value={statistics.overview.summary.averageNoteStrength.toFixed(1)} />
          <MetricCard label="Links" value={String(statistics.overview.summary.edgeCount)} />
          <MetricCard label="Weak notes" value={String(statistics.overview.summary.weakNoteCount)} />
          <MetricCard label="Exam average" value={statistics.exams.summary.averageScore === null ? "--" : `${clampPercent(statistics.exams.summary.averageScore)}%`} />
        </div>
        <div className="stats-header-row">
          <div className="stats-section-tabs">
            {sections.map((section) => (
              <button
                key={section.id}
                className={`button button--ghost stats-section-tab ${activeSection === section.id ? "stats-section-tab--active" : ""}`}
                onClick={() => setActiveSection(section.id)}
                type="button"
              >
                {section.label}
              </button>
            ))}
          </div>
          <div className="stats-header-meta">
            <span className={`soft-badge ${statistics.gitAvailable ? "soft-badge--success" : "soft-badge--neutral"}`}>
              {statistics.gitAvailable ? "Git-backed analytics" : "Scan + file metadata only"}
            </span>
            <span className="meta-pill">Updated {formatDateTime(statistics.generatedAt)}</span>
          </div>
        </div>
        {statistics.gitError ? <p className="stats-inline-note">Git analytics unavailable: {statistics.gitError}</p> : null}
      </section>

      {activeSection === "overview" ? (
        <>
          <section className="surface surface--split">
            <ChartSurface eyebrow="Coverage" title="Coverage trend" hasData={history.length > 0} emptyMessage="No scan history yet.">
              <ResponsiveContainer width="100%" height={260}>
                <LineChart data={history}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="label" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Line type="monotone" dataKey="coveragePercentage" stroke={CHART_COLORS.accent} strokeWidth={2.5} dot={false} />
                </LineChart>
              </ResponsiveContainer>
            </ChartSurface>
            <ChartSurface eyebrow="Growth" title="Indexed notes" hasData={history.length > 0} emptyMessage="No note-growth history yet.">
              <ResponsiveContainer width="100%" height={260}>
                <AreaChart data={history}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="label" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Area type="monotone" dataKey="noteCount" stroke={CHART_COLORS.mint} fill="rgba(80, 215, 165, 0.18)" strokeWidth={2.5} />
                </AreaChart>
              </ResponsiveContainer>
            </ChartSurface>
          </section>
          <section className="surface surface--split">
            <ChartSurface eyebrow="Graph" title="Graph growth" hasData={history.length > 0} emptyMessage="No graph history yet.">
              <ResponsiveContainer width="100%" height={280}>
                <LineChart data={history}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="label" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Line type="monotone" dataKey="edgeCount" stroke={CHART_COLORS.accent} strokeWidth={2.4} dot={false} />
                  <Line type="monotone" dataKey="strongLinks" stroke={CHART_COLORS.mint} strokeWidth={2.1} dot={false} />
                </LineChart>
              </ResponsiveContainer>
            </ChartSurface>
            <ChartSurface eyebrow="Weakness" title="Weak-note trend" hasData={history.length > 0} emptyMessage="No weak-note history yet.">
              <ResponsiveContainer width="100%" height={280}>
                <LineChart data={history}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="label" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Line type="monotone" dataKey="weakNoteCount" stroke={CHART_COLORS.coral} strokeWidth={2.4} dot={false} />
                </LineChart>
              </ResponsiveContainer>
            </ChartSurface>
          </section>
          <section className="surface surface--split">
            <ChartSurface eyebrow="Editing" title="Git commit activity" hasData={gitCommitTimeline.length > 0} emptyMessage="No Git timeline for this scope.">
              <ResponsiveContainer width="100%" height={260}>
                <BarChart data={gitCommitTimeline}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="bucket" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Bar dataKey="commitCount" fill={CHART_COLORS.amber} radius={[8, 8, 0, 0]} />
                </BarChart>
              </ResponsiveContainer>
            </ChartSurface>
            <ListSurface eyebrow="Highlights" title="Ranked signals">
              {statistics.overview.highlights.length > 0 ? (
                <div className="stats-highlight-grid">
                  {statistics.overview.highlights.map((highlight) => (
                    <div key={highlight.label} className="stats-highlight-card">
                      <span>{highlight.label}</span>
                      <strong>{highlight.value}</strong>
                    </div>
                  ))}
                </div>
              ) : (
                <EmptyState title="No ranked highlights yet" description="More scans and more courses will make the overview comparisons richer." />
              )}
            </ListSurface>
          </section>
          {scope === "vault" ? (
            <section className="surface">
              <ChartSurface eyebrow="Comparison" title="Coverage by course" hasData={statistics.overview.courseRows.length > 0} emptyMessage="No course comparison available.">
                <ResponsiveContainer width="100%" height={280}>
                  <BarChart data={statistics.overview.courseRows}>
                    <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                    <XAxis dataKey="courseName" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                    <YAxis domain={[0, 100]} stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                    <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                    <Bar dataKey="coveragePercentage" fill={CHART_COLORS.accent} radius={[8, 8, 0, 0]} />
                  </BarChart>
                </ResponsiveContainer>
              </ChartSurface>
            </section>
          ) : null}
        </>
      ) : null}

      {activeSection === "knowledge" ? (
        <>
          <section className="surface surface--split">
            <ChartSurface eyebrow="Concepts" title="Concept coverage over time" hasData={history.length > 0} emptyMessage="No knowledge history yet.">
              <ResponsiveContainer width="100%" height={260}>
                <LineChart data={history}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="label" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Line type="monotone" dataKey="coveredConcepts" stroke={CHART_COLORS.accent} strokeWidth={2.4} dot={false} />
                  <Line type="monotone" dataKey="totalConcepts" stroke={CHART_COLORS.violet} strokeWidth={2.1} dot={false} />
                </LineChart>
              </ResponsiveContainer>
            </ChartSurface>
            <ChartSurface eyebrow="Formulas" title="Formula count over time" hasData={history.length > 0} emptyMessage="No formula history yet.">
              <ResponsiveContainer width="100%" height={260}>
                <AreaChart data={history}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="label" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Area type="monotone" dataKey="formulaCount" stroke={CHART_COLORS.violet} fill="rgba(192, 168, 255, 0.15)" strokeWidth={2.4} />
                </AreaChart>
              </ResponsiveContainer>
            </ChartSurface>
          </section>
          <section className="surface surface--split">
            <ChartSurface eyebrow="Density" title="Formula density" hasData={statistics.knowledge.formulaDensityBuckets.length > 0} emptyMessage="No formula density yet.">
              <BucketBar data={statistics.knowledge.formulaDensityBuckets} color={CHART_COLORS.amber} />
            </ChartSurface>
            <ChartSurface eyebrow="Concepts" title="Top concepts" hasData={statistics.knowledge.topConcepts.length > 0} emptyMessage="No concepts yet.">
              <ResponsiveContainer width="100%" height={280}>
                <BarChart data={statistics.knowledge.topConcepts}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="name" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Bar dataKey="supportScore" fill={CHART_COLORS.accent} radius={[8, 8, 0, 0]} />
                </BarChart>
              </ResponsiveContainer>
            </ChartSurface>
          </section>
          <section className="surface surface--split">
            <ListSurface eyebrow="Concepts" title="Highest-support concepts">
              <MiniMetricList items={statistics.knowledge.topConcepts.map((item) => ({ label: item.name, value: `${item.supportScore}` }))} />
            </ListSurface>
            <ListSurface eyebrow="Formulas" title="Most visible formulas">
              <MiniMetricList items={statistics.knowledge.topFormulas.map((item) => ({ label: item.latex, value: `${item.noteCount} notes` }))} />
            </ListSurface>
          </section>
        </>
      ) : null}

      {activeSection === "notes" ? (
        <>
          <section className="surface surface--split">
            <ChartSurface eyebrow="Strength" title="Strength distribution" hasData={statistics.notes.strengthBuckets.length > 0} emptyMessage="No note strength data yet.">
              <BucketBar data={statistics.notes.strengthBuckets} color={CHART_COLORS.mint} />
            </ChartSurface>
            <ChartSurface eyebrow="Aging" title="Markdown age buckets" hasData={statistics.notes.activityBuckets.length > 0} emptyMessage="No markdown metadata yet.">
              <ResponsiveContainer width="100%" height={260}>
                <BarChart data={statistics.notes.activityBuckets}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="label" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Bar dataKey="noteCount" fill={CHART_COLORS.amber} radius={[8, 8, 0, 0]} />
                </BarChart>
              </ResponsiveContainer>
            </ChartSurface>
          </section>
          <section className="surface surface--split">
            <ChartSurface eyebrow="Weakness" title="Weak vs isolated notes" hasData={history.length > 0} emptyMessage="No scan history yet.">
              <ResponsiveContainer width="100%" height={280}>
                <LineChart data={history}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="label" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Line type="monotone" dataKey="weakNoteCount" stroke={CHART_COLORS.coral} strokeWidth={2.2} dot={false} />
                  <Line type="monotone" dataKey="isolatedNotes" stroke={CHART_COLORS.violet} strokeWidth={2.2} dot={false} />
                </LineChart>
              </ResponsiveContainer>
            </ChartSurface>
            <ListSurface eyebrow="Churn" title="Most changed notes">
              <NoteRowsList rows={statistics.notes.mostChangedNotes} emptyMessage="No Git note churn yet." />
            </ListSurface>
          </section>
          <section className="surface surface--triptych">
            <ListSurface eyebrow="Weakest" title="Lowest-strength notes">
              <NoteRowsList rows={statistics.notes.weakestNotes} emptyMessage="No note ranking yet." />
            </ListSurface>
            <ListSurface eyebrow="Connected" title="Most connected notes">
              <NoteRowsList rows={statistics.notes.mostConnectedNotes} emptyMessage="No connection ranking yet." />
            </ListSurface>
            <ListSurface eyebrow="Stale" title="Stalest notes">
              <NoteRowsList rows={statistics.notes.stalestNotes} emptyMessage="No stale notes yet." />
            </ListSurface>
          </section>
        </>
      ) : null}

      {activeSection === "exams" ? (
        <>
          <section className="surface surface--split">
            <ChartSurface eyebrow="Scores" title="Exam scores over time" hasData={examHistory.length > 0} emptyMessage="No exam attempts yet.">
              <ResponsiveContainer width="100%" height={260}>
                <LineChart data={examHistory}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="label" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis domain={[0, 100]} stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Line type="monotone" dataKey="scorePercent" stroke={CHART_COLORS.coral} strokeWidth={2.6} dot={{ r: 3 }} />
                </LineChart>
              </ResponsiveContainer>
            </ChartSurface>
            <ChartSurface eyebrow="Volume" title="Attempt cadence" hasData={attemptHistory.length > 0} emptyMessage="No attempt history yet.">
              <SimpleValueBars data={attemptHistory} color={CHART_COLORS.accent} />
            </ChartSurface>
          </section>
          <section className="surface surface--split">
            <ChartSurface eyebrow="Verdicts" title="Verdict mix" hasData={statistics.exams.verdictMix.length > 0} emptyMessage="No verdict data yet.">
              <BucketBar data={statistics.exams.verdictMix} color={CHART_COLORS.amber} dataKey="count" />
            </ChartSurface>
            <ChartSurface eyebrow="Mastery" title="Review vs mastered" hasData={statistics.exams.masteryDistribution.length > 0} emptyMessage="No mastery distribution yet.">
              <BucketBar data={statistics.exams.masteryDistribution} color={CHART_COLORS.mint} dataKey="count" />
            </ChartSurface>
          </section>
          <section className="surface surface--split">
            <ListSurface eyebrow="Recent" title="Recent exams">
              <ExamRowsList rows={statistics.exams.recentExams} emptyMessage="No recent exams yet." />
            </ListSurface>
            <ListSurface eyebrow="Weakest" title="Weakest attempts">
              <ExamRowsList rows={statistics.exams.weakestAttempts} emptyMessage="No weak attempts yet." />
            </ListSurface>
          </section>
        </>
      ) : null}

      {activeSection === "ai" ? (
        <>
          <section className="surface surface--split">
            <ChartSurface eyebrow="Status" title="Current AI breakdown" hasData={statistics.ai.statusBreakdown.length > 0} emptyMessage="No AI data yet.">
              <BucketBar data={statistics.ai.statusBreakdown} color={CHART_COLORS.violet} dataKey="count" />
            </ChartSurface>
            <ChartSurface eyebrow="History" title="AI readiness over scan history" hasData={history.length > 0} emptyMessage="No AI scan history yet.">
              <ResponsiveContainer width="100%" height={280}>
                <LineChart data={history}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="label" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Line type="monotone" dataKey="aiReadyNotes" stroke={CHART_COLORS.mint} strokeWidth={2.4} dot={false} />
                  <Line type="monotone" dataKey="aiStaleNotes" stroke={CHART_COLORS.amber} strokeWidth={2.1} dot={false} />
                  <Line type="monotone" dataKey="aiFailedNotes" stroke={CHART_COLORS.coral} strokeWidth={2.1} dot={false} />
                </LineChart>
              </ResponsiveContainer>
            </ChartSurface>
          </section>
          <section className="surface surface--split">
            <ListSurface eyebrow="Failed" title="Failed notes">
              <NoteRowsList rows={statistics.ai.failedNotes} emptyMessage="No failed notes." />
            </ListSurface>
            <ListSurface eyebrow="Stale" title="Stale or missing notes">
              <NoteRowsList rows={statistics.ai.staleNotes} emptyMessage="No stale notes." />
            </ListSurface>
          </section>
          {scope === "vault" ? (
            <section className="surface">
              <ChartSurface eyebrow="Courses" title="Ready notes by course" hasData={statistics.ai.courseRows.length > 0} emptyMessage="No course AI breakdown yet.">
                <ResponsiveContainer width="100%" height={280}>
                  <BarChart data={statistics.ai.courseRows}>
                    <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                    <XAxis dataKey="courseName" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                    <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                    <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                    <Bar dataKey="aiReadyNotes" fill={CHART_COLORS.mint} radius={[8, 8, 0, 0]} />
                  </BarChart>
                </ResponsiveContainer>
              </ChartSurface>
            </section>
          ) : null}
        </>
      ) : null}

      {activeSection === "outputs" ? (
        <>
          <section className="surface surface--split">
            <ChartSurface eyebrow="Flashcards" title="Cards over time" hasData={history.length > 0} emptyMessage="No output history yet.">
              <ResponsiveContainer width="100%" height={260}>
                <LineChart data={history}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="label" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Line type="monotone" dataKey="flashcardTotalCards" stroke={CHART_COLORS.accent} strokeWidth={2.4} dot={false} />
                  <Line type="monotone" dataKey="flashcardSetCount" stroke={CHART_COLORS.violet} strokeWidth={2.1} dot={false} />
                </LineChart>
              </ResponsiveContainer>
            </ChartSurface>
            <ChartSurface eyebrow="Revision" title="Revision runs over time" hasData={history.length > 0} emptyMessage="No revision history yet.">
              <ResponsiveContainer width="100%" height={260}>
                <LineChart data={history}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="label" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Line type="monotone" dataKey="revisionRunCount" stroke={CHART_COLORS.mint} strokeWidth={2.4} dot={false} />
                </LineChart>
              </ResponsiveContainer>
            </ChartSurface>
          </section>
          <section className="surface surface--split">
            <ChartSurface eyebrow="Mix" title="Current output mix" hasData={statistics.outputs.outputMix.length > 0} emptyMessage="No outputs yet.">
              <BucketBar data={statistics.outputs.outputMix} color={CHART_COLORS.amber} dataKey="count" />
            </ChartSurface>
            <ListSurface eyebrow="Latest" title="Latest exports">
              <dl className="definition-grid">
                <Definition label="Flashcard export" value={statistics.outputs.summary.latestFlashcardExport ?? "Not generated"} />
                <Definition label="Revision note" value={statistics.outputs.summary.latestRevisionNote ?? "Not generated"} />
                <Definition label="Cards" value={String(statistics.outputs.summary.flashcardTotalCards)} />
                <Definition label="Latest revision items" value={String(statistics.outputs.summary.latestRevisionItemCount)} />
              </dl>
            </ListSurface>
          </section>
          {scope === "vault" ? (
            <section className="surface">
              <ChartSurface eyebrow="Courses" title="Output volume by course" hasData={statistics.outputs.courseRows.length > 0} emptyMessage="No output comparison yet.">
                <ResponsiveContainer width="100%" height={280}>
                  <BarChart data={statistics.outputs.courseRows}>
                    <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                    <XAxis dataKey="courseName" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                    <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                    <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                    <Bar dataKey="flashcardTotalCards" fill={CHART_COLORS.accent} radius={[8, 8, 0, 0]} />
                  </BarChart>
                </ResponsiveContainer>
              </ChartSurface>
            </section>
          ) : null}
        </>
      ) : null}

      {activeSection === "vault-activity" ? (
        <>
          <section className="surface surface--split">
            <ChartSurface eyebrow="Files" title="Markdown age buckets" hasData={statistics.vaultActivity.activityBuckets.length > 0} emptyMessage="No file metadata yet.">
              <ResponsiveContainer width="100%" height={260}>
                <BarChart data={statistics.vaultActivity.activityBuckets}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="label" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Bar dataKey="noteCount" fill={CHART_COLORS.amber} radius={[8, 8, 0, 0]} />
                </BarChart>
              </ResponsiveContainer>
            </ChartSurface>
            <ChartSurface eyebrow="Git" title="Markdown edits over time" hasData={statistics.vaultActivity.gitTimeline.length > 0} emptyMessage="No Git timeline for this vault.">
              <ResponsiveContainer width="100%" height={260}>
                <BarChart data={statistics.vaultActivity.gitTimeline}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="bucket" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Bar dataKey="changedNotes" fill={CHART_COLORS.accent} radius={[8, 8, 0, 0]} />
                </BarChart>
              </ResponsiveContainer>
            </ChartSurface>
          </section>
          <section className="surface surface--split">
            <ListSurface eyebrow="Recent" title="Recently modified notes">
              <NoteRowsList rows={statistics.vaultActivity.recentNotes} emptyMessage="No recent note activity yet." />
            </ListSurface>
            <ListSurface eyebrow="Commits" title="Recent markdown commits">
              <CommitList rows={statistics.vaultActivity.recentCommits} emptyMessage="No recent markdown commits." />
            </ListSurface>
          </section>
          {scope === "vault" ? (
            <section className="surface surface--split">
              <ChartSurface eyebrow="Courses" title="Per-course activity" hasData={statistics.vaultActivity.courseActivity.length > 0} emptyMessage="No course activity yet.">
                <ResponsiveContainer width="100%" height={280}>
                  <BarChart data={statistics.vaultActivity.courseActivity}>
                    <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                    <XAxis dataKey="courseName" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                    <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                    <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                    <Bar dataKey="noteCount" fill={CHART_COLORS.mint} radius={[8, 8, 0, 0]} />
                  </BarChart>
                </ResponsiveContainer>
              </ChartSurface>
              <ListSurface eyebrow="Edited" title="Top edited notes">
                <NoteRowsList rows={statistics.vaultActivity.gitTopNotes} emptyMessage="No Git note activity yet." />
              </ListSurface>
            </section>
          ) : null}
        </>
      ) : null}

      {activeSection === "git" && statistics.git ? (
        <>
          <section className="surface surface--split">
            <ChartSurface eyebrow="Commits" title="Commit activity timeline" hasData={statistics.git.commitTimeline.length > 0} emptyMessage="No commit timeline yet.">
              <ResponsiveContainer width="100%" height={260}>
                <BarChart data={statistics.git.commitTimeline}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="bucket" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Bar dataKey="commitCount" fill={CHART_COLORS.accent} radius={[8, 8, 0, 0]} />
                </BarChart>
              </ResponsiveContainer>
            </ChartSurface>
            <ChartSurface eyebrow="Churn" title="Markdown churn timeline" hasData={gitChurnTimeline.length > 0} emptyMessage="No churn timeline yet.">
              <ResponsiveContainer width="100%" height={260}>
                <LineChart data={gitChurnTimeline}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="bucket" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Line type="monotone" dataKey="changedNotes" stroke={CHART_COLORS.amber} strokeWidth={2.4} dot={false} />
                </LineChart>
              </ResponsiveContainer>
            </ChartSurface>
          </section>
          <section className="surface surface--split">
            <ChartSurface eyebrow="Courses" title="Git course activity" hasData={statistics.git.courseActivity.length > 0} emptyMessage="No course activity from Git yet.">
              <ResponsiveContainer width="100%" height={280}>
                <BarChart data={statistics.git.courseActivity}>
                  <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
                  <XAxis dataKey="courseName" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
                  <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
                  <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
                  <Bar dataKey="commitCount" fill={CHART_COLORS.violet} radius={[8, 8, 0, 0]} />
                </BarChart>
              </ResponsiveContainer>
            </ChartSurface>
            <ListSurface eyebrow="Notes" title="Top changed notes">
              <NoteRowsList rows={statistics.git.topNotes} emptyMessage="No changed-note data yet." />
            </ListSurface>
          </section>
          <section className="surface">
            <ListSurface eyebrow="Commits" title="Recent commit feed">
              <CommitList rows={statistics.git.recentCommits} emptyMessage="No recent commits yet." />
            </ListSurface>
          </section>
        </>
      ) : null}
    </div>
  );
}

function ChartSurface({
  eyebrow,
  title,
  children,
  emptyMessage,
  hasData,
}: {
  eyebrow: string;
  title: string;
  children: ReactNode;
  emptyMessage: string;
  hasData: boolean;
}) {
  return (
    <div>
      <div className="surface__header">
        <div>
          <span className="surface__eyebrow">{eyebrow}</span>
          <h3>{title}</h3>
        </div>
      </div>
      {hasData ? <div className="stats-chart-frame">{children}</div> : <EmptyState title="Nothing to chart yet" description={emptyMessage} />}
    </div>
  );
}

function ListSurface({ eyebrow, title, children }: { eyebrow: string; title: string; children: ReactNode }) {
  return (
    <div>
      <div className="surface__header">
        <div>
          <span className="surface__eyebrow">{eyebrow}</span>
          <h3>{title}</h3>
        </div>
      </div>
      {children}
    </div>
  );
}

function MetricCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="metric">
      <span>{label}</span>
      <strong>{value}</strong>
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

function EmptyState({ title, description }: { title: string; description: string }) {
  return (
    <div className="empty-pane">
      <strong>{title}</strong>
      <p>{description}</p>
    </div>
  );
}

function BucketBar({ data, color, dataKey = "count" }: { data: StatisticsCountBucket[]; color: string; dataKey?: "count" | "noteCount" }) {
  return (
    <ResponsiveContainer width="100%" height={260}>
      <BarChart data={data}>
        <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
        <XAxis dataKey="label" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
        <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
        <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
        <Bar dataKey={dataKey} fill={color} radius={[8, 8, 0, 0]} />
      </BarChart>
    </ResponsiveContainer>
  );
}

function SimpleValueBars({ data, color }: { data: StatisticsValuePoint[]; color: string }) {
  return (
    <ResponsiveContainer width="100%" height={260}>
      <BarChart data={data}>
        <CartesianGrid stroke={CHART_COLORS.grid} vertical={false} />
        <XAxis dataKey="label" stroke={CHART_COLORS.text} tickLine={false} axisLine={false} />
        <YAxis stroke={CHART_COLORS.text} tickLine={false} axisLine={false} width={42} />
        <Tooltip contentStyle={tooltipStyle} labelStyle={tooltipLabelStyle} />
        <Bar dataKey="value" fill={color} radius={[8, 8, 0, 0]} />
      </BarChart>
    </ResponsiveContainer>
  );
}

function MiniMetricList({ items }: { items: Array<{ label: string; value: string }> }) {
  if (items.length === 0) {
    return <EmptyState title="Nothing listed yet" description="More scans will make this section denser." />;
  }

  return (
    <div className="stats-mini-list">
      {items.map((item) => (
        <div key={`${item.label}-${item.value}`} className="stats-mini-row">
          {looksLikeLatex(item.label) ? (
            <MathFormula className="stats-mini-row__math" display={false} latex={item.label} showSource={false} />
          ) : (
            <span>{item.label}</span>
          )}
          <strong>{item.value}</strong>
        </div>
      ))}
    </div>
  );
}

function NoteRowsList({ rows, emptyMessage }: { rows: Array<StatisticsNoteRow | GitNoteActivityRow>; emptyMessage: string }) {
  if (rows.length === 0) {
    return <EmptyState title="Nothing listed yet" description={emptyMessage} />;
  }

  return (
    <div className="stats-list">
      {rows.map((row) => (
        <div key={`${row.relativePath}-${row.noteId ?? row.relativePath}`} className="stats-list-row">
          <div>
            <strong>{row.title}</strong>
            <span>{row.relativePath}</span>
          </div>
          <div className="stats-list-meta">
            {"changeCount" in row ? <span>{row.changeCount} edits</span> : null}
            {"strength" in row ? <span>strength {row.strength.toFixed(1)}</span> : null}
            {"linkCount" in row ? <span>{row.linkCount} links</span> : null}
            {"modifiedAt" in row && row.modifiedAt ? <span>{formatDate(row.modifiedAt)}</span> : null}
            {"lastCommitAt" in row && row.lastCommitAt ? <span>{formatDate(row.lastCommitAt)}</span> : null}
          </div>
        </div>
      ))}
    </div>
  );
}

function ExamRowsList({ rows, emptyMessage }: { rows: StatisticsExamPoint[]; emptyMessage: string }) {
  if (rows.length === 0) {
    return <EmptyState title="Nothing listed yet" description={emptyMessage} />;
  }

  return (
    <div className="stats-list">
      {rows.map((row) => (
        <div key={`${row.examId}-${row.submittedAt}`} className="stats-list-row">
          <div>
            <strong>{row.examTitle}</strong>
            <span>{row.courseName ?? "Vault"} · {formatDate(row.submittedAt)}</span>
          </div>
          <div className="stats-list-meta">
            <strong>{clampPercent(row.scorePercent)}%</strong>
          </div>
        </div>
      ))}
    </div>
  );
}

function CommitList({ rows, emptyMessage }: { rows: GitCommitItem[]; emptyMessage: string }) {
  if (rows.length === 0) {
    return <EmptyState title="Nothing listed yet" description={emptyMessage} />;
  }

  return (
    <div className="stats-list">
      {rows.map((row) => (
        <div key={`${row.sha}-${row.committedAt}`} className="stats-list-row">
          <div>
            <strong>{row.summary}</strong>
            <span>{row.authorName} · {formatDate(row.committedAt)}</span>
          </div>
          <div className="stats-list-meta">
            <span>{row.changedNotes} notes</span>
            <code>{row.sha.slice(0, 7)}</code>
          </div>
        </div>
      ))}
    </div>
  );
}

const tooltipStyle = {
  backgroundColor: "#111821",
  border: "1px solid rgba(255, 255, 255, 0.08)",
  borderRadius: 14,
  color: "#f5f7fa",
};

const tooltipLabelStyle = {
  color: "#f5f7fa",
  fontWeight: 600,
};
