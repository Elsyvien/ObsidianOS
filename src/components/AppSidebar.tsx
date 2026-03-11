import type { ReactNode } from "react";
import { ArrowDownToLine, BarChart2, BookOpen, BrainCircuit, FolderGit2, Settings, TerminalSquare } from "lucide-react";
import { formatDate, shortenPath } from "../lib";
import type { CourseConfig } from "../types";
import { APP_VIEWS, type AppView } from "./appShell";
import { BrandMark } from "./BrandMark";

const VIEW_ICONS: Record<AppView, React.ElementType> = {
  overview: BarChart2,
  ai: BrainCircuit,
  logs: TerminalSquare,
  notes: BookOpen,
  outputs: ArrowDownToLine,
  courses: FolderGit2,
  settings: Settings,
};

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

const STUDY_VIEWS: AppView[] = ["overview", "ai", "logs", "notes", "outputs"];
const CONFIG_VIEWS: AppView[] = ["courses", "settings"];

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
  const isPreview = runtimeMode === "browser-preview";

  return (
    <aside className="sidebar">
      <div className="sidebar__brand">
        <div className="sidebar__brand-lockup">
          <BrandMark className="brand-mark brand-mark--sidebar" />
          <div>
            <span className="sidebar__eyebrow">ObsidianOS</span>
            <h1>Exam workspace</h1>
          </div>
        </div>
        <p>One place to scan the vault, review notes, and prepare for the exam.</p>
      </div>

      <div className="workspace-chip">
        <span className={`workspace-chip__dot ${connected ? "workspace-chip__dot--active" : ""}`} />
        <div>
          <strong>{isPreview ? "Preview runtime" : connected ? "Vault connected" : "Vault offline"}</strong>
          <p>
            {isPreview
              ? "Sample data only. Live scan and file writes run in the desktop app."
              : connected
                ? shortenPath(vaultPath)
                : "Open Setup to connect an Obsidian vault."}
          </p>
        </div>
      </div>

      <SidebarGroup label="Study">
        {APP_VIEWS.filter((view) => STUDY_VIEWS.includes(view.id)).map((view) => {
          const Icon = VIEW_ICONS[view.id];
          return (
            <button
              key={view.id}
              className={`nav-item ${activeView === view.id ? "nav-item--active" : ""}`}
              onClick={() => onChangeView(view.id)}
              type="button"
            >
              <span className="nav-item__label">
                <Icon size={16} strokeWidth={1.5} />
                {view.label}
              </span>
              {view.id === "logs" ? <span className="nav-item__badge">{logCount}</span> : null}
            </button>
          );
        })}
      </SidebarGroup>

      <SidebarGroup label="Configure">
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
                <Icon size={16} strokeWidth={1.5} />
                {view.label}
              </span>
            </button>
          );
        })}
      </SidebarGroup>

      <SidebarGroup
        action={
          <button className="sidebar__link" onClick={() => onChangeView("courses")} type="button">
            Manage
          </button>
        }
        label="Course library"
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
      </SidebarGroup>
    </aside>
  );
}

function SidebarGroup({
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
