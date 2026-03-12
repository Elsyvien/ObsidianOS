import { useEffect, useRef, useState } from "react";
import { formatDateTime, shortenPath } from "../lib";
import type {
  ChatMessage,
  ChatScope,
  ChatThreadDetails,
  ChatThreadSummary,
  CourseConfig,
} from "../types";
import { MarkdownContent } from "./MarkdownContent";

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
  const sending = busyAction === "Chat send failed";
  const visibleThreads = chatScope === "course" ? courseThreads : vaultThreads;

  const submitMessage = () => {
    const nextMessage = composer.trim();
    if (!nextMessage || sending || (!selectedCourse && chatScope === "course")) {
      return;
    }

    onSendMessage(nextMessage);
    setComposer("");
  };

  useEffect(() => {
    transcriptRef.current?.scrollTo({
      top: transcriptRef.current.scrollHeight,
      behavior: "smooth",
    });
  }, [sending, threadDetails?.messages.length]);

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
    <div className="page-stack page-stack--chat">
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
      </section>

      <div className="chat-layout">
        <aside className="chat-layout__rail">
          <div className="chat-sidebar-group">
            <div className="surface__header">
              <div>
                <span className="surface__eyebrow">Scope</span>
                <h3>{chatScope === "course" ? "Current course" : "Whole vault"}</h3>
              </div>
            </div>
            <div className="toolbar chat-scope-toggle" role="tablist" aria-label="Chat scope">
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
          </div>

          <ThreadGroup
            emptyLabel={chatScope === "course" ? "No current-course threads yet" : "No vault threads yet"}
            scope={chatScope}
            selectedThreadId={selectedThreadId}
            threads={visibleThreads}
            title={chatScope === "course" ? "Current course threads" : "Whole vault threads"}
            onSelectThread={(threadId) => onSelectThread(threadId, chatScope)}
          />
        </aside>

        <div className="chat-layout__main">
          <div className="surface__header chat-layout__main-header">
            <div>
              <span className="surface__eyebrow">
                {threadDetails ? `${threadDetails.scope === "course" ? "Course" : "Vault"} thread` : "Notes-first chat"}
              </span>
              <h3>{threadDetails?.title ?? "Start a grounded thread"}</h3>
            </div>
            <div className="button-row">
              {sending ? <span className="meta-pill chat-status-pill">Assistant is replying</span> : null}
              {threadDetails ? (
                <button className="button button--ghost button--danger" onClick={() => onDeleteThread(threadDetails.id)} type="button">
                  Delete thread
                </button>
              ) : null}
            </div>
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
                  <>
                    {threadDetails.messages.map((message) => (
                      <ChatMessageCard key={message.id} message={message} onOpenNote={onOpenNote} />
                    ))}
                    {sending ? <StreamingMessageCard /> : null}
                  </>
                ) : (
                  sending ? (
                    <StreamingMessageCard />
                  ) : (
                    <EmptyState
                      title="No messages yet"
                      description="Ask a question about the current course or the whole vault to start a persistent thread."
                    />
                  )
                )}
              </div>
              <div className="chat-composer">
                <label className="field">
                  <span>Question</span>
                  <textarea
                    className="exam-answer-textarea chat-composer__input"
                    onChange={(event) => setComposer(event.target.value)}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" && !event.shiftKey && !event.nativeEvent.isComposing) {
                        event.preventDefault();
                        submitMessage();
                      }
                    }}
                    placeholder={
                      chatScope === "course"
                        ? "Ask the current course a grounded question."
                        : "Ask the whole vault and compare courses, topics, or formulas."
                    }
                    value={composer}
                  />
                  <span>Press Enter to send, Shift+Enter for a new line.</span>
                </label>
                <div className="button-row">
                  <button
                    className="button button--subtle"
                    disabled={sending || !composer.trim() || (!selectedCourse && chatScope === "course")}
                    onClick={submitMessage}
                    type="button"
                  >
                    {sending ? "Sending..." : "Send"}
                  </button>
                </div>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

function ThreadGroup({
  emptyLabel,
  selectedThreadId,
  threads,
  title,
  onSelectThread,
}: {
  emptyLabel: string;
  scope: ChatScope;
  selectedThreadId: string | null;
  threads: ChatThreadSummary[];
  title: string;
  onSelectThread: (threadId: string) => void;
}) {
  return (
    <section className="chat-thread-group">
      <div className="surface__header">
        <div>
          <span className="surface__eyebrow">{title}</span>
          <h3>{threads.length} saved</h3>
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
                <MarkdownContent
                  className="row-item__subtitle chat-thread-preview"
                  text={thread.lastMessagePreview ?? "No messages yet"}
                />
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
        <MarkdownContent className="chat-message__content" text={message.content} />
        {message.usedFallback ? (
          <div className="chat-fallback">
            <strong>Fallback used</strong>
            <MarkdownContent
              className="chat-fallback__reason"
              text={message.fallbackReason ?? "The answer required context beyond the stored notes."}
            />
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
                <MarkdownContent className="chat-citation__excerpt" text={citation.excerpt} />
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

function StreamingMessageCard() {
  return (
    <article className="chat-message chat-message--assistant chat-message--streaming" aria-live="polite">
      <div className="chat-message__meta">
        <span className="surface__eyebrow">Assistant</span>
        <span className="line-item__meta">Streaming reply</span>
      </div>
      <div className="chat-message__body">
        <div className="chat-streaming-indicator">
          <span className="chat-streaming-indicator__dot" />
          <span className="chat-streaming-indicator__dot" />
          <span className="chat-streaming-indicator__dot" />
        </div>
      </div>
    </article>
  );
}
