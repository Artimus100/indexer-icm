"use client";
import { useState } from "react";
import { TradeView } from "../../components/TradeView";
import { Trades } from "../../components/Trades";

export default function CoinsPage() {
  const [selectedMarket, setSelectedMarket] = useState<string>("SOL_USDC");

  return (
    <div className="min-h-screen bg-white text-gray-900">
      <div className="container mx-auto px-4 py-8">
        <div className="mb-8">
          <h1 className="text-3xl font-bold mb-4">Trading Dashboard</h1>
          <div className="flex space-x-4">
            <button
              onClick={() => setSelectedMarket("SOL_USDC")}
              className={`px-4 py-2 rounded-lg transition-colors ${
                selectedMarket === "SOL_USDC"
                  ? "bg-blue-600 text-white"
                  : "bg-gray-200 text-gray-700 hover:bg-gray-300"
              }`}
            >
              SOL/USDC
            </button>
            <button
              onClick={() => setSelectedMarket("BTC_USDC")}
              className={`px-4 py-2 rounded-lg transition-colors ${
                selectedMarket === "BTC_USDC"
                  ? "bg-blue-600 text-white"
                  : "bg-gray-200 text-gray-700 hover:bg-gray-300"
              }`}
            >
              BTC/USDC
            </button>
            <button
              onClick={() => setSelectedMarket("ETH_USDC")}
              className={`px-4 py-2 rounded-lg transition-colors ${
                selectedMarket === "ETH_USDC"
                  ? "bg-blue-600 text-white"
                  : "bg-gray-200 text-gray-700 hover:bg-gray-300"
              }`}
            >
              ETH/USDC
            </button>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Chart Section */}
          <div className="lg:col-span-2">
            <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
              <h2 className="text-xl font-semibold mb-4">Price Chart - {selectedMarket}</h2>
              <TradeView market={selectedMarket} />
            </div>
          </div>

          {/* Trades Section */}
          <div className="lg:col-span-1">
            <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
              <Trades poolId={selectedMarket} />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
