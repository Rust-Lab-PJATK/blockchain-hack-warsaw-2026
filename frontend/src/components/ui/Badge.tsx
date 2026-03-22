import { STATUS_COLORS } from "@/utils/constants";
import { CSSProperties } from "react";

export function Badge({ status }: { status: keyof typeof STATUS_COLORS }) {
  const s = STATUS_COLORS[status] ?? STATUS_COLORS.waiting;
  const badgeStyle = {
    background: s.bg,
    color: s.color,
  } as CSSProperties;

  return (
    <span className="td-badge" style={badgeStyle}>
      {s.label}
    </span>
  );
}
