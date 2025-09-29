"use client";

import { useState } from "react";

export type TimeInterval = "1s" | "15s" | "30s" | "1m" | "5m" | "15m" | "30m" | "1h" | "4h" | "6h" | "12h";

interface ChartControlProps {
  selectedInterval: TimeInterval;
  onIntervalChange: (interval: TimeInterval) => void;
}

export function ChartControl({ selectedInterval, onIntervalChange }: ChartControlProps) {
  const intervals: { value: TimeInterval; label: string }[] = [
    { value: "1s", label: "1s" },
    { value: "15s", label: "15s" },
    { value: "30s", label: "30s" },
    { value: "1m", label: "1m" },
    { value: "5m", label: "5m" },
    { value: "15m", label: "15m" },
    { value: "30m", label: "30m" },
    { value: "1h", label: "1h" },
    { value: "4h", label: "4h" },
    { value: "6h", label: "6h" },
    { value: "12h", label: "12h" },
  ];

  return (
    <div className="flex items-center gap-2 p-4 bg-gray-50 border-b border-gray-200">
      <div className="flex items-center gap-2">
        <span className="text-sm font-medium text-gray-700">Timeframe:</span>
        <div className="flex items-center gap-1">
          {intervals.map((interval) => (
            <button
              key={interval.value}
              onClick={() => onIntervalChange(interval.value)}
              className={`px-3 py-1 text-sm rounded-md transition-colors ${
                selectedInterval === interval.value
                  ? "bg-blue-600 text-white"
                  : "bg-gray-200 text-gray-700 hover:bg-gray-300"
              }`}
            >
              {interval.label}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
