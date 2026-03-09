import { startTransition, useDeferredValue, useEffect, useState } from "react";
import "./App.css";
import {
  chooseVaultDirectory,
  connectVault,
  deleteCourse,
  disconnectVault,
  generateFlashcards,
  generateNoteAiInsight,
  generateRevisionNote,
  getRuntimeMode,
  getDashboard,
  getNoteDetails,
  loadWorkspace,
  runScan,
  saveAiSettings,
  saveCourseConfig,
  startAiEnrichment,
  validateAiSettings,
} from "./api";
import { AppSidebar } from "./components/AppSidebar";
import { InspectorPane } from "./components/InspectorPane";
import { MainPane } from "./components/MainPane";
import { type AppView, type NoteFilter, getViewMeta } from "./components/appShell";
import { Topbar } from "./components/Topbar";
import {
  createCourseDraft,
  DEFAULT_AI_SETTINGS,
  EMPTY_WORKSPACE,
  getErrorMessage,
  type BannerState,
  type CourseDraft,
} from "./lib";
import type {
  AiSettingsInput,
  CourseConfigInput,
  FlashcardGenerationResult,
  NoteDetails,
  RevisionNoteResult,
  ScanReport,
  WorkspaceSnapshot,
} from "./types";

type EditableCourseField = "name" | "folder" | "examDate" | "revisionFolder" | "flashcardsFolder";

function App() {
  const runtimeMode = getRuntimeMode();
  const isPreview = runtimeMode === "browser-preview";
  const [workspace, setWorkspace] = useState<WorkspaceSnapshot>(EMPTY_WORKSPACE);
  const [activeView, setActiveView] = useState<AppView>("overview");
  const [selectedCourseId, setSelectedCourseId] = useState<string | null>(null);
  const [selectedNoteId, setSelectedNoteId] = useState<string | null>(null);
  const [selectedNoteIds, setSelectedNoteIds] = useState<string[]>([]);
  const [noteFilter, setNoteFilter] = useState<NoteFilter>("all");
  const [aiFilter, setAiFilter] = useState<"all" | "ready" | "needs-work" | "failed">("all");
  const [noteDetails, setNoteDetails] = useState<NoteDetails | null>(null);
  const [vaultPath, setVaultPath] = useState("");
  const [courseDraft, setCourseDraft] = useState<CourseDraft>(createCourseDraft());
  const [aiDraft, setAiDraft] = useState<AiSettingsInput>(DEFAULT_AI_SETTINGS);
  const [banner, setBanner] = useState<BannerState>(null);
  const [busyAction, setBusyAction] = useState<string | null>(null);
  const [scanReport, setScanReport] = useState<ScanReport | null>(null);
  const [flashcardResult, setFlashcardResult] = useState<FlashcardGenerationResult | null>(null);
  const [revisionResult, setRevisionResult] = useState<RevisionNoteResult | null>(null);

  const deferredCourseId = useDeferredValue(selectedCourseId);
  const dashboard = workspace.dashboard;
  const selectedCourse = workspace.courses.find((course) => course.id === selectedCourseId) ?? null;
  const viewMeta = getViewMeta(activeView);
  const showInspector =
    activeView === "notes" || activeView === "courses" || activeView === "settings" || activeView === "ai";
  const aiIsRunning = dashboard?.ai.status === "running";

  useEffect(() => {
    void refreshWorkspace();
  }, []);

  useEffect(() => {
    setVaultPath(workspace.vault?.vaultPath ?? "");
  }, [workspace.vault]);

  useEffect(() => {
    setAiDraft(
      workspace.aiSettings
        ? {
            baseUrl: workspace.aiSettings.baseUrl,
            model: workspace.aiSettings.model,
            apiKey: "",
            enabled: workspace.aiSettings.enabled,
            timeoutMs: workspace.aiSettings.timeoutMs,
          }
        : DEFAULT_AI_SETTINGS,
    );
  }, [workspace.aiSettings]);

  useEffect(() => {
    if (!workspace.vault && !isPreview) {
      startTransition(() => {
        setSelectedCourseId(null);
        setSelectedNoteId(null);
        setSelectedNoteIds([]);
      });
      return;
    }

    const nextCourseId =
      selectedCourseId && workspace.courses.some((course) => course.id === selectedCourseId)
        ? selectedCourseId
        : workspace.selectedCourseId ?? workspace.courses[0]?.id ?? null;

    if (nextCourseId !== selectedCourseId) {
      startTransition(() => setSelectedCourseId(nextCourseId));
    }
  }, [isPreview, selectedCourseId, workspace.courses, workspace.selectedCourseId, workspace.vault]);

  useEffect(() => {
    if ((!workspace.vault && !isPreview) || deferredCourseId === workspace.dashboard?.selectedCourseId) {
      return;
    }

    let cancelled = false;
    void getDashboard(deferredCourseId)
      .then((nextDashboard) => {
        if (!cancelled) {
          setWorkspace((current) => ({
            ...current,
            dashboard: nextDashboard,
            selectedCourseId: deferredCourseId,
          }));
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setErrorBanner("Dashboard refresh failed", error);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [deferredCourseId, isPreview, workspace.dashboard?.selectedCourseId, workspace.vault]);

  useEffect(() => {
    const availableNoteIds = new Set(dashboard?.notes.map((note) => note.id) ?? []);
    setSelectedNoteIds((current) => current.filter((noteId) => availableNoteIds.has(noteId)));

    if (selectedNoteId && !availableNoteIds.has(selectedNoteId)) {
      setSelectedNoteId(null);
    }
  }, [dashboard, selectedNoteId]);

  useEffect(() => {
    if (activeView === "notes" && !selectedNoteId && dashboard?.notes[0]) {
      setSelectedNoteId(dashboard.notes[0].id);
    }
  }, [activeView, dashboard, selectedNoteId]);

  useEffect(() => {
    if (!selectedNoteId) {
      setNoteDetails(null);
      return;
    }

    let cancelled = false;
    void getNoteDetails(selectedNoteId)
      .then((details) => {
        if (!cancelled) {
          setNoteDetails(details);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setErrorBanner("Note details unavailable", error);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedNoteId]);

  useEffect(() => {
    if (courseDraft.id && !workspace.courses.some((course) => course.id === courseDraft.id)) {
      setCourseDraft(createCourseDraft());
    }
  }, [courseDraft.id, workspace.courses]);

  useEffect(() => {
    if (!workspace.vault || !selectedCourseId || !aiIsRunning) {
      return;
    }

    let cancelled = false;
    const refresh = () => {
      void getDashboard(selectedCourseId)
        .then((nextDashboard) => {
          if (!cancelled) {
            setWorkspace((current) => ({
              ...current,
              dashboard: nextDashboard,
            }));
          }
        })
        .catch((error) => {
          if (!cancelled) {
            setErrorBanner("AI refresh failed", error);
          }
        });

      if (selectedNoteId) {
        void getNoteDetails(selectedNoteId)
          .then((details) => {
            if (!cancelled) {
              setNoteDetails(details);
            }
          })
          .catch(() => {
            // Keep the current note pane stable while background AI is running.
          });
      }
    };

    refresh();
    const interval = window.setInterval(refresh, 2200);
    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [aiIsRunning, selectedCourseId, selectedNoteId, workspace.vault]);

  async function refreshWorkspace() {
    try {
      const snapshot = await loadWorkspace();
      applyWorkspace(snapshot);
      if (!isPreview && !snapshot.vault) {
        setActiveView("settings");
      }
      setBanner({
        tone: "neutral",
        title: isPreview ? "Browser preview ready" : snapshot.vault ? "Workspace ready" : "Connect a vault to begin",
        detail: isPreview
          ? "This view uses sample content only. Open the Tauri desktop app for live vault browsing, scanning, and file writes."
          : snapshot.vault
            ? "Local vault access is available. Choose a course, run a scan, and work from the note graph."
            : "Add a vault in Settings, then save a course folder and run the first scan.",
      });
    } catch (error) {
      setWorkspace(EMPTY_WORKSPACE);
      setErrorBanner("Workspace failed to load", error);
    }
  }

  function applyWorkspace(next: WorkspaceSnapshot) {
    setWorkspace(next);
    setScanReport(null);
    setFlashcardResult(null);
    setRevisionResult(null);
    setNoteDetails(null);
    const nextCourseId =
      next.selectedCourseId ??
      (selectedCourseId && next.courses.some((course) => course.id === selectedCourseId)
        ? selectedCourseId
        : next.courses[0]?.id ?? null);

    startTransition(() => {
      setSelectedCourseId(nextCourseId);
      setSelectedNoteId(null);
      setSelectedNoteIds([]);
      setNoteFilter("all");
      setAiFilter("all");
    });
  }

  function setErrorBanner(title: string, error: unknown) {
    setBanner({ tone: "error", title, detail: getErrorMessage(error) });
  }

  async function runBusyTask<T>(action: string, task: () => Promise<T>, onSuccess: (result: T) => void) {
    setBusyAction(action);
    try {
      onSuccess(await task());
    } catch (error) {
      setErrorBanner(action, error);
    } finally {
      setBusyAction(null);
    }
  }

  function changeView(view: AppView) {
    if (view === "courses" && !courseDraft.id && selectedCourse) {
      setCourseDraft(createCourseDraft(selectedCourse));
    }
    setActiveView(view);
  }

  function focusCourse(courseId: string, nextView: AppView = activeView) {
    const course = workspace.courses.find((entry) => entry.id === courseId) ?? null;
    if (course) {
      setCourseDraft(createCourseDraft(course));
    }

    startTransition(() => {
      setSelectedCourseId(courseId);
      setSelectedNoteId(null);
      setSelectedNoteIds([]);
      setNoteFilter("all");
      setAiFilter("all");
      setActiveView(nextView);
    });
  }

  function focusNote(noteId: string) {
    startTransition(() => {
      setSelectedNoteId(noteId);
      setActiveView("notes");
    });
  }

  function updateCourseField(field: EditableCourseField, value: string) {
    setCourseDraft((current) => ({ ...current, [field]: value }));
  }

  function updateAiField<K extends keyof AiSettingsInput>(field: K, value: AiSettingsInput[K]) {
    setAiDraft((current) => ({ ...current, [field]: value }));
  }

  function resetCourseDraft() {
    setCourseDraft(createCourseDraft(selectedCourse ?? undefined));
  }

  function startNewCourse() {
    setCourseDraft(createCourseDraft());
    setActiveView("courses");
  }

  const connect = () => {
    if (!vaultPath.trim()) {
      setBanner({
        tone: "error",
        title: "Vault path required",
        detail: isPreview
          ? "Enter a path if you want to relabel the preview workspace. Live folder access still requires the desktop app."
          : "Enter an absolute path before connecting the vault.",
      });
      return;
    }

    void runBusyTask("Vault connection failed", () => connectVault(vaultPath.trim()), (next) => {
      applyWorkspace(next);
      setActiveView("overview");
      setBanner({
        tone: isPreview ? "neutral" : "success",
        title: isPreview ? "Preview path updated" : "Vault connected",
        detail: isPreview
          ? "Browser preview does not touch the filesystem. The entered path is shown only as local context."
          : vaultPath.trim(),
      });
    });
  };

  const browseVault = () =>
    void runBusyTask("Vault picker failed", chooseVaultDirectory, (selected) => {
      if (!selected) {
        setBanner({ tone: "neutral", title: "Vault picker closed" });
        return;
      }

      setVaultPath(selected);
      setBanner({ tone: "neutral", title: "Vault path selected", detail: selected });
    });

  const disconnect = () =>
    void runBusyTask("Vault disconnect failed", disconnectVault, (next) => {
      applyWorkspace(next);
      setCourseDraft(createCourseDraft());
      setActiveView("settings");
      setBanner({
        tone: "neutral",
        title: isPreview ? "Preview workspace cleared" : "Vault disconnected",
        detail: isPreview
          ? "Refresh to reload sample content, or open the desktop app for live access."
          : "Course context has been cleared from the current session.",
      });
    });

  const saveCourse = () => {
    if (!courseDraft.name.trim() || !courseDraft.folder.trim()) {
      setBanner({
        tone: "error",
        title: "Course details missing",
        detail: "Name and folder are required before saving a course.",
      });
      return;
    }

    const input: CourseConfigInput = {
      id: courseDraft.id,
      name: courseDraft.name.trim(),
      folder: courseDraft.folder.trim(),
      examDate: courseDraft.examDate || null,
      revisionFolder: courseDraft.revisionFolder.trim(),
      flashcardsFolder: courseDraft.flashcardsFolder.trim(),
    };

    void runBusyTask("Course save failed", () => saveCourseConfig(input), (next) => {
      applyWorkspace(next);
      const nextCourse =
        next.courses.find((course) => course.name === input.name && course.folder === input.folder) ??
        next.courses.find((course) => course.id === input.id) ??
        null;

      if (nextCourse) {
        setCourseDraft(createCourseDraft(nextCourse));
        setSelectedCourseId(nextCourse.id);
      } else {
        setCourseDraft(createCourseDraft());
      }

      setActiveView("courses");
      setBanner({
        tone: "success",
        title: input.id ? "Course updated" : "Course created",
        detail: input.name,
      });
    });
  };

  const removeCourse = (courseId: string) =>
    void runBusyTask("Course delete failed", () => deleteCourse(courseId), (next) => {
      applyWorkspace(next);
      if (courseDraft.id === courseId) {
        setCourseDraft(createCourseDraft(next.courses[0] ?? undefined));
      }
      setBanner({ tone: "neutral", title: "Course removed" });
    });

  const scan = () => {
    setBanner({
      tone: "neutral",
      title: isPreview ? "Refreshing preview index" : `Scanning ${selectedCourse?.name ?? "course"}`,
      detail: isPreview
        ? "Refreshing the sample workspace."
        : "Reading markdown files, updating the index, and rebuilding links. The page refreshes when the scan finishes.",
    });

    void runBusyTask("Scan failed", runScan, ({ workspace: next, report }) => {
      applyWorkspace(next);
      setScanReport(report);
      const aiStarted = next.dashboard?.ai.status === "running";

      setBanner({
        tone: "success",
        title: isPreview ? "Preview index refreshed" : "Scan completed",
        detail: aiStarted
          ? `${report.changedNotes} changed, ${report.unchangedNotes} unchanged, ${report.generatedWeakLinks} weak-note suggestions. AI enrichment started in the background.`
          : `${report.changedNotes} changed, ${report.unchangedNotes} unchanged, ${report.generatedWeakLinks} weak-note suggestions.`,
      });
    });
  };

  const validateAi = () =>
    void runBusyTask("AI validation failed", () => validateAiSettings(aiDraft), (result) => {
      setBanner({
        tone: result.ok ? "success" : "error",
        title: result.ok ? "AI settings validated" : "AI settings rejected",
        detail: result.message,
      });
    });

  const saveAi = () =>
    void runBusyTask("AI settings save failed", () => saveAiSettings(aiDraft), (next) => {
      applyWorkspace(next);
      setBanner({
        tone: "success",
        title: "AI settings saved",
        detail: aiDraft.enabled
          ? "AI enrichment is armed. The app can start course enrichment after scans."
          : "Local extraction remains the active path.",
      });
    });

  const beginCourseAiEnrichment = (force = false, nextCourseId = selectedCourseId) => {
    if (!nextCourseId) {
      setBanner({
        tone: "error",
        title: "Course required",
        detail: "Choose a course before starting AI enrichment.",
      });
      return;
    }

    void runBusyTask(
      "AI enrichment start failed",
      () => startAiEnrichment(nextCourseId, force),
      (summary) => {
        setWorkspace((current) =>
          current.dashboard && current.dashboard.selectedCourseId === nextCourseId
            ? { ...current, dashboard: { ...current.dashboard, ai: summary } }
            : current,
        );
        setBanner({
          tone: "success",
          title: force ? "AI refresh started" : "AI enrichment started",
          detail: `${summary.pendingNotes || summary.missingNotes || summary.staleNotes} notes are being prepared in the background.`,
        });
      },
    );
  };

  const createFlashcards = () => {
    if (!selectedCourseId || selectedNoteIds.length === 0) {
      setBanner({
        tone: "error",
        title: "Select notes first",
        detail: "Queue one or more notes from the Notes view before generating flashcards.",
      });
      return;
    }

    void runBusyTask(
      "Flashcard generation failed",
      () =>
        generateFlashcards({
          courseId: selectedCourseId,
          noteIds: selectedNoteIds,
          flashcardsFolder: selectedCourse?.flashcardsFolder ?? "Flashcards",
          exportCsv: true,
        }),
      (result) => {
        setFlashcardResult(result);
        setBanner({
          tone: "success",
          title: "Flashcards generated",
          detail: `${result.cardCount} cards exported to markdown${result.csvPath ? " and CSV" : ""}.`,
        });
      },
    );
  };

  const createRevisionNote = () => {
    if (!selectedCourseId) {
      setBanner({
        tone: "error",
        title: "Course required",
        detail: "Choose a course before generating the revision note.",
      });
      return;
    }

    void runBusyTask(
      "Revision note generation failed",
      () =>
        generateRevisionNote({
          courseId: selectedCourseId,
          revisionFolder: selectedCourse?.revisionFolder ?? "Revision",
        }),
      (result) => {
        setRevisionResult(result);
        setBanner({
          tone: "success",
          title: "Revision note created",
          detail: result.notePath,
        });
      },
    );
  };

  const createNoteAiInsight = () => {
    if (!selectedNoteId) {
      setBanner({
        tone: "error",
        title: "Select a note first",
        detail: "Choose a note in the Notes view before generating an AI study brief.",
      });
      return;
    }

    void runBusyTask("AI note insight failed", () => generateNoteAiInsight(selectedNoteId), (result) => {
      setNoteDetails((current) =>
        current && current.id === result.noteId
          ? { ...current, aiInsight: result, aiStatus: "complete", aiError: null }
          : current,
      );
      setBanner({
        tone: "success",
        title: "AI study brief ready",
        detail: result.summary,
      });
    });
  };

  return (
    <div className={`shell shell--${activeView}`}>
      <AppSidebar
        activeView={activeView}
        connected={Boolean(workspace.vault)}
        courses={workspace.courses}
        runtimeMode={runtimeMode}
        selectedCourseId={selectedCourseId}
        vaultPath={workspace.vault?.vaultPath ?? ""}
        onChangeView={changeView}
        onSelectCourse={(courseId) => focusCourse(courseId, "overview")}
      />
      <div className="workspace-shell">
        <Topbar
          activeView={activeView}
          aiStatus={dashboard?.ai ?? null}
          busyAction={busyAction}
          dashboard={dashboard}
          runtimeMode={runtimeMode}
          scanStatus={workspace.scanStatus}
          selectedCourse={selectedCourse}
          title={viewMeta.label}
          onRunAi={() => beginCourseAiEnrichment(false)}
          onRefresh={() => void refreshWorkspace()}
          onScan={scan}
        />
        {banner ? (
          <section className={`banner banner--${banner.tone}`}>
            <strong>{banner.title}</strong>
            {banner.detail ? <span>{banner.detail}</span> : null}
          </section>
        ) : null}
        <div className={`workspace-body workspace-body--${activeView} ${showInspector ? "workspace-body--with-inspector" : "workspace-body--single"}`}>
          <main className="content-pane">
            <MainPane
              activeView={activeView}
              aiFilter={aiFilter}
              busyAction={busyAction}
              flashcardResult={flashcardResult}
              noteFilter={noteFilter}
              revisionResult={revisionResult}
              runtimeMode={runtimeMode}
              scanReport={scanReport}
              selectedCourse={selectedCourse}
              selectedNoteId={selectedNoteId}
              selectedNoteIds={selectedNoteIds}
              workspace={workspace}
              onChangeAiFilter={setAiFilter}
              onChangeNoteFilter={setNoteFilter}
              onCreateRevisionNote={createRevisionNote}
              onGenerateFlashcards={createFlashcards}
              onGenerateCourseAi={() => beginCourseAiEnrichment(false)}
              onRefreshCourseAi={() => beginCourseAiEnrichment(true)}
              onOpenNote={focusNote}
              onSelectCourse={(courseId) => focusCourse(courseId, "courses")}
              onStartNewCourse={startNewCourse}
              onToggleNoteSelection={(noteId) =>
                setSelectedNoteIds((current) =>
                  current.includes(noteId)
                    ? current.filter((entry) => entry !== noteId)
                    : [...current, noteId],
                )
              }
            />
          </main>
          {showInspector ? (
            <InspectorPane
              activeView={activeView}
              aiDraft={aiDraft}
              busyAction={busyAction}
              courseDraft={courseDraft}
              flashcardResult={flashcardResult}
              hasSavedApiKey={Boolean(workspace.aiSettings?.hasApiKey)}
              noteDetails={noteDetails}
              revisionResult={revisionResult}
              runtimeMode={runtimeMode}
              scanReport={scanReport}
              selectedCourse={selectedCourse}
              selectedNoteId={selectedNoteId}
              selectedNoteIds={selectedNoteIds}
              vaultPath={vaultPath}
              workspace={workspace}
              onBrowseVault={browseVault}
              onConnectVault={connect}
              onDeleteCourse={removeCourse}
              onDisconnectVault={disconnect}
              onGenerateCourseAi={() => beginCourseAiEnrichment(false)}
              onResetCourseDraft={resetCourseDraft}
              onSaveAiSettings={saveAi}
              onSaveCourse={saveCourse}
              onGenerateNoteAiInsight={createNoteAiInsight}
              onUpdateAiField={updateAiField}
              onUpdateCourseField={updateCourseField}
              onValidateAiSettings={validateAi}
              onVaultPathChange={setVaultPath}
            />
          ) : null}
        </div>
      </div>
    </div>
  );
}

export default App;
