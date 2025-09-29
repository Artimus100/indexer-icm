"use client";
import { useEffect, useState } from "react";
import { Coin } from "../utils/types";
import { SignalingManager } from "../utils/SignalingManager";
import { getRecentCoins } from "../utils/httpClient";
import Link from "next/link";

export const Coins = () => {
    const [coins, setCoins] = useState<Coin[]>([]);

    useEffect(() => {
        console.log("Coins component: Setting up subscription and fetching initial data");
        
        // Fetch initial coins data
        const fetchInitialCoins = async () => {
            try {
                console.log("Coins component: Fetching recent coins from API");
                const recentCoins = await getRecentCoins(50);
                console.log("Coins component: Fetched coins:", recentCoins.length);
                setCoins(recentCoins);
            } catch (error) {
                console.error("Coins component: Error fetching initial coins:", error);
            }
        };

        fetchInitialCoins();
        
        const handleNewCoin = (data: any) => {
            console.log("Received coin data from websocket:", data);
            console.log("Data type:", typeof data);
            console.log("Data keys:", Object.keys(data || {}));
            
            if (data && data.mint) {
                // Transform the data to match our Coin interface
                const transformedCoin: Partial<Coin> = {
                    id: data.id || 0,
                    mint: data.mint,
                    name: data.name || '',
                    symbol: data.symbol || '',
                    description: data.description,
                    image_uri: data.image_uri,
                    metadata_uri: data.metadata_uri,
                    twitter: data.twitter,
                    telegram: data.telegram,
                    website: data.website,
                    pool_address: data.pool_address || data.mint,
                    creator: data.creator || '',
                    created_timestamp: data.created_timestamp || new Date().toISOString(),
                    complete: data.complete || false,
                    virtual_sol_reserves: data.virtual_sol_reserves || 0,
                    virtual_token_reserves: data.virtual_token_reserves || 0,
                    total_supply: data.total_supply || 0,
                    market_cap: data.market_cap,
                    usd_market_cap: data.usd_market_cap,
                    reply_count: data.reply_count || 0,
                    is_banned: data.is_banned || false,
                    program: data.program || 'meteora',
                    created_at: data.created_at || new Date().toISOString(),
                    updated_at: data.updated_at || new Date().toISOString()
                };
                
                console.log("Transformed coin data:", transformedCoin);
                
                setCoins(prevCoins => {
                    console.log("Previous coins count:", prevCoins.length);
                    const existingIndex = prevCoins.findIndex(coin => coin.mint === transformedCoin.mint);
                    if (existingIndex !== -1) {
                        // Update existing coin
                        console.log("Updating existing coin at index:", existingIndex);
                        const updatedCoins = [...prevCoins];
                        updatedCoins[existingIndex] = { ...updatedCoins[existingIndex], ...transformedCoin };
                        console.log("Updated coins count:", updatedCoins.length);
                        return updatedCoins;
                    } else {
                        // Add new coin to the beginning of the list
                        console.log("Adding new coin to the beginning:", transformedCoin);
                        const newCoins = [transformedCoin as Coin, ...prevCoins];
                        console.log("New coins count:", newCoins.length);
                        return newCoins;
                    }
                });
            } else {
                console.warn("Invalid coin data received:", data);
            }
        };
        
        // Register callback with a small delay to ensure websocket is ready
        const registerCallback = () => {
            console.log("Registering LAUNCH_COIN callback");
            SignalingManager.getInstance().registerCallback("LAUNCH_COIN", handleNewCoin, "COINS-SUBSCRIPTION");
        };
        
        // Register callback immediately and also after a short delay
        registerCallback();
        setTimeout(registerCallback, 100);
        
        console.log("Coins component: Sending subscribe message");
        const subscribeMessage = {
            "type": "SUBSCRIBE",
            "room": "coins",
            "data": null
        };
        console.log("Coins component: Subscribe message:", subscribeMessage);
        SignalingManager.getInstance().sendMessage(subscribeMessage);

        return () => {
            console.log("Coins component: Cleaning up subscription");
            SignalingManager.getInstance().deRegisterCallback("LAUNCH_COIN", "COINS-SUBSCRIPTION");
            SignalingManager.getInstance().sendMessage({
                "type": "UNSUBSCRIBE", 
                "room": "coins",
                "data": null
            });
        }
    }, []);

    return (
        <div className="w-full">
            <h2 className="text-xl font-bold mb-4">New Coins</h2>
            <div className="space-y-4">
                {coins.map((coin) => (
                    <Link 
                        key={coin.mint} 
                        href={`/coins/${coin.pool_address}`}
                        className="block border border-gray-300 rounded-lg p-4 hover:bg-gray-50 transition-colors cursor-pointer"
                    >
                        <div className="flex items-center justify-between">
                            <div className="flex items-center space-x-3">
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
                                    <h3 className="font-medium">{coin.name}</h3>
                                    <p className="text-sm text-slate-400">{coin.symbol}</p>
                                </div>
                            </div>
                            <div className="text-right">
                                <p className="text-sm text-slate-400">Market Cap</p>
                                <p className="font-medium">
                                    {coin.usd_market_cap ? `$${coin.usd_market_cap.toLocaleString()}` : 'N/A'}
                                </p>
                            </div>
                        </div>
                        {coin.description && (
                            <p className="text-sm text-slate-400 mt-2">{coin.description}</p>
                        )}
                        <div className="flex items-center justify-between mt-3 text-xs text-slate-500">
                            <span>Creator: {coin.creator.slice(0, 8)}...</span>
                            <span>{coin.complete ? 'Complete' : 'In Progress'}</span>
                        </div>
                    </Link>
                ))}
            </div>
        </div>
    );
};
