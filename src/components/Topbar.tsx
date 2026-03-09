import { formatDateTime } from "../lib";
import type { CourseConfig, DashboardData, ScanStatus } from "../types";
import type { AppView } from "./appShell";

type TopbarProps = {
  activeView: AppView;
  busyAction: string | null;
  dashboard: DashboardData | null;
  runtimeMode: "tauri" | "browser-preview";
  scanStatus: ScanStatus | null;
  selectedCourse: CourseConfig | null;
  title: string;
  onRefresh: () => void;
  onScan: () => void;
};

export function Topbar({
  activeView,
  busyAction,
  dashboard,
  runtimeMode,
  scanStatus,
  selectedCourse,
  title,
  onRefresh,
  onScan,
}: TopbarProps) {
  const isPreview = runtimeMode === "browser-preview";
  const canScan = isPreview || selectedCourse !== null;
  const headline =
    activeView === "overview"
      ? selectedCourse?.name ?? "Exam workspace"
      : activeView === "notes"
        ? selectedCourse
          ? `${selectedCourse.name} notes`
          : "Notes"
        : title;
  const statusLine = [
    isPreview ? "Preview" : "Desktop",
    selectedCourse ? dashboard?.countdown.label ?? "Course selected" : "No course selected",
    scanStatus?.lastScanAt ? `Last scan ${formatDateTime(scanStatus.lastScanAt)}` : "No scan yet",
  ].join("   ·   ");

  return (
    <header className="topbar">
      <div className="topbar__copy">
        <span className="topbar__eyebrow">
          {title}
          {selectedCourse ? <strong>{selectedCourse.folder}</strong> : null}
        </span>
        <h2>{headline}</h2>
        <p>{statusLine}</p>
      </div>

      <div className="topbar__meta">
        <div className="topbar__actions">
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
