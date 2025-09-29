import { KLine, Trade, Coin } from "./types";

export const BASE_URL = "ws://localhost:8001/ws"

export class SignalingManager {
    private ws: WebSocket | null = null;
    private static instance: SignalingManager;
    private bufferedMessages: any[] = [];
    private callback: any = {};
    private id: number;
    private initialized: boolean = false;
    private reconnectAttempts: number = 0;
    private maxReconnectAttempts: number = 5;
    private reconnectInterval: number = 3000; // 3 seconds
    private reconnectTimer: NodeJS.Timeout | null = null;
    private heartbeatInterval: NodeJS.Timeout | null = null;

    private constructor() {
        console.log("Initializing SignalingManager with URL:", BASE_URL);
        this.bufferedMessages = [];
        this.id = 1;
        this.connect();
        this.setupPageVisibilityListener();
    }

    public static getInstance() {
        if (!this.instance) {
            this.instance = new SignalingManager();
        }
        return this.instance;
    }

    private connect() {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            console.log("WebSocket already connected");
            return;
        }

        console.log(`Attempting to connect to WebSocket (attempt ${this.reconnectAttempts + 1}/${this.maxReconnectAttempts})`);
        this.ws = new WebSocket(BASE_URL);
        this.init();
    }

    private init() {
        if (!this.ws) return;

        this.ws.onopen = () => {
            console.log("WebSocket connection established");
            this.initialized = true;
            this.reconnectAttempts = 0;
            
            // Clear any existing reconnect timer
            if (this.reconnectTimer) {
                clearTimeout(this.reconnectTimer);
                this.reconnectTimer = null;
            }
            
            // Start heartbeat
            this.startHeartbeat();
            
            // Re-register all active subscriptions
            this.reregisterSubscriptions();
            
            console.log(`Sending ${this.bufferedMessages.length} buffered messages`);
            this.bufferedMessages.forEach(message => {
                console.log("Sending buffered message:", message);
                this.sendMessageDirect(message);
            })
            this.bufferedMessages = [];
        }

        this.ws.onclose = (event) => {
            console.log("WebSocket connection closed", event.code, event.reason);
            this.initialized = false;
            this.stopHeartbeat();
            
            // Clear all callbacks to prevent memory leaks
            this.callback = {};
            
            // Attempt to reconnect if not a normal closure
            if (event.code !== 1000 && this.reconnectAttempts < this.maxReconnectAttempts) {
                this.scheduleReconnect();
            }
        }

        this.ws.onerror = (error) => {
            console.error("❌ WebSocket error:", error);
            this.initialized = false;
        }

        this.ws.onmessage = (event) => {
            console.log("Received message:", event.data);
            try {
                const message = JSON.parse(event.data);
                const type = message.type;

                console.log("Message type:", type, "Available callbacks:", Object.keys(this.callback));
                console.log("Message data:", message.data);
                console.log("Callback exists for type:", !!this.callback[type]);

            if (this.callback[type]) {
                console.log(`Found ${this.callback[type].length} callbacks for type ${type}`);
                this.callback[type].forEach(({callback}: {callback: any}) => {
                    if (type == "KLINE") {
                        const newKline: Partial<KLine> = {
                            close: message.data.close,
                            end: message.data.end,
                            high: message.data.high,
                            low: message.data.low,
                            open: message.data.open,
                            quote_volume: message.data.quote_volume,
                            start: message.data.start,
                            trades: message.data.trades,
                            volume: message.data.volume
                        }
                        console.log(newKline);
                        callback(newKline);
                    } else if (type == "TRADE") {
                        console.log("Processing TRADE message:", message.data);
                        // Pass the raw data directly to the callback for better flexibility
                        callback(message.data);
                    } else if (type == "LAUNCH_COIN") {
                        console.log("Processing LAUNCH_COIN message:", message.data);
                        // Pass the raw data directly to the callback for better flexibility
                        callback(message.data);
                    } else if (type == "KLINE") {
                        console.log("Processing KLINE message:", message.data);
                        // Pass the raw data directly to the callback for better flexibility
                        callback(message.data);
                    }
                })
            } else {
                console.log("No callback found for message type:", type);
            }
            } catch (error) {
                console.error("❌ Error parsing WebSocket message:", error);
            }
        }
    }

    private scheduleReconnect() {
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
        }
        
        this.reconnectAttempts++;
        console.log(`Scheduling reconnect in ${this.reconnectInterval}ms (attempt ${this.reconnectAttempts})`);
        
        this.reconnectTimer = setTimeout(() => {
            this.connect();
        }, this.reconnectInterval);
    }

    private startHeartbeat() {
        this.stopHeartbeat(); // Clear any existing heartbeat
        
        this.heartbeatInterval = setInterval(() => {
            if (this.ws && this.ws.readyState === WebSocket.OPEN) {
                console.log("Checking WebSocket connection health");
                // Just check connection state, don't send PING messages
                // The server doesn't support PING messages
            } else {
                console.log("WebSocket connection lost, attempting to reconnect");
                this.forceReconnect();
            }
        }, 30000); // Check connection every 30 seconds
    }

    private stopHeartbeat() {
        if (this.heartbeatInterval) {
            clearInterval(this.heartbeatInterval);
            this.heartbeatInterval = null;
        }
    }

    private sendMessageDirect(message: any) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        } else {
            console.warn("Cannot send message - WebSocket not connected");
        }
    }

    sendMessage(message: any) {
        const messageToSend = {
            ...message,
            data: message.data || null,
            id: this.id++
        }
        console.log("Sending message:", messageToSend);
        if (!this.initialized || !this.ws || this.ws.readyState !== WebSocket.OPEN) {
            console.log("WebSocket not ready, buffering message");
            this.bufferedMessages.push(messageToSend);
            return;
        }
        this.sendMessageDirect(messageToSend);
    }

    async registerCallback(type: string, callback: any, id: string) {
        console.log(`Registering callback for type: ${type} with id: ${id}`);
        this.callback[type] = this.callback[type] || [];
        
        // Remove any existing callback with the same id to avoid duplicates
        const beforeCount = this.callback[type].length;
        this.callback[type] = this.callback[type].filter((cb: any) => cb.id !== id);
        const removedCount = beforeCount - this.callback[type].length;
        
        if (removedCount > 0) {
            console.log(`Removed ${removedCount} existing callback(s) with id: ${id}`);
        }
        
        this.callback[type].push({callback, id});
        console.log(`Total callbacks for ${type}:`, this.callback[type].length);
        console.log(`Available callback types:`, Object.keys(this.callback));
        console.log(`Callback IDs for ${type}:`, this.callback[type].map((cb: any) => cb.id));
    }

    async deRegisterCallback(type: string, id: string) {
        console.log(`De-registering callback for type: ${type} with id: ${id}`);
        if (this.callback[type]) {
            const beforeCount = this.callback[type].length;
            const index = this.callback[type].findIndex((callback: any) => callback.id === id);
            if (index !== -1) {
                this.callback[type].splice(index, 1);
                console.log(`Successfully removed callback for ${type} with id: ${id}`);
            } else {
                console.log(`No callback found for ${type} with id: ${id}`);
            }
            console.log(`Total callbacks for ${type} after removal:`, this.callback[type].length);
        } else {
            console.log(`No callbacks exist for type: ${type}`);
        }
    }

    // Cleanup method for when the app is closing
    cleanup() {
        console.log("Cleaning up SignalingManager");
        this.stopHeartbeat();
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }
        if (this.ws) {
            this.ws.close(1000, "Application closing");
            this.ws = null;
        }
        this.initialized = false;
    }

    // Force reconnect method
    forceReconnect() {
        console.log("Force reconnecting WebSocket");
        this.cleanup();
        this.reconnectAttempts = 0;
        this.connect();
    }

    private setupPageVisibilityListener() {
        if (typeof document !== 'undefined') {
            document.addEventListener('visibilitychange', () => {
                if (document.visibilityState === 'visible') {
                    console.log("Page became visible - checking WebSocket connection");
                    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
                        console.log("WebSocket not connected, attempting to reconnect");
                        this.forceReconnect();
                    }
                } else {
                    console.log("Page became hidden");
                }
            });
        }
    }

    private reregisterSubscriptions() {
        // This method would need to be implemented to track active subscriptions
        // For now, we'll rely on the components to re-register their subscriptions
        console.log("Re-registering subscriptions after reconnection");
    }
}