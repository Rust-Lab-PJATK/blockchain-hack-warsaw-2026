export function MiniChart({ price }: { price: number }) {
  const points = [
    75, 70, 80, 65, 50, 58, 42, 28, 48, 38, 28, 18, 14, 10, 22, 16, 12, 8, 6,
  ];
  const w = 500;
  const h = 100;
  const xs = points.map((_, i) => (i / (points.length - 1)) * w);
  const pathD = points
    .map((y, i) => `${i === 0 ? "M" : "L"}${xs[i].toFixed(1)},${y}`)
    .join(" ");
  const fillD = `${pathD} L${w},${h} L0,${h} Z`;

  return (
    <div className="td-mini-chart">
      <svg
        width="100%"
        height="100"
        viewBox={`0 0 ${w} ${h}`}
        preserveAspectRatio="none"
      >
        <defs>
          <linearGradient id="solArea" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="rgba(124,58,237,0.45)" />
            <stop offset="100%" stopColor="rgba(124,58,237,0)" />
          </linearGradient>
        </defs>
        <path d={fillD} fill="url(#solArea)" />
        <path
          d={pathD}
          fill="none"
          stroke="#a78bfa"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
      <div className="td-mini-chart-label">
        Entry $128.90 | Mark ${price.toFixed(2)}
      </div>
    </div>
  );
}
