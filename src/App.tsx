import { startTransition, useDeferredValue, useEffect, useState } from "react";
import "./App.css";
import {
  addExamSourceNotes,
  applyExamReviewActions,
  chooseVaultDirectory,
  clearExamSourceQueue,
  connectVault,
  deleteCourse,
  disconnectVault,
  getExamDetails,
  getExamWorkspace,
  generateFlashcards,
  generateNoteAiInsight,
  generateRevisionNote,
  getRuntimeMode,
  getDashboard,
  getNoteDetails,
  loadWorkspace,
  queueExams,
  removeExamSourceNotes,
  runScan,
  saveAiSettings,
  saveCourseConfig,
  startAiEnrichment,
  submitExamAttempt,
  validateAiSettings,
} from "./api";
import { AppSidebar } from "./components/AppSidebar";
import { ExamsWorkspace } from "./components/ExamsWorkspace";
import { InspectorPane } from "./components/InspectorPane";
import { MainPane } from "./components/MainPane";
import { type AppView, type NoteFilter, getViewMeta } from "./components/appShell";
import { Topbar } from "./components/Topbar";
import {
  createCourseDraft,
  createExamBuilderInput,
  DEFAULT_AI_SETTINGS,
  DEFAULT_EXAM_DEFAULTS,
  EMPTY_WORKSPACE,
  getErrorMessage,
  type BannerState,
  type CourseDraft,
} from "./lib";
import type {
  ApplyExamReviewActionsRequest,
  ActivityLogEntry,
  AiSettingsInput,
  CourseConfigInput,
  ExamAttemptResult,
  ExamBuilderInput,
  ExamDefaults,
  ExamDetails,
  ExamWorkspaceSnapshot,
  ExamSubmissionRequest,
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
  const [activityLog, setActivityLog] = useState<ActivityLogEntry[]>([]);
  const [examWorkspace, setExamWorkspace] = useState<ExamWorkspaceSnapshot | null>(null);
  const [selectedExamId, setSelectedExamId] = useState<string | null>(null);
  const [examDetails, setExamDetails] = useState<ExamDetails | null>(null);
  const [examAttemptResult, setExamAttemptResult] = useState<ExamAttemptResult | null>(null);
  const [examDefaults, setExamDefaults] = useState<ExamDefaults>(() => {
    if (typeof window === "undefined") {
      return DEFAULT_EXAM_DEFAULTS;
    }

    const raw = window.localStorage.getItem("obsidian-exam-os.exam-defaults");
    if (!raw) {
      return DEFAULT_EXAM_DEFAULTS;
    }

    try {
      return { ...DEFAULT_EXAM_DEFAULTS, ...JSON.parse(raw) } as ExamDefaults;
    } catch {
      return DEFAULT_EXAM_DEFAULTS;
    }
  });
  const [examDraft, setExamDraft] = useState<ExamBuilderInput | null>(null);

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
    if (typeof window === "undefined") {
      return;
    }

    window.localStorage.setItem("obsidian-exam-os.exam-defaults", JSON.stringify(examDefaults));
  }, [examDefaults]);

  useEffect(() => {
    if (!selectedCourseId) {
      setExamDraft(null);
      return;
    }

    setExamDraft((current) =>
      current?.courseId === selectedCourseId ? current : createExamBuilderInput(selectedCourseId, examDefaults),
    );
  }, [examDefaults, selectedCourseId]);

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
    if ((!workspace.vault && !isPreview) || !selectedCourseId) {
      setExamWorkspace(null);
      return;
    }

    let cancelled = false;
    void getExamWorkspace(selectedCourseId)
      .then((nextWorkspace) => {
        if (!cancelled) {
          setExamWorkspace(nextWorkspace);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setErrorBanner("Exam workspace failed to load", error);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [isPreview, selectedCourseId, workspace.vault]);

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
    if (!selectedExamId) {
      setExamDetails(null);
      setExamAttemptResult(null);
      return;
    }

    let cancelled = false;
    void getExamDetails(selectedExamId)
      .then((details) => {
        if (!cancelled) {
          setExamDetails(details);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setErrorBanner("Exam details unavailable", error);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedExamId]);

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
    const availableExamIds = new Set(
      [
        ...(examWorkspace?.queuedExams ?? []),
        ...(examWorkspace?.readyExams ?? []),
        ...(examWorkspace?.failedExams ?? []),
      ].map((exam) => exam.id),
    );
    if (selectedExamId && !availableExamIds.has(selectedExamId)) {
      setSelectedExamId(null);
      setExamDetails(null);
      setExamAttemptResult(null);
    }
  }, [examWorkspace, selectedExamId]);

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

  useEffect(() => {
    if (!selectedCourseId || !examWorkspace) {
      return;
    }

    const isExamGenerating = examWorkspace.summary.queuedCount > 0 || examWorkspace.summary.generatingCount > 0;
    if (!isExamGenerating) {
      return;
    }

    let cancelled = false;
    const refresh = () => {
      void getExamWorkspace(selectedCourseId)
        .then((nextWorkspace) => {
          if (!cancelled) {
            setExamWorkspace(nextWorkspace);
          }
        })
        .catch((error) => {
          if (!cancelled) {
            setErrorBanner("Exam refresh failed", error);
          }
        });

      if (selectedExamId) {
        void getExamDetails(selectedExamId)
          .then((details) => {
            if (!cancelled) {
              setExamDetails(details);
            }
          })
          .catch(() => {
            // Keep current exam view stable during generation.
          });
      }
    };

    const interval = window.setInterval(refresh, 1800);
    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [examWorkspace, selectedCourseId, selectedExamId]);

  async function refreshWorkspace() {
    try {
      const snapshot = await loadWorkspace();
      applyWorkspace(snapshot);
      if (!isPreview && !snapshot.vault) {
        setActiveView("settings");
      }
      showBanner({
        tone: "neutral",
        title: isPreview ? "Browser preview ready" : snapshot.vault ? "Workspace ready" : "Connect a vault to begin",
        detail: isPreview
          ? "This view uses sample content only. Open the Tauri desktop app for live vault browsing, scanning, and file writes."
          : snapshot.vault
            ? "Local vault access is available. Choose a course, run a scan, and work from the note graph."
            : "Add a vault in Settings, then save a course folder and run the first scan.",
      }, "Overview");
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
    setExamWorkspace(null);
    setSelectedExamId(null);
    setExamDetails(null);
    setExamAttemptResult(null);
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
    showBanner({ tone: "error", title, detail: getErrorMessage(error) });
  }

  function showBanner(nextBanner: NonNullable<BannerState>, scope = viewMeta.label) {
    setBanner(nextBanner);
    setActivityLog((current) =>
      [
        {
          id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
          timestamp: new Date().toISOString(),
          scope,
          title: nextBanner.title,
          detail: nextBanner.detail ?? "",
          tone: nextBanner.tone,
        },
        ...current,
      ].slice(0, 80),
    );
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

  function focusNote(noteId: string, nextView: AppView = "notes") {
    startTransition(() => {
      setSelectedNoteId(noteId);
      setActiveView(nextView);
    });
  }

  function applyAiSelection(noteIds: string[], label: string, mode: "append" | "replace" = "replace") {
    const availableNoteIds = new Set(dashboard?.notes.map((note) => note.id) ?? []);
    const nextSelection = Array.from(new Set(noteIds.filter((noteId) => availableNoteIds.has(noteId))));

    if (!nextSelection.length) {
      showBanner({
        tone: "error",
        title: `${label} unavailable`,
        detail: "This course does not have enough notes in the required AI state yet.",
      }, "AI");
      return;
    }

    setSelectedNoteIds((current) =>
      mode === "append" ? Array.from(new Set([...current, ...nextSelection])) : nextSelection,
    );
    setSelectedNoteId(nextSelection[0] ?? null);
    setActiveView("ai");
    showBanner({
      tone: "neutral",
      title: label,
      detail: `${nextSelection.length} notes are now in the study queue.`,
    }, "AI");
  }

  function updateCourseField(field: EditableCourseField, value: string) {
    setCourseDraft((current) => ({ ...current, [field]: value }));
  }

  function updateAiField<K extends keyof AiSettingsInput>(field: K, value: AiSettingsInput[K]) {
    setAiDraft((current) => ({ ...current, [field]: value }));
  }

  function updateExamDefaultField<K extends keyof ExamDefaults>(field: K, value: ExamDefaults[K]) {
    setExamDefaults((current) => ({ ...current, [field]: value }));
  }

  function updateExamDraftField<K extends keyof ExamBuilderInput>(field: K, value: ExamBuilderInput[K]) {
    setExamDraft((current) => (current ? { ...current, [field]: value } : current));
  }

  function applyExamPreset(preset: "sprint" | "mock" | "final") {
    const presets = {
      sprint: { multipleChoiceCount: 6, shortAnswerCount: 2, difficulty: "mixed", timeLimitMinutes: 10, generateCount: 1 },
      mock: { multipleChoiceCount: 14, shortAnswerCount: 6, difficulty: "mixed", timeLimitMinutes: 25, generateCount: 1 },
      final: { multipleChoiceCount: 24, shortAnswerCount: 16, difficulty: "hard", timeLimitMinutes: 45, generateCount: 1 },
    } as const;

    setExamDraft((current) =>
      current
        ? {
            ...current,
            preset,
            ...presets[preset],
          }
        : current,
    );
  }

  function resetCourseDraft() {
    setCourseDraft(createCourseDraft(selectedCourse ?? undefined));
  }

  function startNewCourse() {
    setCourseDraft(createCourseDraft());
    setActiveView("courses");
  }

  function previewExamNote(noteId: string) {
    setSelectedNoteId(noteId);
  }

  const connect = () => {
    if (!vaultPath.trim()) {
      showBanner({
        tone: "error",
        title: "Vault path required",
        detail: isPreview
          ? "Enter a path if you want to relabel the preview workspace. Live folder access still requires the desktop app."
          : "Enter an absolute path before connecting the vault.",
      }, "Setup");
      return;
    }

    void runBusyTask("Vault connection failed", () => connectVault(vaultPath.trim()), (next) => {
      applyWorkspace(next);
      setActiveView("overview");
      showBanner({
        tone: isPreview ? "neutral" : "success",
        title: isPreview ? "Preview path updated" : "Vault connected",
        detail: isPreview
          ? "Browser preview does not touch the filesystem. The entered path is shown only as local context."
          : vaultPath.trim(),
      }, "Setup");
    });
  };

  const browseVault = () =>
    void runBusyTask("Vault picker failed", chooseVaultDirectory, (selected) => {
      if (!selected) {
        showBanner({ tone: "neutral", title: "Vault picker closed" }, "Setup");
        return;
      }

      setVaultPath(selected);
      showBanner({ tone: "neutral", title: "Vault path selected", detail: selected }, "Setup");
    });

  const disconnect = () =>
    void runBusyTask("Vault disconnect failed", disconnectVault, (next) => {
      applyWorkspace(next);
      setCourseDraft(createCourseDraft());
      setActiveView("settings");
      showBanner({
        tone: "neutral",
        title: isPreview ? "Preview workspace cleared" : "Vault disconnected",
        detail: isPreview
          ? "Refresh to reload sample content, or open the desktop app for live access."
          : "Course context has been cleared from the current session.",
      }, "Setup");
    });

  const saveCourse = () => {
    if (!courseDraft.name.trim() || !courseDraft.folder.trim()) {
      showBanner({
        tone: "error",
        title: "Course details missing",
        detail: "Name and folder are required before saving a course.",
      }, "Courses");
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
      showBanner({
        tone: "success",
        title: input.id ? "Course updated" : "Course created",
        detail: input.name,
      }, "Courses");
    });
  };

  const removeCourse = (courseId: string) =>
    void runBusyTask("Course delete failed", () => deleteCourse(courseId), (next) => {
      applyWorkspace(next);
      if (courseDraft.id === courseId) {
        setCourseDraft(createCourseDraft(next.courses[0] ?? undefined));
      }
      showBanner({ tone: "neutral", title: "Course removed" }, "Courses");
    });

  const scan = () => {
    showBanner({
      tone: "neutral",
      title: isPreview ? "Refreshing preview index" : `Scanning ${selectedCourse?.name ?? "course"}`,
      detail: isPreview
        ? "Refreshing the sample workspace."
        : "Reading markdown files, updating the index, and rebuilding links. The page refreshes when the scan finishes.",
    }, "Overview");

    void runBusyTask("Scan failed", runScan, ({ workspace: next, report }) => {
      applyWorkspace(next);
      setScanReport(report);
      const aiStarted = next.dashboard?.ai.status === "running";

      showBanner({
        tone: "success",
        title: isPreview ? "Preview index refreshed" : "Scan completed",
        detail: aiStarted
          ? `${report.changedNotes} changed, ${report.unchangedNotes} unchanged, ${report.generatedWeakLinks} weak-note suggestions. AI enrichment started in the background.`
          : `${report.changedNotes} changed, ${report.unchangedNotes} unchanged, ${report.generatedWeakLinks} weak-note suggestions.`,
      }, "Overview");
    });
  };

  const validateAi = () =>
    void runBusyTask("AI validation failed", () => validateAiSettings(aiDraft), (result) => {
      showBanner({
        tone: result.ok ? "success" : "error",
        title: result.ok ? "AI settings validated" : "AI settings rejected",
        detail: result.message,
      }, "Setup");
    });

  const saveAi = () =>
    void runBusyTask("AI settings save failed", () => saveAiSettings(aiDraft), (next) => {
      applyWorkspace(next);
      showBanner({
        tone: "success",
        title: "AI settings saved",
        detail: aiDraft.enabled
          ? "AI enrichment is armed. The app can start course enrichment after scans."
          : "Local extraction remains the active path.",
      }, "Setup");
    });

  const beginCourseAiEnrichment = (force = false, nextCourseId = selectedCourseId) => {
    if (!nextCourseId) {
      showBanner({
        tone: "error",
        title: "Course required",
        detail: "Choose a course before starting AI enrichment.",
      }, "AI");
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
        showBanner({
          tone: "success",
          title: force ? "AI refresh started" : "AI enrichment started",
          detail: `${summary.pendingNotes || summary.missingNotes || summary.staleNotes} notes are being prepared in the background.`,
        }, "AI");
      },
    );
  };

  const createFlashcards = () => {
    if (!selectedCourseId || selectedNoteIds.length === 0) {
      showBanner({
        tone: "error",
        title: "Select notes first",
        detail: "Queue one or more notes from the Notes view before generating flashcards.",
      }, "Outputs");
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
        showBanner({
          tone: "success",
          title: "Flashcards generated",
          detail: `${result.cardCount} cards exported to markdown${result.csvPath ? " and CSV" : ""}.`,
        }, "Outputs");
      },
    );
  };

  const createRevisionNote = () => {
    if (!selectedCourseId) {
      showBanner({
        tone: "error",
        title: "Course required",
        detail: "Choose a course before generating the revision note.",
      }, "Outputs");
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
        showBanner({
          tone: "success",
          title: "Revision note created",
          detail: result.notePath,
        }, "Outputs");
      },
    );
  };

  const createNoteAiInsight = () => {
    if (!selectedNoteId) {
      showBanner({
        tone: "error",
        title: "Select a note first",
        detail: "Choose a note in the Notes view before generating an AI study brief.",
      }, "Notes");
      return;
    }

    void runBusyTask("AI note insight failed", () => generateNoteAiInsight(selectedNoteId), (result) => {
      setNoteDetails((current) =>
        current && current.id === result.noteId
          ? { ...current, aiInsight: result, aiStatus: "complete", aiError: null }
          : current,
      );
      showBanner({
        tone: "success",
        title: "AI study brief ready",
        detail: result.summary,
      }, "Notes");
    });
  };

  const refreshExamWorkspace = (courseId = selectedCourseId) => {
    if (!courseId) {
      return Promise.resolve(null);
    }

    return getExamWorkspace(courseId).then((next) => {
      setExamWorkspace(next);
      return next;
    });
  };

  const addQueuedNotesToExamQueue = () => {
    if (!selectedCourseId || selectedNoteIds.length === 0) {
      showBanner({
        tone: "error",
        title: "Select notes first",
        detail: "Queue one or more notes in Notes or AI before adding them to the exam source queue.",
      }, "Exams");
      return;
    }

    void runBusyTask(
      "Exam source queue update failed",
      () => addExamSourceNotes(selectedCourseId, selectedNoteIds),
      (nextWorkspace) => {
        setExamWorkspace(nextWorkspace);
        showBanner({
          tone: "success",
          title: "Exam source queue updated",
          detail: `${selectedNoteIds.length} notes were added to the exam source queue.`,
        }, "Exams");
      },
    );
  };

  const addNotesToExamQueue = (noteIds: string[]) => {
    if (!selectedCourseId || noteIds.length === 0) {
      return;
    }

    void runBusyTask(
      "Exam source queue update failed",
      () => addExamSourceNotes(selectedCourseId, noteIds),
      (nextWorkspace) => {
        setExamWorkspace(nextWorkspace);
        showBanner({
          tone: "success",
          title: "Added to exam queue",
          detail: `${noteIds.length} note${noteIds.length === 1 ? "" : "s"} added to the exam source queue.`,
        }, "Exams");
      },
    );
  };

  const removeSourceNote = (noteId: string) => {
    if (!selectedCourseId) {
      return;
    }

    void runBusyTask(
      "Exam source queue update failed",
      () => removeExamSourceNotes(selectedCourseId, [noteId]),
      (nextWorkspace) => {
        setExamWorkspace(nextWorkspace);
      },
    );
  };

  const clearSourceQueue = () => {
    if (!selectedCourseId) {
      return;
    }

    void runBusyTask(
      "Exam source queue update failed",
      () => clearExamSourceQueue(selectedCourseId),
      (nextWorkspace) => {
        setExamWorkspace(nextWorkspace);
        showBanner({
          tone: "neutral",
          title: "Exam source queue cleared",
        }, "Exams");
      },
    );
  };

  const queueExamBatch = () => {
    if (!examDraft) {
      showBanner({
        tone: "error",
        title: "Course required",
        detail: "Choose a course before queueing exams.",
      }, "Exams");
      return;
    }

    void runBusyTask("Exam queue failed", () => queueExams(examDraft), (nextWorkspace) => {
      setExamWorkspace(nextWorkspace);
      showBanner({
        tone: "success",
        title: "Exams queued",
        detail: `${examDraft.generateCount} exam${examDraft.generateCount === 1 ? "" : "s"} added to the generation queue.`,
      }, "Exams");
      setActiveView("exams");
    });
  };

  const submitExam = (answers: Record<string, string>) => {
    if (!examDetails) {
      return;
    }

    const request: ExamSubmissionRequest = {
      examId: examDetails.id,
      answers: examDetails.questions.map((question) => ({
        questionId: question.id,
        answer: answers[question.id] ?? "",
      })),
    };

    void runBusyTask("Exam submission failed", () => submitExamAttempt(request), (result) => {
      setExamAttemptResult(result);
      void refreshExamWorkspace();
      showBanner({
        tone: "success",
        title: "Exam submitted",
        detail: `Score ${result.scorePercent}% · ${result.incorrectCount} question${result.incorrectCount === 1 ? "" : "s"} still need work.`,
      }, "Exams");
    });
  };

  const applyReviewActions = (actions: ApplyExamReviewActionsRequest["actions"]) => {
    if (!examAttemptResult) {
      return;
    }

    void runBusyTask(
      "Exam review apply failed",
      () =>
        applyExamReviewActions({
          attemptId: examAttemptResult.attemptId,
          actions,
        }),
      (nextWorkspace) => {
        setExamWorkspace(nextWorkspace);
        showBanner({
          tone: "success",
          title: "Learning queue updated",
          detail: "Mastery and review recommendations were applied.",
        }, "Exams");
      },
    );
  };

  return (
    <div className={`shell shell--${activeView}`}>
      <AppSidebar
        activeView={activeView}
        connected={Boolean(workspace.vault)}
        courses={workspace.courses}
        logCount={activityLog.length}
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
            {activeView === "exams" ? (
              <ExamsWorkspace
                busyAction={busyAction}
                examAttemptResult={examAttemptResult}
                examDefaults={examDefaults}
                examDetails={examDetails}
                examDraft={examDraft}
                examWorkspace={examWorkspace}
                noteDetails={noteDetails}
                selectedCourse={selectedCourse}
                selectedNoteIds={selectedNoteIds}
                onAddQueuedNotes={addQueuedNotesToExamQueue}
                onApplyReviewActions={applyReviewActions}
                onChangeDefaultField={updateExamDefaultField}
                onChangeDraftField={updateExamDraftField}
                onClearSourceQueue={clearSourceQueue}
                onOpenNote={previewExamNote}
                onQueueExams={queueExamBatch}
                onRemoveSourceNote={removeSourceNote}
                onSelectExam={(examId) => {
                  setSelectedExamId(examId);
                  setExamAttemptResult(null);
                }}
                onSubmitExam={submitExam}
                onUsePreset={applyExamPreset}
              />
            ) : (
              <MainPane
                activeView={activeView}
                activityLog={activityLog}
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
                onOpenAiNote={(noteId) => focusNote(noteId, "ai")}
                onApplyAiSelection={applyAiSelection}
                onAddNotesToExamQueue={addNotesToExamQueue}
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
            )}
          </main>
          {showInspector ? (
            <InspectorPane
              activeView={activeView}
              aiDraft={aiDraft}
              busyAction={busyAction}
              courseDraft={courseDraft}
              examDefaults={examDefaults}
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
              onToggleNoteSelection={(noteId) =>
                setSelectedNoteIds((current) =>
                  current.includes(noteId)
                    ? current.filter((entry) => entry !== noteId)
                    : [...current, noteId],
                )
              }
              onResetCourseDraft={resetCourseDraft}
              onSaveAiSettings={saveAi}
              onSaveCourse={saveCourse}
              onGenerateNoteAiInsight={createNoteAiInsight}
              onUpdateAiField={updateAiField}
              onUpdateExamDefaultField={updateExamDefaultField}
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
