"use client";

import { useEffect, useId, useMemo, useState } from "react";

const BASE_POINTS = [
  78, 76, 74, 75, 73, 72, 74, 70, 69, 67, 66, 68, 65, 63, 62, 60, 59, 57, 56,
  55, 54, 53, 55, 52, 51, 50, 49, 48, 47, 46,
];

function clampPoint(value: number) {
  return Math.max(6, Math.min(94, value));
}

export function MiniChart({ price }: { price: number }) {
  const [points, setPoints] = useState(BASE_POINTS);
  const gradientId = useId().replace(/:/g, "");
  const w = 500;
  const h = 100;

  useEffect(() => {
    const id = window.setInterval(() => {
      setPoints((previous) => {
        const lastPoint = previous[previous.length - 1];
        const delta = (Math.random() - 0.5) * 7;
        const nextPoint = Number(clampPoint(lastPoint + delta).toFixed(1));
        return [...previous.slice(1), nextPoint];
      });
    }, 850);

    return () => window.clearInterval(id);
  }, []);

  const { pathD, fillD, xs } = useMemo(() => {
    const zoomedOutPoints = points.map((point) =>
      Number((50 + (point - 50) * 0.72).toFixed(1)),
    );
    const xValues = zoomedOutPoints.map(
      (_, i) => (i / (zoomedOutPoints.length - 1)) * w,
    );
    const linePath = zoomedOutPoints
      .map((y, i) => `${i === 0 ? "M" : "L"}${xValues[i].toFixed(1)},${y}`)
      .join(" ");
    const areaPath = `${linePath} L${w},${h} L0,${h} Z`;

    return {
      pathD: linePath,
      fillD: areaPath,
      xs: xValues,
    };
  }, [points]);

  const lastX = xs[xs.length - 1];
  const lastY = Number((50 + (points[points.length - 1] - 50) * 0.72).toFixed(1));

  return (
    <div className="td-mini-chart">
      <svg
        width="100%"
        height="100"
        viewBox={`0 0 ${w} ${h}`}
        preserveAspectRatio="none"
      >
        <defs>
          <linearGradient id={`${gradientId}-area`} x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="rgba(124,58,237,0.45)" />
            <stop offset="100%" stopColor="rgba(124,58,237,0)" />
          </linearGradient>
        </defs>
        <path d={fillD} className="td-mini-chart-area" fill={`url(#${gradientId}-area)`} />
        <path
          d={pathD}
          className="td-mini-chart-line"
          fill="none"
          stroke="#a78bfa"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
        <circle className="td-mini-chart-dot" cx={lastX} cy={lastY} r="3.2" />
      </svg>
      <div className="td-mini-chart-label">
        Entry $128.90 | Mark ${price.toFixed(2)}
      </div>
    </div>
  );
}
