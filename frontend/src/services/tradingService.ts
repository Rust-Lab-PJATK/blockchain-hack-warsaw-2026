import { MOCK_RESPONSES } from "@/services/tradingData";
import { Position } from "@/utils/types";

export function getRandomMockResponse() {
  const idx = Math.floor(Math.random() * MOCK_RESPONSES.length);
  return MOCK_RESPONSES[idx];
}

export function getNextMarketTick(
  previousPrice: number,
  previousFunding: number,
) {
  const priceDelta = (Math.random() - 0.5) * 1.8;
  const nextPrice = Number((previousPrice + priceDelta).toFixed(2));

  const pctChange = Number(
    (((nextPrice - previousPrice) / previousPrice) * 100).toFixed(2),
  );

  const fundingDelta = (Math.random() - 0.5) * 0.01;
  const nextFunding = Number((previousFunding + fundingDelta).toFixed(3));

  return {
    nextPrice,
    pctChange,
    nextFunding,
  };
}

export function updatePositionMarks(
  positions: Position[],
  nextPrice: number,
): Position[] {
  return positions.map((position) => {
    const volatility = 1 + (Math.random() - 0.5) * 0.004;
    const mark = Number((nextPrice * volatility).toFixed(2));
    const entry = position.entry;
    const isLong = position.side === "long";
    const rawPnl = isLong ? mark - entry : entry - mark;
    const pnl = Number((rawPnl * 10).toFixed(2));

    return {
      ...position,
      mark,
      pnl,
    };
  });
}
