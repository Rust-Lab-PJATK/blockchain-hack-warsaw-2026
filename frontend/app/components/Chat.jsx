"use client";

// TODO To okienko chatu jest najpewniej to edycji po stronie frondendu, trzeba sprawdzić

import { useState, useRef, useEffect } from "react";
import styles from "./Chat.module.css";

function LoadingDots() {
  return (
    <div className={styles.loadingDots}>
      <span className={styles.loadingDot} />
      <span className={styles.loadingDot} />
      <span className={styles.loadingDot} />
    </div>
  );
}

function MessageBubble({ message }) {
  const isUser = message.role === "you";

  return (
    <div className={styles.bubble}>
      <div className={isUser ? styles.bubbleLabelUser : styles.bubbleLabelAgent}>
        {isUser ? "YOU" : "AGENT"}
      </div>
      <div className={isUser ? styles.bubbleCardUser : styles.bubbleCardAgent}>
        {message.content}
      </div>
    </div>
  );
}

export default function Chat() {
  const [messages, setMessages] = useState([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const scrollRef = useRef(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages, loading]);

  const sendMessage = async () => {
    const text = input.trim();
    if (!text || loading) return;

    const userMsg = { role: "you", content: text };
    const updatedMessages = [...messages, userMsg];
    setMessages(updatedMessages);
    setInput("");
    setLoading(true);

    try {
      const apiMessages = updatedMessages.map((m) => ({
        role: m.role === "you" ? "user" : "assistant",
        content: m.content,
      }));
      const res = await fetch(`${process.env.NEXT_PUBLIC_API_URL}/api/chat`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ messages: apiMessages }),
      });

      if (!res.ok) throw new Error(`Request failed (${res.status})`);

      const data = await res.json();
      const agentContent = data.content || "";

      setMessages((prev) => [
        ...prev,
        { role: "agent", content: agentContent },
      ]);
    } catch (err) {
      setMessages((prev) => [
        ...prev,
        {
          role: "agent",
          content: `[ERROR] Connection lost. ${err.message}. Retrying is recommended.`,
        },
      ]);
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (e) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  return (
    <div className={styles.panel}>
      <div className={styles.header}>
        <div className={styles.headerLabel}>
          <span className={styles.statusDot} />
          AI AGENT
        </div>
        <div className={styles.headerStatus}>Monitoring 3 strategies</div>
      </div>

      <div ref={scrollRef} className={styles.messages}>
        {messages.map((msg, i) => (
          <MessageBubble key={i} message={msg} />
        ))}
        {loading && (
          <div className={styles.bubble}>
            <div className={styles.bubbleLabelAgent}>AGENT</div>
            <div className={styles.bubbleCardAgent}>
              <LoadingDots />
            </div>
          </div>
        )}
      </div>

      <div className={styles.inputWrapper}>
        <input
          className={styles.input}
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Message agent..."
        />
      </div>
    </div>
  );
}
