use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;
use rand::Rng;

#[derive(Debug)]
pub struct LiveDataClient {
    tx: broadcast::Sender<String>,
}

impl LiveDataClient {
    pub fn new(tx: broadcast::Sender<String>) -> Self {
        Self { tx }
    }

    /// Start live data streams from Binance WebSocket
    pub async fn start_binance_streams(&self) {
        let symbols = vec!["btcusdt", "ethusdt", "solusdt"];
        
        for symbol in symbols {
            let tx = self.tx.clone();
            let symbol_clone = symbol.to_string();
            
            // Start price ticker stream
            tokio::spawn(async move {
                Self::binance_ticker_stream(&symbol_clone, tx).await;
            });
            
            // Add small delay between connections
            sleep(Duration::from_millis(100)).await;
        }
        
        // Start a combined trade stream for all symbols
        let tx_trades = self.tx.clone();
        tokio::spawn(async move {
            Self::binance_trade_streams(tx_trades).await;
        });
    }

    /// Binance ticker stream for price updates (24hr rolling window stats)
    async fn binance_ticker_stream(symbol: &str, tx: broadcast::Sender<String>) {
        loop {
            match Self::connect_ticker_stream(symbol, &tx).await {
                Ok(_) => {},
                Err(error_msg) => {
                    eprintln!("Ticker stream error for {}: {}", symbol, error_msg);
                    sleep(Duration::from_secs(5)).await; // Reconnect delay
                }
            }
        }
    }

    async fn connect_ticker_stream(symbol: &str, tx: &broadcast::Sender<String>) -> Result<(), String> {
        let url = format!("wss://stream.binance.com:9443/ws/{}@ticker", symbol);
        println!("Connecting to Binance ticker stream: {}", url);
        
        let (ws_stream, _) = connect_async(&url).await.map_err(|e| e.to_string())?;
        let (mut write, mut read) = ws_stream.split();
        
        // Keep connection alive with pings
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if write.send(Message::Ping(vec![])).await.is_err() {
                    break;
                }
            }
        });

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(data) = serde_json::from_str::<Value>(&text) {
                        // Transform Binance data to our format
                        if let Some(transformed) = Self::transform_binance_ticker(&data) {
                            let _ = tx.send(transformed);
                        }
                    }
                }
                Ok(Message::Pong(_)) => {
                    // Keep alive response
                }
                Ok(Message::Close(_)) | Err(_) => {
                    return Err("Connection closed".into());
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    /// Combined trade streams for multiple symbols
    async fn binance_trade_streams(tx: broadcast::Sender<String>) {
        loop {
            match Self::connect_trade_streams(&tx).await {
                Ok(_) => {},
                Err(error_msg) => {
                    eprintln!("Trade streams error: {}", error_msg);
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn connect_trade_streams(tx: &broadcast::Sender<String>) -> Result<(), String> {
        let url = "wss://stream.binance.com:9443/stream?streams=btcusdt@trade/ethusdt@trade/solusdt@trade";
        println!("Connecting to Binance trade streams: {}", url);
        
        let (ws_stream, _) = connect_async(url).await.map_err(|e| e.to_string())?;
        let (mut write, mut read) = ws_stream.split();
        
        // Keep connection alive
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if write.send(Message::Ping(vec![])).await.is_err() {
                    break;
                }
            }
        });

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(data) = serde_json::from_str::<Value>(&text) {
                        if let Some(transformed) = Self::transform_binance_trade(&data) {
                            let _ = tx.send(transformed);
                        }
                    }
                }
                Ok(Message::Pong(_)) => {}
                Ok(Message::Close(_)) | Err(_) => {
                    return Err("Trade connection closed".into());
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    /// Transform Binance ticker data to our price format
    fn transform_binance_ticker(data: &Value) -> Option<String> {
        let symbol = data.get("s")?.as_str()?;
        let price = data.get("c")?.as_str()?.parse::<f64>().ok()?;
        let volume = data.get("v")?.as_str()?.parse::<f64>().ok()? as u64;
        
        // Convert to our format
        let transformed = serde_json::json!({
            "type": "price",
            "symbol": Self::normalize_symbol(symbol),
            "price": price,
            "volume": volume,
            "ts": chrono::Utc::now().timestamp_micros()
        });
        
        serde_json::to_string(&transformed).ok()
    }

    /// Transform Binance trade data to our trade format
    fn transform_binance_trade(data: &Value) -> Option<String> {
        // Handle combined stream format
        let trade_data = if let Some(stream_data) = data.get("data") {
            stream_data
        } else {
            data
        };
        
        let symbol = trade_data.get("s")?.as_str()?;
        let price = trade_data.get("p")?.as_str()?.parse::<f64>().ok()?;
        let size = trade_data.get("q")?.as_str()?.parse::<f64>().ok()?;
        let is_buyer_maker = trade_data.get("m")?.as_bool()?;
        
        let transformed = serde_json::json!({
            "type": "trade",
            "symbol": Self::normalize_symbol(symbol),
            "price": price,
            "size": size,
            "side": if is_buyer_maker { "sell" } else { "buy" },
            "ts": chrono::Utc::now().timestamp_micros()
        });
        
        serde_json::to_string(&transformed).ok()
    }

    /// Normalize symbol names (BTCUSDT -> BTC/USD)
    fn normalize_symbol(binance_symbol: &str) -> String {
        match binance_symbol.to_uppercase().as_str() {
            "BTCUSDT" => "BTC/USD".to_string(),
            "ETHUSDT" => "ETH/USD".to_string(),
            "SOLUSDT" => "SOL/USD".to_string(),
            _ => binance_symbol.to_uppercase()
        }
    }
}

/// System metrics generator (still simulated for now)  
pub fn start_system_metrics_stream(tx: broadcast::Sender<String>) {
    tokio::spawn(async move {
        let mut msg_count = 0u64;
        loop {
            {
                let mut rng = rand::thread_rng();
                msg_count += rng.gen_range(50..200);
                let metric = serde_json::json!({
                    "type": "system",
                    "cpu_pct": rng.gen_range(10.0..80.0),
                    "mem_mb": rng.gen_range(500..2000),
                    "msg_rate": msg_count,
                    "ts": chrono::Utc::now().timestamp_micros()
                });
                if let Ok(s) = serde_json::to_string(&metric) { 
                    let _ = tx.send(s); 
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    });
}