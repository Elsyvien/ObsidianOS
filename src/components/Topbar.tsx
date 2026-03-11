import { formatDateTime } from "../lib";
import type { AiCourseSummary, CourseConfig, DashboardData, ScanStatus } from "../types";
import type { AppView } from "./appShell";
import { BrandMark } from "./BrandMark";

type TopbarProps = {
  activeView: AppView;
  aiStatus: AiCourseSummary | null;
  busyAction: string | null;
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
  const headline =
    activeView === "overview"
      ? selectedCourse?.name ?? "Exam workspace"
      : activeView === "ai"
        ? selectedCourse
          ? `${selectedCourse.name} AI workspace`
          : "AI workspace"
        : activeView === "notes"
          ? selectedCourse
            ? `${selectedCourse.name} notes`
            : "Notes"
        : title;
  const statusItems = [
    isPreview ? "Preview" : "Desktop",
    selectedCourse ? dashboard?.countdown.label ?? "Course selected" : "No course selected",
    activeView === "ai" && aiStatus ? `AI ${aiStatus.status}` : null,
    scanStatus?.lastScanAt ? `Last scan ${formatDateTime(scanStatus.lastScanAt)}` : "No scan yet",
  ].filter(Boolean);

  return (
    <header className="topbar">
      <div className="topbar__copy">
        <div className="topbar__brand">
          <BrandMark className="brand-mark brand-mark--topbar" />
          <span>ObsidianOS</span>
        </div>
        <span className="topbar__eyebrow">
          {title}
          {selectedCourse ? <strong>{selectedCourse.folder}</strong> : null}
        </span>
        <h2>{headline}</h2>
        <div className="meta-pills">
          {statusItems.map((item) => (
            <span key={item} className="meta-pill">
              {item}
            </span>
          ))}
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
            {busyAction === "Scan failed" ? "Scanning..." : "Scan course"}
          </button>
          <button className="button button--ghost" disabled={busyAction !== null} onClick={onRefresh} type="button">
            Refresh
          </button>
        </div>
      </div>
    </header>
  );
}
