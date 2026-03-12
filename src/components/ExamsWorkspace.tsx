import { useEffect, useState } from "react";
import { formatDateTime, shortenPath } from "../lib";
import type {
  CourseConfig,
  ExamAttemptResult,
  ExamAttemptSummary,
  ExamBuilderInput,
  ExamDefaults,
  ExamDetails,
  ExamDifficulty,
  ExamPreset,
  ExamQuestion,
  ExamReviewAction,
  ExamSourceNote,
  ExamWorkspaceSnapshot,
  NoteDetails,
  NoteMasteryState,
} from "../types";
import { MarkdownContent } from "./MarkdownContent";

type ExamsWorkspaceProps = {
  busyAction: string | null;
  examAttemptResult: ExamAttemptResult | null;
  examDefaults: ExamDefaults;
  examDetails: ExamDetails | null;
  examDraft: ExamBuilderInput | null;
  examWorkspace: ExamWorkspaceSnapshot | null;
  noteDetails: NoteDetails | null;
  selectedCourse: CourseConfig | null;
  selectedNoteIds: string[];
  onAddQueuedNotes: () => void;
  onApplyReviewActions: (actions: ExamReviewAction[]) => void;
  onChangeDefaultField: <K extends keyof ExamDefaults>(field: K, value: ExamDefaults[K]) => void;
  onChangeDraftField: <K extends keyof ExamBuilderInput>(field: K, value: ExamBuilderInput[K]) => void;
  onClearSourceQueue: () => void;
  onOpenNote: (noteId: string) => void;
  onQueueExams: () => void;
  onRemoveSourceNote: (noteId: string) => void;
  onSelectExam: (examId: string | null) => void;
  onSubmitExam: (answers: Record<string, string>) => void;
  onUsePreset: (preset: ExamPreset) => void;
};

type ReaderMode = "split" | "maximized";

export function ExamsWorkspace({
  busyAction,
  examAttemptResult,
  examDefaults,
  examDetails,
  examDraft,
  examWorkspace,
  noteDetails,
  selectedCourse,
  selectedNoteIds,
  onAddQueuedNotes,
  onApplyReviewActions,
  onChangeDefaultField,
  onChangeDraftField,
  onClearSourceQueue,
  onOpenNote,
  onQueueExams,
  onRemoveSourceNote,
  onSelectExam,
  onSubmitExam,
  onUsePreset,
}: ExamsWorkspaceProps) {
  const [readerMode, setReaderMode] = useState<ReaderMode>("split");
  const [selectedQuestionId, setSelectedQuestionId] = useState<string | null>(null);
  const [answers, setAnswers] = useState<Record<string, string>>({});
  const [reviewActions, setReviewActions] = useState<Record<string, ExamReviewAction>>({});
  const examFeed = [
    ...(examWorkspace?.queuedExams ?? []),
    ...(examWorkspace?.readyExams ?? []),
    ...(examWorkspace?.failedExams ?? []),
  ];
  const latestAttempt = examWorkspace?.history[0] ?? null;
  const currentQuestion = examDetails?.questions.find((question) => question.id === selectedQuestionId) ?? null;
  const currentNoteDetails = noteDetails?.id === currentQuestion?.sourceNoteId ? noteDetails : null;

  useEffect(() => {
    if (!examDetails?.questions.length) {
      setSelectedQuestionId(null);
      setAnswers({});
      return;
    }

    setSelectedQuestionId((current) =>
      current && examDetails.questions.some((question) => question.id === current)
        ? current
        : examDetails.questions[0].id,
    );
    setAnswers(Object.fromEntries(examDetails.questions.map((question) => [question.id, ""])));
  }, [examDetails?.id, examDetails?.questions]);

  useEffect(() => {
    if (!examAttemptResult) {
      setReviewActions({});
      return;
    }

    setReviewActions(
      Object.fromEntries(
        examAttemptResult.noteSuggestions.map((suggestion) => [
          suggestion.noteId,
          {
            noteId: suggestion.noteId,
            nextState: suggestion.recommendedState,
            addToExamQueue: suggestion.recommendedState === "review" && !suggestion.currentlyInSourceQueue,
          },
        ]),
      ),
    );
  }, [examAttemptResult]);

  useEffect(() => {
    if (currentQuestion) {
      onOpenNote(currentQuestion.sourceNoteId);
    }
  }, [currentQuestion, onOpenNote]);

  const activeResult =
    examAttemptResult && examDetails && examAttemptResult.examId === examDetails.id ? examAttemptResult : null;

  if (!selectedCourse) {
    return (
      <div className="page-stack">
        <section className="surface surface--hero">
          <span className="surface__eyebrow">Exams</span>
          <h3>Select a course first</h3>
          <p>Create or choose a course before building an exam queue.</p>
        </section>
      </div>
    );
  }

  if (examDetails) {
    return (
      <div className="page-stack">
        <section className="surface surface--hero">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Exam session</span>
              <h3>{examDetails.title}</h3>
            </div>
            <div className="button-row">
              <button className="button button--ghost" onClick={() => onSelectExam(null)} type="button">
                Back to exams
              </button>
              <button
                className={`button button--ghost ${readerMode === "maximized" ? "button--active" : ""}`}
                onClick={() => setReaderMode((current) => (current === "split" ? "maximized" : "split"))}
                type="button"
              >
                {readerMode === "split" ? "Maximize reader" : "Split view"}
              </button>
            </div>
          </div>
          <div className="metric-strip">
            <Metric label="Preset" value={capitalize(examDetails.preset)} />
            <Metric label="Difficulty" value={capitalize(examDetails.difficulty)} />
            <Metric label="Questions" value={String(examDetails.questionCount)} />
            <Metric label="Time" value={`${examDetails.timeLimitMinutes} min`} />
          </div>
        </section>

        {activeResult ? (
          <ExamResultView
            busyAction={busyAction}
            examDetails={examDetails}
            result={activeResult}
            reviewActions={reviewActions}
            onApply={() => onApplyReviewActions(Object.values(reviewActions))}
            onChangeAction={(noteId, next) =>
              setReviewActions((current) => ({
                ...current,
                [noteId]: next,
              }))
            }
            onOpenNote={onOpenNote}
          />
        ) : examDetails.status !== "ready" ? (
          <section className="surface">
            <EmptyState
              title={examDetails.status === "failed" ? "Exam generation failed" : "Exam is still being prepared"}
              description={examDetails.lastError ?? "The exam queue is still generating. Stay on this page and it will refresh."}
            />
          </section>
        ) : (
          <section className={`surface exam-session exam-session--${readerMode}`}>
            <div className="exam-session__rail">
              <div className="surface__header">
                <div>
                  <span className="surface__eyebrow">Questions</span>
                  <h3>Navigator</h3>
                </div>
              </div>
              <div className="exam-question-list">
                {examDetails.questions.map((question) => (
                  <button
                    key={question.id}
                    className={`exam-question-pill ${selectedQuestionId === question.id ? "exam-question-pill--active" : ""}`}
                    onClick={() => setSelectedQuestionId(question.id)}
                    type="button"
                  >
                    <span>Q{question.index}</span>
                    <strong>{question.type === "multiple-choice" ? "MCQ" : "Short"}</strong>
                    <small>{answers[question.id] ? "Answered" : "Open"}</small>
                  </button>
                ))}
              </div>
            </div>

            <div className="exam-session__main">
              {currentQuestion ? (
                <ExamQuestionCard
                  answer={answers[currentQuestion.id] ?? ""}
                  noteDetails={currentNoteDetails}
                  question={currentQuestion}
                  readerMode={readerMode}
                  onChangeAnswer={(value) =>
                    setAnswers((current) => ({
                      ...current,
                      [currentQuestion.id]: value,
                    }))
                  }
                  onOpenNote={onOpenNote}
                />
              ) : (
                <EmptyState
                  title="Select a question"
                  description="Choose a question from the navigator to begin the exam."
                />
              )}
              <div className="button-row">
                <button
                  className="button button--subtle"
                  disabled={!currentQuestion}
                  onClick={() => {
                    if (!currentQuestion || !examDetails) {
                      return;
                    }
                    const currentIndex = examDetails.questions.findIndex((question) => question.id === currentQuestion.id);
                    const nextQuestion = examDetails.questions[currentIndex + 1];
                    if (nextQuestion) {
                      setSelectedQuestionId(nextQuestion.id);
                    }
                  }}
                  type="button"
                >
                  Next question
                </button>
                <button
                  className="button button--subtle"
                  disabled={busyAction !== null}
                  onClick={() => onSubmitExam(answers)}
                  type="button"
                >
                  {busyAction === "Exam submission failed" ? "Submitting..." : "Submit exam"}
                </button>
              </div>
            </div>
          </section>
        )}
      </div>
    );
  }

  return (
    <div className="page-stack">
      <section className="surface surface--hero">
        <div className="surface__header">
          <div>
            <span className="surface__eyebrow">Exams</span>
            <h3>{selectedCourse.name} exam engine</h3>
          </div>
          <div className="button-row">
            <button
              className="button button--subtle"
              disabled={busyAction !== null || selectedNoteIds.length === 0}
              onClick={onAddQueuedNotes}
              type="button"
            >
              Add queued notes
            </button>
            <button
              className="button button--subtle"
              disabled={busyAction !== null || !examDraft || !examWorkspace?.sourceQueue.length}
              onClick={onQueueExams}
              type="button"
            >
              {busyAction === "Exam queue failed" ? "Queueing..." : "Queue exams"}
            </button>
          </div>
        </div>
        <p className="surface__summary">
          Build mixed exams from a dedicated source queue, run them inside the app, and rebalance what you still need to learn after grading.
        </p>
        <div className="metric-strip">
          <Metric label="Source queue" value={String(examWorkspace?.summary.sourceQueueCount ?? 0)} />
          <Metric label="Ready exams" value={String(examWorkspace?.summary.readyCount ?? 0)} />
          <Metric
            label="Generating"
            value={String((examWorkspace?.summary.generatingCount ?? 0) + (examWorkspace?.summary.queuedCount ?? 0))}
          />
          <Metric label="Review notes" value={String(examWorkspace?.summary.reviewCount ?? 0)} />
          <Metric
            label="Latest attempt"
            value={latestAttempt?.submittedAt ? formatDateTime(latestAttempt.submittedAt) : "None yet"}
          />
        </div>
      </section>

      <section className="surface surface--split">
        <ExamBuilderPanel
          busyAction={busyAction}
          defaults={examDefaults}
          draft={examDraft}
          onChangeDefaultField={onChangeDefaultField}
          onChangeDraftField={onChangeDraftField}
          onQueueExams={onQueueExams}
          onUsePreset={onUsePreset}
        />
        <section className="exam-panel">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Source queue</span>
              <h3>Notes used for generation</h3>
            </div>
            <div className="button-row">
              <button
                className="button button--ghost"
                disabled={busyAction !== null || !(examWorkspace?.sourceQueue.length)}
                onClick={onClearSourceQueue}
                type="button"
              >
                Clear queue
              </button>
            </div>
          </div>
          {examWorkspace?.sourceQueue.length ? (
            <div className="row-list row-list--compact">
              {examWorkspace.sourceQueue.map((note) => (
                <article key={note.noteId} className="row-item">
                  <button className="row-item__main" onClick={() => onOpenNote(note.noteId)} type="button">
                    <div className="row-item__title-row">
                      <span className="row-item__title">{note.title}</span>
                      <span className={`soft-badge soft-badge--${masteryTone(note.masteryState)}`}>
                        {note.masteryState}
                      </span>
                    </div>
                    <span className="row-item__subtitle">{note.relativePath}</span>
                  </button>
                  <div className="row-item__meta">
                    <span>{note.conceptCount} concepts</span>
                    <span>{note.formulaCount} formulas</span>
                    <span>{note.lastAccuracy !== null ? `${note.lastAccuracy}% accuracy` : "No attempts yet"}</span>
                  </div>
                  <button className="button button--ghost" onClick={() => onRemoveSourceNote(note.noteId)} type="button">
                    Remove
                  </button>
                </article>
              ))}
            </div>
          ) : (
            <EmptyState
              title="No notes in the exam queue"
              description="Add notes from Notes or AI, or use the current queue button above."
            />
          )}
        </section>
      </section>

      <section className="surface surface--split">
        <section className="exam-panel">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Generated exams</span>
              <h3>Queue and ready sets</h3>
            </div>
          </div>
          {examFeed.length ? (
            <div className="row-list row-list--compact">
              {examFeed.map((exam) => (
                <article key={exam.id} className="row-item">
                  <button className="row-item__main" onClick={() => onSelectExam(exam.id)} type="button">
                    <div className="row-item__title-row">
                      <span className="row-item__title">{exam.title}</span>
                      <span className={`soft-badge soft-badge--${examTone(exam.status)}`}>{exam.status}</span>
                    </div>
                    <span className="row-item__subtitle">
                      {formatDateTime(exam.updatedAt)} · {exam.questionCount} questions · {exam.timeLimitMinutes} min
                    </span>
                  </button>
                  <div className="row-item__meta">
                    <span>{exam.sourceNoteCount} notes</span>
                    <span>{exam.attemptCount} attempts</span>
                    <span>{exam.latestScorePercent !== null ? `${exam.latestScorePercent}%` : "No score yet"}</span>
                  </div>
                  <button className="button button--ghost" onClick={() => onSelectExam(exam.id)} type="button">
                    {exam.status === "ready" ? "Open" : "View"}
                  </button>
                </article>
              ))}
            </div>
          ) : (
            <EmptyState
              title="No exams generated yet"
              description="Queue the first exam once the source queue has the notes you want."
            />
          )}
        </section>

        <section className="exam-panel">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Attempt history</span>
              <h3>Recent exam runs</h3>
            </div>
          </div>
          {examWorkspace?.history.length ? (
            <div className="line-list">
              {examWorkspace.history.map((attempt) => (
                <HistoryRow key={attempt.id} attempt={attempt} />
              ))}
            </div>
          ) : (
            <EmptyState
              title="No attempts yet"
              description="History will appear here once you complete the first generated exam."
            />
          )}
        </section>
      </section>

      <section className="surface surface--split">
        <section className="exam-panel">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Review queue</span>
              <h3>Notes you still need to revisit</h3>
            </div>
          </div>
          {examWorkspace?.reviewNotes.length ? (
            <div className="line-list">
              {examWorkspace.reviewNotes.map((note) => (
                <ReviewNoteRow key={note.noteId} note={note} onOpenNote={onOpenNote} />
              ))}
            </div>
          ) : (
            <EmptyState
              title="Review queue is empty"
              description="After you submit an exam, missed notes can be moved here for another pass."
            />
          )}
        </section>

        <section className="exam-panel">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Mastered notes</span>
              <h3>Content you can put away</h3>
            </div>
          </div>
          {examWorkspace?.masteredNotes.length ? (
            <div className="line-list">
              {examWorkspace.masteredNotes.map((note) => (
                <ReviewNoteRow key={note.noteId} note={note} onOpenNote={onOpenNote} />
              ))}
            </div>
          ) : (
            <EmptyState
              title="Nothing mastered yet"
              description="Strong exam performance will move stable notes into the put-away lane."
            />
          )}
        </section>
      </section>
    </div>
  );
}

function ExamBuilderPanel({
  busyAction,
  defaults,
  draft,
  onChangeDefaultField,
  onChangeDraftField,
  onQueueExams,
  onUsePreset,
}: {
  busyAction: string | null;
  defaults: ExamDefaults;
  draft: ExamBuilderInput | null;
  onChangeDefaultField: <K extends keyof ExamDefaults>(field: K, value: ExamDefaults[K]) => void;
  onChangeDraftField: <K extends keyof ExamBuilderInput>(field: K, value: ExamBuilderInput[K]) => void;
  onQueueExams: () => void;
  onUsePreset: (preset: ExamPreset) => void;
}) {
  return (
    <section className="exam-panel">
      <div className="surface__header">
        <div>
          <span className="surface__eyebrow">Builder</span>
          <h3>Queue a new exam batch</h3>
        </div>
      </div>
      <div className="toolbar">
        {(["sprint", "mock", "final"] as ExamPreset[]).map((preset) => (
          <button
            key={preset}
            className={`toolbar__item ${draft?.preset === preset ? "toolbar__item--active" : ""}`}
            onClick={() => onUsePreset(preset)}
            type="button"
          >
            {capitalize(preset)}
          </button>
        ))}
      </div>
      <div className="form-grid">
        <Field label="Multiple choice">
          <input
            min={1}
            onChange={(event) => onChangeDraftField("multipleChoiceCount", Number(event.target.value) || 1)}
            type="number"
            value={draft?.multipleChoiceCount ?? 0}
          />
        </Field>
        <Field label="Short answer">
          <input
            min={1}
            onChange={(event) => onChangeDraftField("shortAnswerCount", Number(event.target.value) || 1)}
            type="number"
            value={draft?.shortAnswerCount ?? 0}
          />
        </Field>
        <Field label="Difficulty">
          <select
            onChange={(event) => onChangeDraftField("difficulty", event.target.value as ExamDifficulty)}
            value={draft?.difficulty ?? "mixed"}
          >
            <option value="easy">Easy</option>
            <option value="mixed">Mixed</option>
            <option value="hard">Hard</option>
          </select>
        </Field>
        <Field label="Time limit (min)">
          <input
            min={5}
            onChange={(event) => onChangeDraftField("timeLimitMinutes", Number(event.target.value) || 5)}
            type="number"
            value={draft?.timeLimitMinutes ?? 0}
          />
        </Field>
        <Field label="Generate count">
          <input
            max={5}
            min={1}
            onChange={(event) => onChangeDraftField("generateCount", Number(event.target.value) || 1)}
            type="number"
            value={draft?.generateCount ?? 1}
          />
        </Field>
      </div>
      <div className="button-row">
        <button className="button button--subtle" disabled={busyAction !== null || !draft} onClick={onQueueExams} type="button">
          {busyAction === "Exam queue failed" ? "Queueing..." : "Queue exams"}
        </button>
      </div>
      <div className="definition-grid">
        <Definition label="Default preset" value={capitalize(defaults.preset)} />
        <Definition label="Default difficulty" value={capitalize(defaults.difficulty)} />
        <Definition label="Default time" value={`${defaults.timeLimitMinutes} min`} />
        <Definition label="Default batch size" value={String(defaults.generateCount)} />
      </div>
      <div className="form-grid">
        <Field label="Save default preset">
          <select
            onChange={(event) => onChangeDefaultField("preset", event.target.value as ExamPreset)}
            value={defaults.preset}
          >
            <option value="sprint">Sprint</option>
            <option value="mock">Mock</option>
            <option value="final">Final</option>
          </select>
        </Field>
        <Field label="Save default difficulty">
          <select
            onChange={(event) => onChangeDefaultField("difficulty", event.target.value as ExamDifficulty)}
            value={defaults.difficulty}
          >
            <option value="easy">Easy</option>
            <option value="mixed">Mixed</option>
            <option value="hard">Hard</option>
          </select>
        </Field>
      </div>
    </section>
  );
}

function ExamQuestionCard({
  answer,
  noteDetails,
  question,
  readerMode,
  onChangeAnswer,
  onOpenNote,
}: {
  answer: string;
  noteDetails: NoteDetails | null;
  question: ExamQuestion;
  readerMode: ReaderMode;
  onChangeAnswer: (value: string) => void;
  onOpenNote: (noteId: string) => void;
}) {
  return (
    <div className={`exam-reader exam-reader--${readerMode}`}>
      <div className="exam-reader__question">
        <span className="surface__eyebrow">
          Question {question.index} · {question.type === "multiple-choice" ? "MCQ" : "Short answer"}
        </span>
        <MarkdownContent className="exam-reader__prompt" text={question.prompt} />
        {question.type === "multiple-choice" ? (
          <div className="exam-answer-list">
            {question.options.map((option) => (
              <label key={option} className={`exam-answer-option ${answer === option ? "exam-answer-option--active" : ""}`}>
                <input
                  checked={answer === option}
                  name={question.id}
                  onChange={() => onChangeAnswer(option)}
                  type="radio"
                />
                <MarkdownContent className="exam-answer-option__copy" text={option} />
              </label>
            ))}
          </div>
        ) : (
          <textarea
            className="exam-answer-textarea"
            onChange={(event) => onChangeAnswer(event.target.value)}
            placeholder="Write the answer in your own words."
            value={answer}
          />
        )}
      </div>
      <div className="exam-reader__context">
        <div className="surface__header">
          <div>
            <span className="surface__eyebrow">Source note</span>
            <h3>{noteDetails?.title ?? question.sourceNoteTitle}</h3>
          </div>
          <button className="button button--ghost" onClick={() => onOpenNote(question.sourceNoteId)} type="button">
            Open in Notes
          </button>
        </div>
        {noteDetails ? (
          <div className="insight-stack">
            <MarkdownContent className="inspector-copy" text={noteDetails.excerpt} />
            <dl className="definition-grid">
              <Definition label="Path" value={shortenPath(noteDetails.relativePath)} />
              <Definition label="Links" value={String(noteDetails.links.length)} />
            </dl>
            <ContextList title="Headings" items={noteDetails.headings} />
            <ContextList title="Concepts" items={noteDetails.concepts} />
            <ContextList title="Suggestions" items={noteDetails.suggestions} />
          </div>
        ) : (
          <EmptyState
            title="Loading note context"
            description="The note reader follows the currently selected question."
          />
        )}
      </div>
    </div>
  );
}

function ReviewNoteRow({
  note,
  onOpenNote,
}: {
  note: ExamSourceNote;
  onOpenNote: (noteId: string) => void;
}) {
  return (
    <button key={note.noteId} className="line-item" onClick={() => onOpenNote(note.noteId)} type="button">
      <span className="line-item__title">{note.title}</span>
      <span className="line-item__subtitle">{note.relativePath}</span>
      <span className="line-item__meta">
        {note.lastAccuracy !== null ? `${note.lastAccuracy}% last accuracy` : "Needs first review"} · {note.conceptCount} concepts
      </span>
    </button>
  );
}

function HistoryRow({ attempt }: { attempt: ExamAttemptSummary }) {
  return (
    <div className="line-item line-item--static">
      <span className="line-item__title">{attempt.examTitle}</span>
      <span className="line-item__subtitle">{formatDateTime(attempt.submittedAt)}</span>
      <span className="line-item__meta">
        {attempt.scorePercent}% score · {attempt.correctCount} correct · {attempt.partialCount} partial · {attempt.incorrectCount} incorrect
      </span>
    </div>
  );
}

function ExamResultView({
  busyAction,
  examDetails,
  result,
  reviewActions,
  onApply,
  onChangeAction,
  onOpenNote,
}: {
  busyAction: string | null;
  examDetails: ExamDetails;
  result: ExamAttemptResult;
  reviewActions: Record<string, ExamReviewAction>;
  onApply: () => void;
  onChangeAction: (noteId: string, next: ExamReviewAction) => void;
  onOpenNote: (noteId: string) => void;
}) {
  return (
    <>
      <section className="surface">
        <div className="surface__header">
          <div>
            <span className="surface__eyebrow">Result</span>
            <h3>{examDetails.title} review</h3>
          </div>
          <button className="button button--subtle" disabled={busyAction === "Exam review apply failed"} onClick={onApply} type="button">
            {busyAction === "Exam review apply failed" ? "Applying..." : "Apply learning update"}
          </button>
        </div>
        <div className="metric-strip">
          <Metric label="Score" value={`${result.scorePercent}%`} />
          <Metric label="Correct" value={String(result.correctCount)} />
          <Metric label="Partial" value={String(result.partialCount)} />
          <Metric label="Incorrect" value={String(result.incorrectCount)} />
        </div>
        <MarkdownContent className="surface__summary" text={result.overallFeedback} />
      </section>

      <section className="surface surface--split">
        <section className="exam-panel">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Question review</span>
              <h3>Why each answer was right or wrong</h3>
            </div>
          </div>
          <div className="line-list">
            {result.questionResults.map((question) => (
              <div key={question.questionId} className="line-item line-item--static">
                <span className="line-item__title">
                  Q{question.index} · {question.verdict}
                </span>
                <MarkdownContent className="line-item__meta" text={question.prompt} />
                <MarkdownContent className="line-item__meta" text={`Your answer: ${renderAnswer(question.userAnswer)}`} />
                <MarkdownContent className="line-item__meta" text={`Expected: ${question.expectedAnswer}`} />
                <MarkdownContent className="line-item__meta" text={question.feedback} />
              </div>
            ))}
          </div>
        </section>

        <section className="exam-panel">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">Learning actions</span>
              <h3>Decide what to keep, revisit, or retire</h3>
            </div>
          </div>
          <div className="line-list">
            {result.noteSuggestions.map((suggestion) => {
              const action = reviewActions[suggestion.noteId];
              return (
                <div key={suggestion.noteId} className="line-item line-item--static">
                  <span className="line-item__title">{suggestion.title}</span>
                  <span className="line-item__subtitle">{suggestion.relativePath}</span>
                  <span className="line-item__meta">
                    {suggestion.accuracy}% accuracy · suggested {suggestion.recommendedState}
                  </span>
                  <MarkdownContent className="line-item__meta" text={suggestion.reason} />
                  <div className="exam-review-actions">
                    <select
                      onChange={(event) =>
                        onChangeAction(suggestion.noteId, {
                          ...action,
                          noteId: suggestion.noteId,
                          nextState: event.target.value as NoteMasteryState,
                        })
                      }
                      value={action?.nextState ?? suggestion.recommendedState}
                    >
                      <option value="review">Review</option>
                      <option value="active">Active</option>
                      <option value="mastered">Mastered</option>
                    </select>
                    <label className="toggle">
                      <input
                        checked={action?.addToExamQueue ?? false}
                        onChange={(event) =>
                          onChangeAction(suggestion.noteId, {
                            ...action,
                            noteId: suggestion.noteId,
                            nextState: action?.nextState ?? suggestion.recommendedState,
                            addToExamQueue: event.target.checked,
                          })
                        }
                        type="checkbox"
                      />
                      <span>Add back to exam queue</span>
                    </label>
                    <button className="button button--ghost" onClick={() => onOpenNote(suggestion.noteId)} type="button">
                      Open note
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        </section>
      </section>
    </>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="field">
      <span>{label}</span>
      {children}
    </label>
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

function ContextList({ items, title }: { items: string[]; title: string }) {
  if (!items.length) {
    return null;
  }

  return (
    <div className="insight-list">
      <strong>{title}</strong>
      <ul>
        {items.slice(0, 5).map((item) => (
          <li key={`${title}-${item}`}>
            <MarkdownContent text={item} />
          </li>
        ))}
      </ul>
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

function examTone(status: string) {
  switch (status) {
    case "ready":
      return "success";
    case "failed":
      return "warning";
    default:
      return "neutral";
  }
}

function masteryTone(state: NoteMasteryState) {
  switch (state) {
    case "mastered":
      return "success";
    case "review":
      return "warning";
    default:
      return "neutral";
  }
}

function renderAnswer(answer: string | string[]) {
  return Array.isArray(answer) ? answer.join(", ") : answer || "No answer";
}

function capitalize(value: string) {
  return value.slice(0, 1).toUpperCase() + value.slice(1);
}
