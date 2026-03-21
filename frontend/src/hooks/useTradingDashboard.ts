"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { createTradingGateway } from "@/services/tradingGateway";
import { Message, Position, Strategy } from "@/utils/types";

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
  const [strategies, setStrategies] = useState<Strategy[]>([]);
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

    async function bootstrap() {
      try {
        const data = await gatewayRef.current.getInitialDashboardState();

        if (!active) {
          return;
        }

        setSolPrice(data.solPrice);
        setChg(data.chg);
        setFund(data.fund);
        setPositions(data.positions);
        setStrategies(data.strategies);
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
    };
  }, []);

  useEffect(() => {
    if (isBootstrapping) {
      return;
    }

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
    }, 2000);

    return () => window.clearInterval(id);
  }, [isBootstrapping]);

  useEffect(() => {
    const node = msgsRef.current;
    if (!node) {
      return;
    }
    node.scrollTop = node.scrollHeight;
  }, [messages]);

  const send = useCallback(async () => {
    const prompt = input.trim();
    if (!prompt || isSending) {
      return;
    }

    setMessages((prev) => [...prev, { role: "user", text: prompt }]);
    setInput("");

    setIsSending(true);

    try {
      const response = await gatewayRef.current.sendPrompt(prompt);
      setMessages((prev) => [
        ...prev,
        {
          role: "agent",
          text: response.text,
          parsed: response.parsed,
        },
      ]);
      setLastError(null);
    } catch {
      setMessages((prev) => [
        ...prev,
        {
          role: "agent",
          text: "I could not reach the strategy parser service. Please retry.",
        },
      ]);
      setLastError("Message service unavailable.");
    } finally {
      setIsSending(false);
    }
  }, [input, isSending]);

  const totalPnl = useMemo(
    () => positions.reduce((sum, position) => sum + position.pnl, 0),
    [positions],
  );

  const activeStrategiesCount = useMemo(
    () => strategies.filter((strategy) => strategy.status !== "waiting").length,
    [strategies],
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
    strategies,
    totalPnl,
    activeStrategiesCount,
    input,
    setInput,
    messages,
    send,
    msgsRef,
    isBootstrapping,
    isSending,
    lastError,
    connectionMode: gatewayRef.current.mode,
  };
}
