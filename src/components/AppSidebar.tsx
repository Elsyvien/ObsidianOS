import { useState, type ReactNode } from "react";
import {
  ArrowDownToLine,
  BarChart2,
  BookOpen,
  BrainCircuit,
  ChartNoAxesCombined,
  ChevronDown,
  ClipboardCheck,
  FolderGit2,
  MessageSquareMore,
  Settings,
  Sigma,
  TerminalSquare,
} from "lucide-react";
import { formatDate } from "../lib";
import type { CourseConfig } from "../types";
import { APP_VIEWS, type AppView } from "./appShell";
import { BrandMark } from "./BrandMark";

const VIEW_ICONS: Record<AppView, React.ElementType> = {
  overview: BarChart2,
  statistics: ChartNoAxesCombined,
  notes: BookOpen,
  formulas: Sigma,
  ai: BrainCircuit,
  exams: ClipboardCheck,
  chat: MessageSquareMore,
  logs: TerminalSquare,
  outputs: ArrowDownToLine,
  courses: FolderGit2,
  settings: Settings,
};

/** Sub-groups inside the Study section. */
const NAV_GROUPS: Array<{ label: string; views: AppView[] }> = [
  { label: "Dashboard", views: ["overview", "statistics"] },
  { label: "Content",   views: ["notes", "formulas"] },
  { label: "AI & Study", views: ["ai", "chat", "exams"] },
  { label: "Generate",  views: ["outputs", "logs"] },
];

const CONFIG_VIEWS: AppView[] = ["courses", "settings"];

type AppSidebarProps = {
  activeView: AppView;
  connected: boolean;
  courses: CourseConfig[];
  logCount: number;
  runtimeMode: "tauri" | "browser-preview";
  selectedCourseId: string | null;
  vaultPath: string;
  onChangeView: (view: AppView) => void;
  onSelectCourse: (courseId: string) => void;
};

export function AppSidebar({
  activeView,
  connected,
  courses,
  logCount,
  runtimeMode,
  selectedCourseId,
  vaultPath,
  onChangeView,
  onSelectCourse,
}: AppSidebarProps) {
  // Auto-expand the group that contains the active view
  const activeGroupLabel = NAV_GROUPS.find((g) => g.views.includes(activeView))?.label ?? "";
  const selectedCourse = courses.find((course) => course.id === selectedCourseId) ?? null;
  const vaultPathSegments = vaultPath.split(/[\\/]/).filter(Boolean);
  const vaultName = vaultPathSegments[vaultPathSegments.length - 1] ?? vaultPath;
  const focusTitle = selectedCourse?.name ?? (runtimeMode === "browser-preview" ? "Preview workspace" : connected ? "Choose a course" : "Connect a vault");
  const focusSummary = selectedCourse
    ? `${Math.round(selectedCourse.coverage)}% coverage · ${selectedCourse.noteCount} notes · ${selectedCourse.weakNoteCount} weak`
    : runtimeMode === "browser-preview"
      ? "Demo data is loaded so the interface can be reviewed without a local vault."
      : connected
        ? `Vault ${vaultName || "connected"} is ready. Pick a course to open its study lanes.`
        : "Connect the Obsidian vault to index notes, formulas, and exam outputs.";

  return (
    <aside className="sidebar">
      <div className="sidebar__brand">
        <div className="sidebar__brand-lockup">
          <BrandMark className="brand-mark brand-mark--sidebar" />
          <div className="sidebar__brand-copy">
            <span className="sidebar__eyebrow">Study cockpit</span>
            <h1>ObsidianOS</h1>
          </div>
        </div>
      </div>

      <section className="sidebar__focus" aria-label="Workspace summary">
        <div className="sidebar__status-row">
          <span
            className={`sidebar__status-pill ${
              connected || runtimeMode === "browser-preview" ? "sidebar__status-pill--active" : ""
            }`}
          >
            {runtimeMode === "browser-preview" ? "Preview mode" : connected ? "Vault linked" : "Setup required"}
          </span>
          <span className="sidebar__status-pill sidebar__status-pill--muted">{courses.length} courses</span>
        </div>
        <div className="sidebar__focus-copy">
          <strong>{focusTitle}</strong>
          <p>{focusSummary}</p>
        </div>
        <div className="sidebar__focus-metrics">
          <div>
            <span>Exam</span>
            <strong>{selectedCourse?.examDate ? formatDate(selectedCourse.examDate) : "Unset"}</strong>
          </div>
          <div>
            <span>{selectedCourse ? "Notes" : "Courses"}</span>
            <strong>{selectedCourse ? String(selectedCourse.noteCount) : String(courses.length)}</strong>
          </div>
        </div>
      </section>

      {/* ── Study nav (grouped) ───────────────── */}
      <SidebarSection label="Study">
        {NAV_GROUPS.map((group) => (
          <NavGroup
            key={group.label}
            label={group.label}
            defaultOpen={group.label === activeGroupLabel}
          >
            {group.views.map((viewId) => {
              const view = APP_VIEWS.find((v) => v.id === viewId);
              if (!view) return null;
              const Icon = VIEW_ICONS[view.id];
              return (
                <button
                  key={view.id}
                  className={`nav-item ${activeView === view.id ? "nav-item--active" : ""}`}
                  onClick={() => onChangeView(view.id)}
                  type="button"
                >
                  <span className="nav-item__label">
                    <Icon size={15} strokeWidth={1.6} />
                    {view.label}
                  </span>
                  {view.id === "logs" ? <span className="nav-item__badge">{logCount}</span> : null}
                </button>
              );
            })}
          </NavGroup>
        ))}
      </SidebarSection>

      {/* ── Configure nav ──────────────────────── */}
      <SidebarSection label="Configure">
        {APP_VIEWS.filter((view) => CONFIG_VIEWS.includes(view.id)).map((view) => {
          const Icon = VIEW_ICONS[view.id];
          return (
            <button
              key={view.id}
              className={`nav-item ${activeView === view.id ? "nav-item--active" : ""}`}
              onClick={() => onChangeView(view.id)}
              type="button"
            >
              <span className="nav-item__label">
                <Icon size={15} strokeWidth={1.6} />
                {view.label}
              </span>
            </button>
          );
        })}
      </SidebarSection>

      {/* ── Course library ─────────────────────── */}
      <SidebarSection
        label="Course library"
        action={
          <button className="sidebar__link" onClick={() => onChangeView("courses")} type="button">
            Manage
          </button>
        }
      >
        {courses.length === 0 ? (
          <div className="sidebar__empty">
            <strong>No courses saved</strong>
            <p>Connect the vault, then review or edit the imported course folders.</p>
          </div>
        ) : (
          <div className="course-nav">
            {courses.map((course) => {
              const coverageWidth = `${Math.max(6, Math.min(100, Math.round(course.coverage)))}%`;
              return (
                <button
                  key={course.id}
                  className={`course-nav__item ${selectedCourseId === course.id ? "course-nav__item--active" : ""}`}
                  onClick={() => onSelectCourse(course.id)}
                  type="button"
                >
                  <div className="course-nav__title-row">
                    <span className="course-nav__name">{course.name}</span>
                    <span className="course-nav__coverage">{Math.round(course.coverage)}%</span>
                  </div>
                  <span className="course-nav__meta">
                    {course.examDate ? formatDate(course.examDate) : "No exam date"}
                  </span>
                  <span className="course-nav__meta">
                    {course.noteCount} notes, {course.weakNoteCount} weak
                  </span>
                  <span className="course-nav__bar" aria-hidden="true">
                    <span style={{ width: coverageWidth }} />
                  </span>
                </button>
              );
            })}
          </div>
        )}
      </SidebarSection>
    </aside>
  );
}

/* ── Shared components ────────────────────────────── */

/** Top-level sidebar section with a label header. */
function SidebarSection({
  action,
  children,
  label,
}: {
  action?: ReactNode;
  children: ReactNode;
  label: string;
}) {
  return (
    <section className="sidebar__group">
      <div className="sidebar__group-header">
        <span className="sidebar__group-label">{label}</span>
        {action}
      </div>
      <div className="sidebar__group-body">{children}</div>
    </section>
  );
}

/** Collapsible sub-group within a sidebar section. */
function NavGroup({
  children,
  defaultOpen = false,
  label,
}: {
  children: ReactNode;
  defaultOpen?: boolean;
  label: string;
}) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <div className={`nav-group ${open ? "nav-group--open" : ""}`}>
      <button
        className="nav-group__toggle"
        onClick={() => setOpen((o) => !o)}
        type="button"
      >
        <ChevronDown size={12} strokeWidth={2} className="nav-group__chevron" />
        <span>{label}</span>
      </button>
      {open && <div className="nav-group__items">{children}</div>}
    </div>
  );
}
