import axios from "axios";
import { KLine, Coin, Trade } from "./types";

const BASE_URL = "http://localhost:3001/api/v1"

// Klines endpoints
export async function getKlines(market: string, interval: string, startTime: number, endTime: number): Promise<KLine[]> {
    const response = await axios.get(`${BASE_URL}/klines/${market}`, {
        params: {
            interval,
            start_time: startTime.toString(),
            end_time: endTime.toString()
        }
    });
    return response.data;
}

// Coins endpoints
export async function getTrendingCoins(limit?: number): Promise<Coin[]> {
    const response = await axios.get(`${BASE_URL}/coins/trending`, {
        params: { limit }
    });
    return response.data;
}

export async function getRecentCoins(limit?: number): Promise<Coin[]> {
    const response = await axios.get(`${BASE_URL}/coins/recent`, {
        params: { limit }
    });
    return response.data;
}

export async function searchCoins(query: string, limit?: number): Promise<Coin[]> {
    const response = await axios.get(`${BASE_URL}/coins/search`, {
        params: { q: query, limit }
    });
    return response.data;
}

export async function getCoinsByCreator(creator: string, limit?: number): Promise<Coin[]> {
    const response = await axios.get(`${BASE_URL}/coins/creator`, {
        params: { creator, limit }
    });
    return response.data;
}

export async function getCoinByMint(mint: string): Promise<Coin | null> {
    const response = await axios.get(`${BASE_URL}/coins/mint/${mint}`);
    return response.data;
}

export async function getCoinByPoolId(poolId: string): Promise<Coin | null> {
    const response = await axios.get(`${BASE_URL}/coins/pool/${poolId}`);
    return response.data;
}

export async function getAllCoins(limit?: number, page?: number): Promise<Coin[]> {
    const response = await axios.get(`${BASE_URL}/coins`, {
        params: { limit, page }
    });
    return response.data;
}

export async function getTopHolders(mint: string): Promise<any> {
    const response = await axios.get(`${BASE_URL}/coins/${mint}/holders`);
    return response.data;
}

export async function getBondingCurveStats(mint: string): Promise<any> {
    const response = await axios.get(`${BASE_URL}/coins/${mint}/bonding-curve`);
    return response.data;
}

// Trades endpoints
export async function getRecentTrades(mint: string, limit?: number, offset?: number, startTime?: string, endTime?: string): Promise<Trade[]> {
    const response = await axios.get(`${BASE_URL}/trades/${mint}`, {
        params: { limit, offset, start_time: startTime, end_time: endTime }
    });
    return response.data;
}

export async function getTradeById(id: number): Promise<Trade | null> {
    const response = await axios.get(`${BASE_URL}/trades/id/${id}`);
    return response.data;
}