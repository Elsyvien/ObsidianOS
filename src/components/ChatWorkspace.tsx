import { useEffect, useRef, useState } from "react";
import { formatDateTime, shortenPath } from "../lib";
import type {
  ChatMessage,
  ChatScope,
  ChatThreadDetails,
  ChatThreadSummary,
  CourseConfig,
} from "../types";

type ChatWorkspaceProps = {
  aiEnabled: boolean;
  busyAction: string | null;
  chatScope: ChatScope;
  courseThreads: ChatThreadSummary[];
  selectedCourse: CourseConfig | null;
  selectedThreadId: string | null;
  threadDetails: ChatThreadDetails | null;
  vaultThreads: ChatThreadSummary[];
  onChangeScope: (scope: ChatScope) => void;
  onCreateThread: () => void;
  onDeleteThread: (threadId: string) => void;
  onOpenNote: (noteId: string) => void;
  onSelectThread: (threadId: string, scope: ChatScope) => void;
  onSendMessage: (content: string) => void;
};

export function ChatWorkspace({
  aiEnabled,
  busyAction,
  chatScope,
  courseThreads,
  selectedCourse,
  selectedThreadId,
  threadDetails,
  vaultThreads,
  onChangeScope,
  onCreateThread,
  onDeleteThread,
  onOpenNote,
  onSelectThread,
  onSendMessage,
}: ChatWorkspaceProps) {
  const [composer, setComposer] = useState("");
  const transcriptRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    transcriptRef.current?.scrollTo({
      top: transcriptRef.current.scrollHeight,
      behavior: "smooth",
    });
  }, [threadDetails?.messages.length]);

  const sending = busyAction === "Chat send failed";

  if (!selectedCourse && chatScope === "course") {
    return (
      <div className="page-stack">
        <section className="surface surface--hero">
          <span className="surface__eyebrow">Chat</span>
          <h3>Select a course first</h3>
          <p>Course chat needs an active course. Switch to vault scope or choose a course from the sidebar.</p>
        </section>
      </div>
    );
  }

  return (
    <div className="page-stack">
      <section className="surface surface--hero">
        <div className="surface__header">
          <div>
            <span className="surface__eyebrow">Chat</span>
            <h3>{chatScope === "course" ? `${selectedCourse?.name ?? "Course"} grounded chat` : "Whole vault chat"}</h3>
          </div>
          <div className="button-row">
            <button className="button button--subtle" disabled={busyAction !== null} onClick={onCreateThread} type="button">
              New thread
            </button>
          </div>
        </div>
        <p className="surface__summary">
          Ask the current course or the whole vault a question. Answers stay notes-first, include citations, and clearly flag any fallback beyond your notes.
        </p>
        <div className="toolbar">
          {(["course", "vault"] as ChatScope[]).map((scope) => (
            <button
              key={scope}
              className={`toolbar__item ${chatScope === scope ? "toolbar__item--active" : ""}`}
              onClick={() => onChangeScope(scope)}
              type="button"
            >
              {scope === "course" ? "Current course" : "Whole vault"}
            </button>
          ))}
        </div>
      </section>

      <section className="surface chat-workspace">
        <aside className="chat-workspace__rail">
          <ThreadGroup
            activeScope={chatScope}
            emptyLabel="No current-course threads yet"
            scope="course"
            selectedThreadId={selectedThreadId}
            threads={courseThreads}
            title="Current course"
            onSelectThread={(threadId) => onSelectThread(threadId, "course")}
          />
          <ThreadGroup
            activeScope={chatScope}
            emptyLabel="No vault threads yet"
            scope="vault"
            selectedThreadId={selectedThreadId}
            threads={vaultThreads}
            title="Whole vault"
            onSelectThread={(threadId) => onSelectThread(threadId, "vault")}
          />
        </aside>

        <div className="chat-workspace__main">
          <div className="surface__header">
            <div>
              <span className="surface__eyebrow">
                {threadDetails ? `${threadDetails.scope === "course" ? "Course" : "Vault"} thread` : "Notes-first chat"}
              </span>
              <h3>{threadDetails?.title ?? "Start a grounded thread"}</h3>
            </div>
            {threadDetails ? (
              <button className="button button--ghost button--danger" onClick={() => onDeleteThread(threadDetails.id)} type="button">
                Delete thread
              </button>
            ) : null}
          </div>

          {!aiEnabled ? (
            <EmptyState
              title="AI is disabled"
              description="Enable AI in Setup before using notes-grounded chat."
            />
          ) : (
            <>
              <div className="chat-transcript" ref={transcriptRef}>
                {threadDetails?.messages.length ? (
                  threadDetails.messages.map((message) => (
                    <ChatMessageCard key={message.id} message={message} onOpenNote={onOpenNote} />
                  ))
                ) : (
                  <EmptyState
                    title="No messages yet"
                    description="Ask a question about the current course or the whole vault to start a persistent thread."
                  />
                )}
              </div>
              <div className="chat-composer">
                <label className="field">
                  <span>Question</span>
                  <textarea
                    className="exam-answer-textarea chat-composer__input"
                    onChange={(event) => setComposer(event.target.value)}
                    placeholder={
                      chatScope === "course"
                        ? "Ask the current course a grounded question."
                        : "Ask the whole vault and compare courses, topics, or formulas."
                    }
                    value={composer}
                  />
                </label>
                <div className="button-row">
                  <button
                    className="button button--subtle"
                    disabled={sending || !composer.trim() || (!selectedCourse && chatScope === "course")}
                    onClick={() => {
                      onSendMessage(composer.trim());
                      setComposer("");
                    }}
                    type="button"
                  >
                    {sending ? "Sending..." : "Send"}
                  </button>
                </div>
              </div>
            </>
          )}
        </div>
      </section>
    </div>
  );
}

function ThreadGroup({
  activeScope,
  emptyLabel,
  scope,
  selectedThreadId,
  threads,
  title,
  onSelectThread,
}: {
  activeScope: ChatScope;
  emptyLabel: string;
  scope: ChatScope;
  selectedThreadId: string | null;
  threads: ChatThreadSummary[];
  title: string;
  onSelectThread: (threadId: string) => void;
}) {
  return (
    <section className={`chat-thread-group ${activeScope === scope ? "chat-thread-group--active" : ""}`}>
      <div className="surface__header">
        <div>
          <span className="surface__eyebrow">{title}</span>
          <h3>Threads</h3>
        </div>
      </div>
      {threads.length ? (
        <div className="row-list row-list--compact">
          {threads.map((thread) => (
            <article key={thread.id} className={`row-item ${selectedThreadId === thread.id ? "row-item--active" : ""}`}>
              <button className="row-item__main" onClick={() => onSelectThread(thread.id)} type="button">
                <div className="row-item__title-row">
                  <span className="row-item__title">{thread.title}</span>
                  <span className="soft-badge">{thread.messageCount}</span>
                </div>
                <span className="row-item__subtitle">{thread.lastMessagePreview ?? "No messages yet"}</span>
              </button>
              <div className="row-item__meta">
                <span>{thread.courseName ?? "Whole vault"}</span>
                <span>{formatDateTime(thread.updatedAt)}</span>
              </div>
            </article>
          ))}
        </div>
      ) : (
        <EmptyState title="No saved threads" description={emptyLabel} />
      )}
    </section>
  );
}

function ChatMessageCard({
  message,
  onOpenNote,
}: {
  message: ChatMessage;
  onOpenNote: (noteId: string) => void;
}) {
  return (
    <article className={`chat-message chat-message--${message.role}`}>
      <div className="chat-message__meta">
        <span className="surface__eyebrow">{message.role === "assistant" ? "Assistant" : "You"}</span>
        <span className="line-item__meta">{formatDateTime(message.createdAt)}</span>
      </div>
      <div className="chat-message__body">
        <p>{message.content}</p>
        {message.usedFallback ? (
          <div className="chat-fallback">
            <strong>Fallback used</strong>
            <p>{message.fallbackReason ?? "The answer required context beyond the stored notes."}</p>
          </div>
        ) : null}
        {message.citations.length ? (
          <div className="chat-citations">
            {message.citations.map((citation) => (
              <button key={`${citation.chunkId}-${citation.noteId}`} className="chat-citation" onClick={() => onOpenNote(citation.noteId)} type="button">
                <strong>{citation.noteTitle}</strong>
                <span>{shortenPath(citation.relativePath)}</span>
                <span>{citation.headingPath}</span>
                <span>{citation.courseName}</span>
                <span>{citation.excerpt}</span>
              </button>
            ))}
          </div>
        ) : null}
      </div>
    </article>
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
