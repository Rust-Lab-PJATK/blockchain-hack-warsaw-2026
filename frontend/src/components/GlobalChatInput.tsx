import { useEffect, useRef } from "react";
import { useTradingDashboardContext } from "@/context/TradingDashboardContext";

export function GlobalChatInput({ autoFocus }: { autoFocus?: boolean }) {
  const {
    input,
    setInput,
    send,
    isBootstrapping,
    isSending,
    lastError,
    hasPendingConfirmation,
    pendingActionSummary,
  } = useTradingDashboardContext();
  const inputRef = useRef<HTMLInputElement | null>(null);
  const isInteractionLocked =
    isBootstrapping || isSending || hasPendingConfirmation;

  useEffect(() => {
    if (autoFocus && inputRef.current) {
      // small timeout to ensure mobile sheet mount and possible keyboard readiness
      const t = setTimeout(() => inputRef.current?.focus(), 60);
      return () => clearTimeout(t);
    }
  }, [autoFocus]);

  return (
    <div className="td-chat-input-wrap td-global-chat-input">
      <div className="td-chat-input-box">
        <input
          ref={inputRef}
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
                : hasPendingConfirmation
                  ? "Confirm or cancel pending action first"
                : "Ask the agent..."
          }
          disabled={isInteractionLocked}
          className="td-chat-input"
        />
        <button
          type="button"
          onClick={send}
          disabled={isInteractionLocked}
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
      {hasPendingConfirmation && pendingActionSummary ? (
        <div className="td-pending-hint">
          Pending confirmation: {pendingActionSummary}
        </div>
      ) : null}
      {lastError ? <div className="td-error-text">{lastError}</div> : null}
    </div>
  );
}

export default GlobalChatInput;
