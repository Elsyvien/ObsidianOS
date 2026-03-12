import { formatDateTime } from "../lib";
import type { AiCourseSummary, ChatScope, CourseConfig, DashboardData, ScanStatus } from "../types";
import type { AppView } from "./appShell";
import { BrandMark } from "./BrandMark";

type TopbarProps = {
  activeView: AppView;
  aiStatus: AiCourseSummary | null;
  busyAction: string | null;
  chatScope: ChatScope;
  dashboard: DashboardData | null;
  runtimeMode: "tauri" | "browser-preview";
  scanStatus: ScanStatus | null;
  selectedCourse: CourseConfig | null;
  title: string;
  onRunAi: () => void;
  onRefresh: () => void;
  onScan: () => void;
};

export function Topbar({
  activeView,
  aiStatus,
  busyAction,
  chatScope,
  dashboard,
  runtimeMode,
  scanStatus,
  selectedCourse,
  title,
  onRunAi,
  onRefresh,
  onScan,
}: TopbarProps) {
  const isPreview = runtimeMode === "browser-preview";
  const canScan = isPreview || selectedCourse !== null;
  const workspaceSummary = selectedCourse
    ? `${selectedCourse.folder} · ${dashboard?.graph.noteCount ?? 0} indexed notes`
    : isPreview
      ? "Preview dataset loaded for layout review"
      : "Select a course to start reviewing notes, formulas, and exam readiness";
  const headline = (() => {
    switch (activeView) {
      case "overview":
        return selectedCourse?.name ?? "Study workspace";
      case "statistics":
        return selectedCourse ? `${selectedCourse.name} statistics` : "Vault statistics";
      case "notes":
        return selectedCourse ? `${selectedCourse.name} notes` : "Notes";
      case "formulas":
        return selectedCourse ? `${selectedCourse.name} formula library` : "Formulas";
      case "ai":
        return selectedCourse ? `${selectedCourse.name} AI workspace` : "AI workspace";
      case "exams":
        return selectedCourse ? `${selectedCourse.name} exam engine` : "Exams";
      case "chat":
        return chatScope === "course"
          ? selectedCourse
            ? `${selectedCourse.name} grounded chat`
            : "Course chat"
          : "Vault chat";
      default:
        return title;
    }
  })();
  const statusItems = [
    isPreview ? "Preview" : "Desktop",
    selectedCourse ? dashboard?.countdown.label ?? "Course selected" : "No course selected",
    activeView === "ai" && aiStatus ? `AI ${aiStatus.status}` : null,
    scanStatus?.lastScanAt ? `Last scan ${formatDateTime(scanStatus.lastScanAt)}` : "No scan yet",
  ].filter(Boolean);

  return (
    <header className="topbar">
      <div className="topbar__inner">
        <div className="topbar__copy">
          <div className="topbar__eyebrow">
            <div className="topbar__brand">
              <BrandMark className="brand-mark brand-mark--topbar" />
              <span>ObsidianOS</span>
            </div>
            <span className="topbar__divider">/</span>
            <span>{title}</span>
            {selectedCourse ? (
              <>
                <span className="topbar__divider">/</span>
                <strong>{selectedCourse.folder}</strong>
              </>
            ) : null}
          </div>
          <h2>{headline}</h2>
          <div className="topbar__meta-row">
            <p className="topbar__summary">{workspaceSummary}</p>
            <div className="meta-pills">
              {statusItems.map((item) => (
                <span key={item} className="meta-pill">
                  {item}
                </span>
              ))}
            </div>
          </div>
        </div>

        <div className="topbar__meta">
          <div className="topbar__actions">
            {activeView === "ai" ? (
              <button
                className="button button--subtle"
                disabled={!selectedCourse || busyAction !== null || aiStatus?.status === "running"}
                onClick={onRunAi}
                type="button"
              >
                {aiStatus?.status === "running" ? "AI running..." : "Run AI"}
              </button>
            ) : null}
            <button className="button button--subtle" disabled={!canScan || busyAction !== null} onClick={onScan} type="button">
              {busyAction === "Scan failed" ? "Scanning..." : "Scan vault"}
            </button>
            <button className="button button--ghost" disabled={busyAction !== null} onClick={onRefresh} type="button">
              Refresh
            </button>
          </div>
        </div>
      </div>
    </header>
  );
}
