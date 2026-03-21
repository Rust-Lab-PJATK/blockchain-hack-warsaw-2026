import { RefObject } from "react";
import { Message } from "@/utils/types";
import { ParsedCard } from "@/components/ui/ParsedCard";

type ChatPanelProps = {
  messages: Message[];
  activeStrategiesCount: number;
  input: string;
  setInput: (value: string) => void;
  send: () => void;
  msgsRef: RefObject<HTMLDivElement | null>;
  isBootstrapping: boolean;
  isSending: boolean;
  lastError: string | null;
  connectionMode: "mock" | "http";
  onClose?: () => void;
  showInput?: boolean;
};

export function ChatPanel({
  messages,
  activeStrategiesCount,
  input,
  setInput,
  send,
  msgsRef,
  isBootstrapping,
  isSending,
  lastError,
  connectionMode,
  onClose,
  showInput = true,
}: ChatPanelProps) {
  const statusLabel =
    connectionMode === "http" ? "Backend connected" : "Mock mode";
  const hasError = Boolean(lastError);

  return (
    <div className="td-chat">
      <div className="td-chat-header">
        <div className="td-section-title">AI Agent</div>
        <div className="td-header-actions">
          <div
            className={`td-status-inline ${
              hasError ? "td-status-inline-error" : "td-status-inline-ok"
            }`}
          >
            <div className="td-status-dot" />
            {activeStrategiesCount} active · {statusLabel}
          </div>
          {onClose ? (
            <button
              type="button"
              className="td-chat-close"
              aria-label="Close agent chat"
              onClick={onClose}
            >
              ×
            </button>
          ) : null}
        </div>
      </div>

      <div ref={msgsRef} className="td-chat-list">
        {messages.map((m, i) => (
          <div key={`${m.role}-${i}`} className="td-chat-message">
            <div
              className={`td-chat-role ${
                m.role === "user" ? "td-chat-role-user" : "td-chat-role-agent"
              }`}
            >
              {m.role === "user" ? "You" : "Agent"}
            </div>
            <div
              className={`td-chat-bubble ${
                m.role === "user" ? "td-chat-bubble-user" : "td-chat-bubble-agent"
              }`}
            >
              {m.text}
              {m.parsed ? <ParsedCard data={m.parsed} /> : null}
            </div>
          </div>
        ))}
      </div>

      {showInput ? (
        <div className="td-chat-input-wrap">
          <div className="td-chat-input-box">
            <input
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  send();
                }
              }}
              placeholder={
                isBootstrapping
                  ? "Loading dashboard..."
                  : isSending
                    ? "Sending..."
                    : "Describe your strategy..."
              }
              disabled={isBootstrapping || isSending}
              className="td-chat-input"
            />
            <button
              type="button"
              onClick={send}
              disabled={isBootstrapping || isSending}
              className="td-send-btn"
            >
              <svg width="11" height="11" viewBox="0 0 12 12" fill="none">
                <path
                  d="M1 11L11 1M11 1H4M11 1V8"
                  stroke="white"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
            </button>
          </div>
          {lastError ? <div className="td-error-text">{lastError}</div> : null}
        </div>
      ) : null}
    </div>
  );
}
