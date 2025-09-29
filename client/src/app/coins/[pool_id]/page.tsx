"use client";
import { useState, use, useEffect } from "react";
import { TradeView } from "../../../components/TradeView";
import { Trades } from "../../../components/Trades";
import { getCoinByPoolId } from "../../../utils/httpClient";
import { Coin } from "../../../utils/types";
import Link from "next/link";

export default function CoinPage({ params }: { params: Promise<{ pool_id: string }> }) {
  const { pool_id: poolId } = use(params);
  const [coin, setCoin] = useState<Coin | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchCoinData = async () => {
      try {
        setLoading(true);
        console.log("Coin page: Fetching coin data for pool:", poolId);
        const coinData = await getCoinByPoolId(poolId);
        console.log("Coin page: Fetched coin data:", coinData);
        setCoin(coinData);
      } catch (error) {
        console.error("Coin page: Error fetching coin data:", error);
      } finally {
        setLoading(false);
      }
    };

    fetchCoinData();
  }, [poolId]);

  return (
    <div className="min-h-screen bg-white text-gray-900">
      <div className="container mx-auto px-4 py-8">
        <div className="mb-8">
          <div className="flex items-center space-x-4 mb-4">
            <Link 
              href="/"
              className="text-blue-600 hover:text-blue-700 transition-colors"
            >
              ← Back to Home
            </Link>
            <Link 
              href="/coins"
              className="text-blue-600 hover:text-blue-700 transition-colors"
            >
              ← Back to Trading Dashboard
            </Link>
          </div>
          {loading ? (
            <div className="animate-pulse">
              <div className="h-8 bg-gray-300 rounded w-1/3 mb-2"></div>
              <div className="h-4 bg-gray-300 rounded w-1/2"></div>
            </div>
          ) : coin ? (
            <div>
              <div className="flex items-center space-x-4 mb-4">
                {coin.image_uri && (
                  <img 
                    src={coin.image_uri} 
                    alt={coin.name}
                    className="w-12 h-12 rounded-full"
                    onError={(e) => {
                      e.currentTarget.style.display = 'none';
                    }}
                  />
                )}
                <div>
                  <h1 className="text-3xl font-bold">{coin.name}</h1>
                  <p className="text-gray-600">{coin.symbol} • {coin.mint}</p>
                </div>
              </div>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
                <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
                  <p className="text-sm text-gray-600">Base Mint</p>
                  <p className="font-mono text-sm break-all">{coin.mint}</p>
                </div>
                <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
                  <p className="text-sm text-gray-600">Pool Address</p>
                  <p className="font-mono text-sm break-all">{coin.pool_address}</p>
                </div>
                <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
                  <p className="text-sm text-gray-600">Creator</p>
                  <p className="font-mono text-sm break-all">{coin.creator}</p>
                </div>
              </div>
              {coin.description && (
                <p className="text-gray-700 mb-4">{coin.description}</p>
              )}
            </div>
          ) : (
            <div>
              <h1 className="text-3xl font-bold mb-4">Pool: {poolId}</h1>
              <p className="text-gray-600">Coin data not found</p>
            </div>
          )}
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Chart Section */}
          <div className="lg:col-span-2">
            <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
              <h2 className="text-xl font-semibold mb-4">Price Chart - {poolId}</h2>
              <TradeView market={poolId} />
            </div>
          </div>

          {/* Trades Section */}
          <div className="lg:col-span-1">
            <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
              <Trades poolId={poolId} />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
