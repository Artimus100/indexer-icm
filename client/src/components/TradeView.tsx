"use client"

import { useEffect, useRef, useState, useCallback } from "react";
import { ChartManager } from "../utils/ChartManager";
import { getKlines } from "../utils/httpClient";
import { KLine } from "../utils/types";
import { ChartControl, TimeInterval } from "./ChartControl";

export function TradeView({
  market,
}: {
  market: string;
}) {
  const chartRef = useRef<HTMLDivElement>(null);
  const chartManagerRef = useRef<ChartManager | null>(null);
  const [selectedInterval, setSelectedInterval] = useState<TimeInterval>("1h");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Calculate time range based on interval
  const getTimeRange = useCallback((interval: TimeInterval) => {
    const now = Date.now();
    const oneHour = 1000 * 60 * 60;
    const oneDay = oneHour * 24;
    const oneWeek = oneDay * 7;
    const oneMonth = oneDay * 30;
    const oneYear = oneDay * 365;

    switch (interval) {
      case "1s":
      case "15s":
      case "30s":
        return { start: now - oneHour, end: now }; // 1 hour for seconds
      case "1m":
      case "5m":
      case "15m":
      case "30m":
        return { start: now - oneDay, end: now }; // 1 day for minutes
      case "1h":
      case "4h":
      case "6h":
      case "12h":
        return { start: now - oneWeek, end: now }; // 1 week for hours
      default:
        return { start: now - oneWeek, end: now };
    }
  }, []);

  const init = useCallback(async () => {
    if (!market || !chartRef.current) return;
    
    setLoading(true);
    setError(null);
    
    try {
      const { start, end } = getTimeRange(selectedInterval);
      const klineData = await getKlines(
        market, 
        selectedInterval, 
        Math.floor(start / 1000), 
        Math.floor(end / 1000)
      );
      
      console.log("Fetched klines:", klineData.length, "for interval:", selectedInterval);
      console.log("Sample kline data:", klineData.slice(0, 2));

      // Destroy existing chart
      if (chartManagerRef.current) {
        chartManagerRef.current.destroy();
        chartManagerRef.current = null;
      }

      // Process and sort data
      const processedData = klineData
        .map((x) => {
          const close = parseFloat(x.close) || 0;
          const high = parseFloat(x.high) || 0;
          const low = parseFloat(x.low) || 0;
          const open = parseFloat(x.open) || 0;
          
          // Determine the best scale factor based on the values
          // If values are very small (like 0.000000419), scale them up
          let scale = 1;
          if (close > 0 && close < 0.001) {
            scale = 1000000; // 1e6 for values like 0.000000419
          } else if (close > 0 && close < 0.01) {
            scale = 100000; // 1e5 for values like 0.00000419
          } else if (close > 0 && close < 0.1) {
            scale = 10000; // 1e4 for values like 0.0000419
          }
          
          return {
            close: close * scale,
            high: high * scale,
            low: low * scale,
            open: open * scale,
            volume: parseFloat(x.volume) || 0,
            timestamp: parseInt(x.end) * 1000, // Convert to milliseconds
          };
        })
        .sort((a, b) => a.timestamp - b.timestamp);

      console.log("Processed data sample:", processedData.slice(0, 2));

      // Create new chart
      const chartManager = new ChartManager(
        chartRef.current,
        processedData,
        {
          background: "#ffffff",
          color: "#171717",
        }
      );
      
      chartManagerRef.current = chartManager;
    } catch (e) {
      console.error("Error fetching klines:", e);
      setError(e instanceof Error ? e.message : "Failed to load chart data");
    } finally {
      setLoading(false);
    }
  }, [market, selectedInterval, getTimeRange]);

  useEffect(() => {
    init();
  }, [init]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (chartManagerRef.current) {
        chartManagerRef.current.destroy();
        chartManagerRef.current = null;
      }
    };
  }, []);

  return (
    <>
      <ChartControl 
        selectedInterval={selectedInterval} 
        onIntervalChange={setSelectedInterval} 
      />
      <div className="relative">
        {loading && (
          <div className="absolute inset-0 flex items-center justify-center bg-white bg-opacity-75 z-10">
            <div className="flex items-center gap-2">
              <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"></div>
              <span className="text-sm text-gray-600">Loading chart data...</span>
            </div>
          </div>
        )}
        {error && (
          <div className="absolute inset-0 flex items-center justify-center bg-red-50 z-10">
            <div className="text-center">
              <div className="text-red-600 text-sm mb-2">Failed to load chart</div>
              <div className="text-red-500 text-xs">{error}</div>
              <button 
                onClick={init}
                className="mt-2 px-3 py-1 bg-red-600 text-white text-xs rounded hover:bg-red-700"
              >
                Retry
              </button>
            </div>
          </div>
        )}
        <div 
          ref={chartRef} 
          style={{ 
            height: "520px", 
            width: "100%", 
            marginTop: 4,
            position: "relative",
            overflow: "visible"
          }}
          className={loading || error ? "opacity-50" : ""}
        />
      </div>
    </>
  );
}