"use client";

import { useEffect, useRef, useState, useCallback, useSyncExternalStore } from "react";
import GlobalChatInput from "@/components/GlobalChatInput";
import { ChatPanel } from "@/components/ChatPanel";
import { PortfolioPanel } from "@/components/PortfolioPanel";
import { Topbar } from "@/components/Topbar";
import { TradingPanel } from "@/components/TradingPanel";
import {
  TradingDashboardProvider,
  useTradingDashboardContext,
} from "@/context/TradingDashboardContext";

function DashboardContent() {
  const [isMobileChatOpen, setIsMobileChatOpen] = useState(false);
  const mobileChatRef = useRef<HTMLDivElement | null>(null);
  const touchStartYRef = useRef<number | null>(null);
  const touchCurrentYRef = useRef<number | null>(null);
  const startHeightRef = useRef<number | null>(null);
  const [mobileChatHeight, setMobileChatHeight] = useState<number | null>(null);
  const touchFromListRef = useRef(false);

  const isDesktop = useSyncExternalStore(
    (onStoreChange) => {
      if (typeof window === "undefined") {
        return () => {};
      }

      const mediaQuery = window.matchMedia("(min-width: 1025px)");
      const handler = () => onStoreChange();
      mediaQuery.addEventListener("change", handler);

      return () => {
        mediaQuery.removeEventListener("change", handler);
      };
    },
    () => {
      if (typeof window === "undefined") {
        return false;
      }

      return window.matchMedia("(min-width: 1025px)").matches;
    },
    () => false,
  );
  const minMobileChatHeight = 240;

  const getMaxMobileChatHeight = useCallback(() => {
    return Math.round(window.innerHeight * 0.85);
  }, []);

  const clampChatHeight = useCallback(
    (height: number) => {
      return Math.min(Math.max(height, minMobileChatHeight), getMaxMobileChatHeight());
    },
    [getMaxMobileChatHeight],
  );

  const {
    activeNav,
    setActiveNav,
    activeTab,
    setActiveTab,
    solPrice,
    chg,
    positions,
    totalPnl,
    strategies,
    fund,
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
  } = useTradingDashboardContext();

  useEffect(() => {
    document.body.classList.toggle("td-mobile-chat-lock", !isDesktop && isMobileChatOpen);

    return () => {
      document.body.classList.remove("td-mobile-chat-lock");
    };
  }, [isDesktop, isMobileChatOpen]);

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isMobileChatOpen) {
        setIsMobileChatOpen(false);
      }
    };

    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [isMobileChatOpen]);

  const resetTransform = useCallback(() => {
    const el = mobileChatRef.current;
    if (!el) return;
    el.style.transition = "transform 200ms ease";
    el.style.transform = "translateY(0px)";
    setTimeout(() => {
      if (el) el.style.transition = "";
    }, 220);
  }, []);

  const openMobileChat = useCallback(() => {
    const defaultH = Math.round(window.innerHeight * 0.33);
    const h = clampChatHeight(defaultH);
    setMobileChatHeight(h);
    setIsMobileChatOpen(true);
  }, [clampChatHeight]);

  // existing swipe-to-close behavior (for dragging the entire panel down)
  const handleTouchStart = useCallback(
    (e: TouchEvent) => {
      if (!isMobileChatOpen) return;

      const listNode = msgsRef?.current;
      const target = e.target as Node | null;

      if (target instanceof Element && target.closest(".td-chat-handle")) {
        const currentHeight = mobileChatHeight ?? Math.round(window.innerHeight * 0.33);
        startHeightRef.current = clampChatHeight(currentHeight);
        touchStartYRef.current = e.touches[0].clientY;
        touchCurrentYRef.current = e.touches[0].clientY;
        touchFromListRef.current = false;
        return;
      }

      // If the touch starts inside the messages list and the list is scrolled,
      // do not treat it as a close gesture — allow normal scrolling.
      if (listNode && target && listNode.contains(target) && listNode.scrollTop > 0) {
        touchStartYRef.current = null;
        touchCurrentYRef.current = null;
        touchFromListRef.current = false;
        return;
      }

      touchStartYRef.current = e.touches[0].clientY;
      touchCurrentYRef.current = e.touches[0].clientY;
      // mark origin when starting from the list (and at top) so we can allow pull-to-close
      if (listNode && target && listNode.contains(target) && listNode.scrollTop === 0) {
        touchFromListRef.current = true;
      } else {
        touchFromListRef.current = false;
      }
    },
    [isMobileChatOpen, msgsRef, mobileChatHeight, clampChatHeight],
  );

  const handleTouchMove = useCallback(
    (e: TouchEvent) => {
      if (!isMobileChatOpen || touchStartYRef.current == null) return;
      touchCurrentYRef.current = e.touches[0].clientY;
      if (startHeightRef.current != null) {
        const dragDelta = touchCurrentYRef.current - touchStartYRef.current;
        const nextHeight = clampChatHeight(startHeightRef.current - dragDelta);
        setMobileChatHeight(nextHeight);
        e.preventDefault();
        return;
      }
      const delta = Math.max(0, touchCurrentYRef.current - touchStartYRef.current);
      const el = mobileChatRef.current;
      if (el && !startHeightRef.current && touchFromListRef.current) {
        // pull-down-to-close when started from list at scrollTop === 0
        el.style.transform = `translateY(${delta}px)`;
        // prevent the list from scrolling while pulling
        e.preventDefault();
      } else if (el && !startHeightRef.current && !touchFromListRef.current) {
        // allow dragging from other areas (like the panel background)
        el.style.transform = `translateY(${delta}px)`;
      }
    },
    [isMobileChatOpen, clampChatHeight],
  );

  const handleTouchEnd = useCallback(() => {
    if (!isMobileChatOpen) return;
    if (startHeightRef.current != null) {
      startHeightRef.current = null;
      touchStartYRef.current = null;
      touchCurrentYRef.current = null;
      touchFromListRef.current = false;
      return;
    }
    if (touchStartYRef.current == null || touchCurrentYRef.current == null) return;
    const delta = touchCurrentYRef.current - touchStartYRef.current;
    const threshold = 80; // px to trigger close
    if (delta > threshold && touchFromListRef.current) {
      setIsMobileChatOpen(false);
    } else {
      resetTransform();
    }
    touchStartYRef.current = null;
    touchCurrentYRef.current = null;
    touchFromListRef.current = false;
  }, [isMobileChatOpen, resetTransform]);

  useEffect(() => {
    const el = mobileChatRef.current;
    if (!el) return;
    el.addEventListener("touchstart", handleTouchStart);
    el.addEventListener("touchmove", handleTouchMove);
    el.addEventListener("touchend", handleTouchEnd);
    return () => {
      el.removeEventListener("touchstart", handleTouchStart);
      el.removeEventListener("touchmove", handleTouchMove);
      el.removeEventListener("touchend", handleTouchEnd);
    };
  }, [
    isMobileChatOpen,
    handleTouchStart,
    handleTouchMove,
    handleTouchEnd,
    resetTransform,
    mobileChatHeight,
    clampChatHeight,
  ]);

  useEffect(() => {
    // when closing ensure transform is reset
    if (!isMobileChatOpen && mobileChatRef.current) {
      mobileChatRef.current.style.transform = "";
      mobileChatRef.current.style.transition = "";
    }
  }, [isMobileChatOpen]);

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }

    const mediaQuery = window.matchMedia("(min-width: 1025px)");
    const handleDesktopChange = (event: MediaQueryListEvent) => {
      if (event.matches) {
        setIsMobileChatOpen(false);
      }
    };

    mediaQuery.addEventListener("change", handleDesktopChange);

    return () => {
      mediaQuery.removeEventListener("change", handleDesktopChange);
    };
  }, []);

  return (
    <div className="td-app">
      <div className="td-frame">
        <Topbar activeNav={activeNav} setActiveNav={setActiveNav} />

        <div className="td-grid">
          <div className="td-chat-shell td-chat-shell-desktop">
            <ChatPanel
              messages={messages}
              activeStrategiesCount={activeStrategiesCount}
              input={input}
              setInput={setInput}
              send={send}
              msgsRef={msgsRef}
              isBootstrapping={isBootstrapping}
              isSending={isSending}
              lastError={lastError}
              connectionMode={connectionMode}
              showInput={false}
            />
            <div className="td-chat-footer">
              {/* centralized input for desktop chat */}
              <GlobalChatInput />
            </div>
          </div>

          <TradingPanel
            solPrice={solPrice}
            chg={chg}
            activeTab={activeTab}
            setActiveTab={setActiveTab}
            positions={positions}
          />
          <PortfolioPanel
            totalPnl={totalPnl}
            strategies={strategies}
            fund={fund}
            connectionMode={connectionMode}
            lastError={lastError}
          />
        </div>

        {/* moved FAB and backdrop outside td-frame to avoid stacking/containing-block issues */}
      </div>
      {!isDesktop && isMobileChatOpen ? (
        <>
          <button
            type="button"
            className="td-chat-backdrop"
            aria-label="Close chat backdrop"
            onClick={() => setIsMobileChatOpen(false)}
          />

          <div
            ref={mobileChatRef}
            className="td-mobile-chat-sheet"
            style={mobileChatHeight ? { height: `${mobileChatHeight}px` } : undefined}
          >
            <div className="td-chat-handle" aria-label="Resize chat panel" />

            <ChatPanel
              messages={messages}
              activeStrategiesCount={activeStrategiesCount}
              input={input}
              setInput={setInput}
              send={send}
              msgsRef={msgsRef}
              isBootstrapping={isBootstrapping}
              isSending={isSending}
              lastError={lastError}
              connectionMode={connectionMode}
              onClose={() => setIsMobileChatOpen(false)}
              showInput={false}
            />

            <div className="td-chat-footer">
              {/* centralized input for mobile sheet */}
              <GlobalChatInput autoFocus={true} />
            </div>
          </div>
        </>
      ) : null}

      

      {!isDesktop ? (
        <button
          type="button"
          className={`td-chat-fab ${isMobileChatOpen ? "td-chat-fab-hidden" : ""}`}
          onClick={() => openMobileChat()}
          aria-label="Open agent chat"
        >
          AI
        </button>
      ) : null}
    </div>
  );
}

export default function HomePage() {
  return (
    <TradingDashboardProvider>
      <DashboardContent />
    </TradingDashboardProvider>
  );
}
