"use client";
import { useEffect, useState } from "react";
import { Trade, Coin } from "../utils/types";
import { SignalingManager } from "../utils/SignalingManager";
import { getRecentTrades, getCoinByPoolId } from "../utils/httpClient";

export function Trades({poolId}: {poolId: string}) {
    const [trades, setTrades] = useState<Trade[]>([]);
    const [coin, setCoin] = useState<Coin | null>(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        console.log(`Trades component: Setting up subscription and fetching initial data for pool: ${poolId}`);
        
        // Fetch initial data
        const fetchInitialData = async () => {
            try {
                setLoading(true);
                console.log(`Trades component: Starting fetchInitialData for pool ${poolId}`);
                
                // Fetch coin data using pool address
                console.log(`Trades component: Fetching coin data for pool ${poolId} from API`);
                let coinData = await getCoinByPoolId(poolId);
                console.log(`Trades component: Pool address lookup result:`, coinData);
                
                setCoin(coinData);
                
                // Fetch trades using the pool address (mint parameter)
                console.log(`Trades component: About to call getRecentTrades with poolId: ${poolId}`);
                const recentTrades = await getRecentTrades(poolId, 50);
                console.log(`Trades component: getRecentTrades returned:`, recentTrades);
                console.log(`Trades component: Fetched trades count:`, recentTrades.length);
                setTrades(recentTrades);
            } catch (error) {
                console.error(`Trades component: Error fetching initial data for pool ${poolId}:`, error);
                setTrades([]);
            } finally {
                setLoading(false);
            }
        };

        fetchInitialData();
        
        const handleNewTrade = (data: any) => {
            console.log(`Received trade data from websocket for pool ${poolId}:`, data);
            console.log(`Data type:`, typeof data);
            console.log(`Data keys:`, Object.keys(data || {}));
            console.log(`Full data structure:`, JSON.stringify(data, null, 2));
            
            // The data should have token_id field from the server
            if (data && data.token_id === poolId) {
                console.log(`Processing trade for our pool: ${poolId}`);
                
                // Transform the server TradeData to match our Trade interface
                const transformedTrade: Trade = {
                    id: Date.now(), // Generate unique ID since server doesn't provide one
                    pool_id: data.token_id || poolId,
                    timestamp: data.time ? new Date(data.time).toISOString() : new Date().toISOString(),
                    price: data.price_at_trade ? String(data.price_at_trade) : '0',
                    quantity: data.amount_out ? String(data.amount_out) : '0', // amount_out is tokens received
                    volume: data.amount_in ? String(data.amount_in) : '0' // amount_in is SOL spent
                };
                
                console.log(`Transformed trade data:`, transformedTrade);
                
                setTrades(prevTrades => {
                    console.log(`Previous trades count:`, prevTrades.length);
                    // Check if this trade already exists to prevent duplicates
                    const tradeExists = prevTrades.some(existingTrade => 
                        existingTrade.pool_id === transformedTrade.pool_id &&
                        existingTrade.timestamp === transformedTrade.timestamp &&
                        existingTrade.price === transformedTrade.price &&
                        existingTrade.quantity === transformedTrade.quantity
                    );
                    
                    if (tradeExists) {
                        console.log(`Trade already exists, skipping duplicate:`, transformedTrade);
                        return prevTrades;
                    }
                    
                    // Add new trade to the beginning and keep last 50 trades
                    const updatedTrades = [transformedTrade, ...prevTrades].slice(0, 50);
                    console.log(`Updated trades count:`, updatedTrades.length);
                    return updatedTrades;
                });
            } else {
                console.warn(`Trade data not for our pool. Expected: ${poolId}, Got: ${data?.token_id}`);
                console.warn(`Available fields in data:`, Object.keys(data || {}));
            }
        };
        
        // Clean up any existing subscription first
        console.log(`Trades component: Cleaning up any existing subscription for pool: ${poolId}`);
        SignalingManager.getInstance().deRegisterCallback("TRADE", `TRADES-${poolId}`);
        SignalingManager.getInstance().sendMessage({
            "type": "UNSUBSCRIBE",
            "room": `trades@${poolId}`,
            "data": null
        });
        
        // Small delay to ensure cleanup is processed
        setTimeout(() => {
            // Register callback
            console.log(`Registering TRADE callback for pool: ${poolId}`);
            SignalingManager.getInstance().registerCallback("TRADE", handleNewTrade, `TRADES-${poolId}`);
            
            // Subscribe to websocket
            console.log(`Trades component: Sending subscribe message for room: trades@${poolId}`);
            const subscribeMessage = {
                "type": "SUBSCRIBE",
                "room": `trades@${poolId}`,
                "data": null
            };
            console.log(`Trades component: Subscribe message:`, subscribeMessage);
            SignalingManager.getInstance().sendMessage(subscribeMessage);
            
            // Add a test message to verify subscription
            console.log(`Trades component: Successfully subscribed to trades@${poolId}. Waiting for trade data...`);
        }, 100);

        return () => {
            console.log(`Trades component: Cleaning up subscription for pool: ${poolId}`);
            SignalingManager.getInstance().deRegisterCallback("TRADE", `TRADES-${poolId}`);
            SignalingManager.getInstance().sendMessage({
                "type": "UNSUBSCRIBE",
                "room": `trades@${poolId}`,
                "data": null
            });
        }
    }, [poolId]);

    const formatTime = (timestamp: string | undefined) => {
        if (!timestamp) {
            return 'Unknown';
        }
        try {
            // Handle both millisecond and second timestamps
            const timestampNum = parseInt(timestamp);
            const date = new Date(timestampNum > 1e10 ? timestampNum : timestampNum * 1000);
            return date.toLocaleTimeString();
        } catch (error) {
            console.error('Error formatting timestamp:', error);
            return 'Invalid time';
        }
    };

    const formatAmount = (amount: string | undefined) => {
        if (!amount || amount === '0' || amount === '') {
            return '0';
        }
        try {
            const num = parseFloat(amount);
            if (isNaN(num)) {
                return '0';
            }
            
            // Convert from raw units (9 decimals) to actual units
            // Both SOL and token have 9 decimals, so we divide by 10^9
            const actualAmount = num / Math.pow(10, 9);
            
            // Format with appropriate decimal places
            if (actualAmount >= 1000000) {
                return (actualAmount / 1000000).toFixed(2) + 'M';
            } else if (actualAmount >= 1000) {
                return (actualAmount / 1000).toFixed(2) + 'K';
            } else if (actualAmount < 0.01) {
                return actualAmount.toExponential(2);
            } else {
                return actualAmount.toLocaleString(undefined, { 
                    minimumFractionDigits: 0, 
                    maximumFractionDigits: 6 
                });
            }
        } catch (error) {
            console.error('Error formatting amount:', error);
            return '0';
        }
    };

    const formatPrice = (price: string | undefined) => {
        if (!price || price === '0' || price === '') {
            return '0.00';
        }
        try {
            const num = parseFloat(price);
            if (isNaN(num) || num <= 0) {
                return '0.00';
            }
            
            // Price is typically already in the correct units (SOL per token)
            // But if it's in raw units, we might need to adjust
            // For now, assuming price is already in correct units
            const actualPrice = num;
            
            // Format price with appropriate decimal places
            if (actualPrice >= 1) {
                return actualPrice.toFixed(2);
            } else if (actualPrice >= 0.01) {
                return actualPrice.toFixed(4);
            } else if (actualPrice >= 0.0001) {
                return actualPrice.toFixed(6);
            } else {
                return actualPrice.toExponential(2);
            }
        } catch (error) {
            console.error('Error formatting price:', error);
            return '0.00';
        }
    };

    return (
        <div className="w-full">
            {loading ? (
                <div className="animate-pulse">
                    <div className="h-6 bg-gray-300 rounded w-1/3 mb-4"></div>
                    <div className="space-y-2">
                        {[...Array(3)].map((_, i) => (
                            <div key={i} className="h-16 bg-gray-300 rounded"></div>
                        ))}
                    </div>
                </div>
            ) : (
                <>
                    {/* Coin Information Header */}
                    <div className="mb-6 p-4 bg-blue-600 rounded-lg text-white">
                        {coin ? (
                            <>
                                <div className="flex items-center space-x-3 mb-3">
                                    {coin.image_uri && (
                                        <img 
                                            src={coin.image_uri} 
                                            alt={coin.name}
                                            className="w-8 h-8 rounded-full"
                                            onError={(e) => {
                                                e.currentTarget.style.display = 'none';
                                            }}
                                        />
                                    )}
                                    <div>
                                        <h3 className="font-bold text-lg">{coin.name}</h3>
                                        <p className="text-sm text-blue-100">{coin.symbol}</p>
                                    </div>
                                </div>
                                
                                <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-sm">
                                    <div>
                                        <p className="text-blue-100">Base Mint:</p>
                                        <p className="font-mono text-xs break-all">{coin.mint}</p>
                                    </div>
                                    <div>
                                        <p className="text-blue-100">Pool Address:</p>
                                        <p className="font-mono text-xs break-all">{coin.pool_address}</p>
                                    </div>
                                    <div>
                                        <p className="text-blue-100">Creator:</p>
                                        <p className="font-mono text-xs break-all">{coin.creator}</p>
                                    </div>
                                    <div>
                                        <p className="text-blue-100">Market Cap:</p>
                                        <p className="font-medium">
                                            {coin.usd_market_cap ? `$${coin.usd_market_cap.toLocaleString()}` : 'N/A'}
                                        </p>
                                    </div>
                                    <div>
                                        <p className="text-blue-100">Status:</p>
                                        <p className={`font-medium ${coin.complete ? 'text-green-400' : 'text-yellow-400'}`}>
                                            {coin.complete ? 'Complete' : 'In Progress'}
                                        </p>
                                    </div>
                                    <div>
                                        <p className="text-blue-100">Program:</p>
                                        <p className="font-medium">{coin.program}</p>
                                    </div>
                                </div>
                                
                                {coin.description && (
                                    <div className="mt-3">
                                        <p className="text-blue-100 text-sm">Description:</p>
                                        <p className="text-sm text-gray-700">{coin.description}</p>
                                    </div>
                                )}
                            </>
                        ) : (
                            <div>
                                <div className="flex items-center space-x-3 mb-3">
                                    <div className="w-8 h-8 bg-gray-300 rounded-full flex items-center justify-center">
                                        <span className="text-xs">?</span>
                                    </div>
                                    <div>
                                        <h3 className="font-bold text-lg">Pool: {poolId}</h3>
                                        <p className="text-sm text-blue-100">Coin data not available</p>
                                    </div>
                                </div>
                                
                                <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-sm">
                                    <div>
                                        <p className="text-blue-100">Pool ID:</p>
                                        <p className="font-mono text-xs break-all">{poolId}</p>
                                    </div>
                                    <div>
                                        <p className="text-blue-100">Status:</p>
                                        <p className="font-medium text-yellow-400">Loading...</p>
                                    </div>
                                </div>
                                
                                <div className="mt-3">
                                    <p className="text-blue-100 text-sm">Note:</p>
                                    <p className="text-sm text-gray-700">Unable to fetch coin information. This might be a new pool or the API is unavailable.</p>
                                </div>
                            </div>
                        )}
                    </div>
                    
                    {/* Trades Section */}
                    <div>
                        <h2 className="text-xl font-bold mb-4">
                            Recent Trades {coin ? `- ${coin.symbol}` : `- ${poolId}`}
                        </h2>
                        <div className="space-y-2 max-h-96 overflow-y-auto">
                            {trades.length === 0 ? (
                                <div className="text-center py-8">
                                    <p className="text-blue-100 mb-2">No trades yet</p>
                                    <p className="text-xs text-slate-500">Trades will appear here when users start trading this token</p>
                                </div>
                            ) : (
                                trades.map((trade, index) => {
                                    const hasValidData = trade.quantity && trade.volume && trade.price && 
                                        parseFloat(trade.quantity) > 0 && parseFloat(trade.volume) > 0 && parseFloat(trade.price) > 0;
                                    
                                    return (
                                        <div key={`${trade.id}-${index}`} className="flex items-center justify-between py-3 px-4 border-b border-gray-200 hover:bg-gray-50 transition-colors">
                                            <div className="flex items-center space-x-3">
                                                <div className={`w-2 h-2 rounded-full ${hasValidData ? 'bg-green-400' : 'bg-gray-400'}`}></div>
                                                <div>
                                                    <p className="text-sm font-medium">
                                                        {hasValidData ? (
                                                            <>
                                                                <span className="text-gray-700">Qty: {formatAmount(trade.quantity)} tokens</span>
                                                                <span className="mx-2 text-slate-500">•</span>
                                                                <span className="text-gray-700">Vol: {formatAmount(trade.volume)} SOL</span>
                                                            </>
                                                        ) : (
                                                            <span className="text-slate-500">Incomplete trade data</span>
                                                        )}
                                                    </p>
                                                    <p className="text-xs text-blue-100">
                                                        ID: {trade.id} • Pool: {trade.pool_id?.slice(0, 8)}...
                                                    </p>
                                                </div>
                                            </div>
                                            <div className="text-right">
                                                <p className="text-sm font-medium">
                                                    {hasValidData ? 
                                                        `${formatPrice(trade.price)} SOL` : 
                                                        'N/A'
                                                    }
                                                </p>
                                                <p className="text-xs text-blue-100">
                                                    {formatTime(trade.timestamp)}
                                                </p>
                                            </div>
                                        </div>
                                    );
                                })
                            )}
                        </div>
                    </div>
                </>
            )}
        </div>
    );
}