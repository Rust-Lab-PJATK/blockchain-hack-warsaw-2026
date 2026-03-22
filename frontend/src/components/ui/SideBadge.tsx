import { CSSProperties } from "react";

export function SideBadge({ side }: { side: "long" | "short" }) {
  const isLong = side === "long";
  const sideStyle = {
    background: isLong
      ? "rgba(16, 185, 129, 0.14)"
      : "rgba(248, 113, 113, 0.14)",
    color: isLong ? "#10b981" : "#f87171",
  } as CSSProperties;

  return (
    <span className="td-side-badge" style={sideStyle}>
      {side.toUpperCase()}
    </span>
  );
}
