"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { createTradingGateway } from "@/services/tradingGateway";
import { Message, Position } from "@/utils/types";

const MARKET_TICK_INTERVAL_MS = 2000;

function inferConfirmableAction(prompt: string) {
  const normalizedPrompt = prompt.trim();
  if (!normalizedPrompt) {
    return null;
  }

  // Safety gate: these commands imply account-affecting changes and should
  // only run after explicit user confirmation.
  const requiresConfirmationPattern =
    /(buy|sell|open|close|long|short|stop|take profit|tp|sl|leverage|size)/i;

  if (!requiresConfirmationPattern.test(normalizedPrompt)) {
    return null;
  }

  return {
    label: "Proposed trading action",
    summary: normalizedPrompt,
  };
}

function createActionId() {
  return `action-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

function getPendingConfirmationSummary(messages: Message[]) {
  const pendingAction = messages.find(
    (message) => message.confirmationAction?.status === "pending",
  )?.confirmationAction;

  return pendingAction?.summary ?? null;
}

export function useTradingDashboard() {
  const gatewayRef = useRef(createTradingGateway());
  const isTickInFlightRef = useRef(false);
  const priceRef = useRef(131.42);
  const fundRef = useRef(0.031);
  const positionsRef = useRef<Position[]>([]);

  const [activeNav, setActiveNav] = useState("Trade");
  const [activeTab, setActiveTab] = useState("15m");
  const [solPrice, setSolPrice] = useState(131.42);
  const [chg, setChg] = useState(1.96);
  const [fund, setFund] = useState(0.031);
  const [positions, setPositions] = useState<Position[]>([]);
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<Message[]>([]);
  const [isBootstrapping, setIsBootstrapping] = useState(true);
  const [isSending, setIsSending] = useState(false);
  const [lastError, setLastError] = useState<string | null>(null);

  const msgsRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    priceRef.current = solPrice;
  }, [solPrice]);

  useEffect(() => {
    fundRef.current = fund;
  }, [fund]);

  useEffect(() => {
    positionsRef.current = positions;
  }, [positions]);

  useEffect(() => {
    let active = true;
    const bootstrapAbortController = new AbortController();

    async function bootstrap() {
      try {
        const data = await gatewayRef.current.getInitialDashboardState({
          signal: bootstrapAbortController.signal,
        });

        if (!active) {
          return;
        }

        setSolPrice(data.solPrice);
        setChg(data.chg);
        setFund(data.fund);
        setPositions(data.positions);
        setMessages([data.welcomeMessage]);
        setLastError(null);
      } catch {
        if (!active) {
          return;
        }

        setLastError("Unable to load dashboard data.");
      } finally {
        if (active) {
          setIsBootstrapping(false);
        }
      }
    }

    bootstrap();

    return () => {
      active = false;
      bootstrapAbortController.abort();
    };
  }, []);

  useEffect(() => {
    if (isBootstrapping) {
      return;
    }

    // Backend integration point:
    // Current implementation polls every MARKET_TICK_INTERVAL_MS.
    // When backend exposes push transport (SSE/WebSocket), replace this block
    // with subscription setup and dispatch incoming ticks directly into state.
    const id = window.setInterval(async () => {
      if (isTickInFlightRef.current) {
        return;
      }

      isTickInFlightRef.current = true;

      try {
        const nextTick = await gatewayRef.current.getMarketTick(
          priceRef.current,
          fundRef.current,
          positionsRef.current,
        );

        setSolPrice(nextTick.nextPrice);
        setChg(nextTick.pctChange);
        setFund(nextTick.nextFunding);
        setPositions(nextTick.positions);
        setLastError(null);
      } catch {
        setLastError("Market feed temporarily unavailable.");
      } finally {
        isTickInFlightRef.current = false;
      }
    }, MARKET_TICK_INTERVAL_MS);

    return () => window.clearInterval(id);
  }, [isBootstrapping]);

  useEffect(() => {
    const node = msgsRef.current;
    if (!node) {
      return;
    }
    node.scrollTop = node.scrollHeight;
  }, [messages]);

  const pendingActionSummary = useMemo(
    () => getPendingConfirmationSummary(messages),
    [messages],
  );
  const hasPendingConfirmation = pendingActionSummary !== null;

  const send = useCallback(async () => {
    const prompt = input.trim();
    if (!prompt || isSending) {
      return;
    }

    if (hasPendingConfirmation) {
      // Safety lock: while there is a pending trade action, block new commands
      // until user explicitly confirms or cancels that action.
      setLastError("Confirm or cancel the pending action before sending a new command.");
      return;
    }

    setMessages((prev) => [...prev, { role: "user", text: prompt }]);
    setInput("");

    setIsSending(true);

    try {
      const start = Date.now();
      // Backend integration point:
      // For streamed responses, append partial chunks to the last agent message
      // and finalize once server sends completion event.
      const response = await gatewayRef.current.sendPrompt(prompt);

      // For mock gateway keep typing indicator for ~5s total to simulate thinking
      if (gatewayRef.current.mode === "mock") {
        const elapsed = Date.now() - start;
        const remaining = 5000 - elapsed;
        if (remaining > 0) {
          await new Promise((r) => window.setTimeout(r, remaining));
        }
      }

      // Confirmation integration point:
      // Backend can return response.proposedAction. Until then, infer candidate
      // actions from user prompt and gate them behind explicit confirmation.
      const proposedAction =
        response.proposedAction ?? inferConfirmableAction(prompt);

      const agentMessage: Message = {
        role: "agent",
        text: proposedAction
          ? `${response.text}\n\nThis action requires your confirmation before execution.`
          : response.text,
        confirmationAction: proposedAction
          ? {
              id: createActionId(),
              label: proposedAction.label,
              summary: proposedAction.summary,
              status: "pending",
            }
          : undefined,
      };

      setMessages((prev) => [...prev, agentMessage]);
      setLastError(null);
    } catch {
      setMessages((prev) => [
        ...prev,
        {
          role: "agent",
          text: "I could not reach the agent service. Please retry.",
        },
      ]);
      setLastError("Message service unavailable.");
    } finally {
      setIsSending(false);
    }
  }, [hasPendingConfirmation, input, isSending]);

  const confirmAction = useCallback((actionId: string) => {
    // Execution gate:
    // This callback is the only place where a pending chatbot action should be
    // promoted to executable. Hook backend execution call here.
    setMessages((prev) => {
      const nextMessages = prev.map((message) => {
        if (message.confirmationAction?.id !== actionId) {
          return message;
        }

        return {
          ...message,
          confirmationAction: {
            ...message.confirmationAction,
            status: "confirmed",
          },
        };
      });

      const confirmedAction = nextMessages.find(
        (message) => message.confirmationAction?.id === actionId,
      )?.confirmationAction;

      if (!confirmedAction) {
        return nextMessages;
      }

      return [
        ...nextMessages,
        {
          role: "agent",
          text: `Confirmed. Queued action: ${confirmedAction.summary}`,
        },
      ];
    });
  }, []);

  const cancelAction = useCallback((actionId: string) => {
    setMessages((prev) => {
      const nextMessages = prev.map((message) => {
        if (message.confirmationAction?.id !== actionId) {
          return message;
        }

        return {
          ...message,
          confirmationAction: {
            ...message.confirmationAction,
            status: "cancelled",
          },
        };
      });

      return [
        ...nextMessages,
        {
          role: "agent",
          text: "Action cancelled. No trade was executed.",
        },
      ];
    });
  }, []);

  const totalPnl = useMemo(
    () => positions.reduce((sum, position) => sum + position.pnl, 0),
    [positions],
  );

  return {
    activeNav,
    setActiveNav,
    activeTab,
    setActiveTab,
    solPrice,
    chg,
    fund,
    positions,
    totalPnl,
    input,
    setInput,
    messages,
    send,
    confirmAction,
    cancelAction,
    msgsRef,
    isBootstrapping,
    isSending,
    lastError,
    hasPendingConfirmation,
    pendingActionSummary,
    connectionMode: gatewayRef.current.mode,
  };
}
