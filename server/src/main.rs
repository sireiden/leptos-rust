use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use app::*;
use leptos::logging::log;
use tokio::sync::broadcast;
use rand::Rng;
use std::time::Duration;
use serde::Serialize;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

mod live_data;

#[derive(Clone)]
struct AppState {
    leptos_options: LeptosOptions,
    tx: broadcast::Sender<String>,
    sleep_ms: Arc<AtomicU64>, // controls update frequency for all streams
    use_live_data: bool,      // toggle between simulated and real data
}

#[tokio::main]
async fn main() {

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // broadcast channel for live updates (high capacity for bursts)
    let (tx, _rx) = broadcast::channel::<String>(500);

    // tunable: message frequency in milliseconds (lower = faster updates)
    let sleep_ms = Arc::new(AtomicU64::new(50)); // default 50ms = ~20 Hz

    // Check if we should use live data (environment variable or command line arg)
    let use_live_data = std::env::var("USE_LIVE_DATA").unwrap_or_default() == "true" ||
                        std::env::args().any(|arg| arg == "--live-data");

    if use_live_data {
        println!("🔥 Starting LIVE data streams from Binance WebSocket...");
        let live_client = live_data::LiveDataClient::new(tx.clone());
        live_client.start_binance_streams().await;
        
        // Still use simulated system metrics
        live_data::start_system_metrics_stream(tx.clone());
        
        println!("✅ Live data streams started! Connect to ws://127.0.0.1:3000/ws");
    } else {
        println!("🤖 Starting SIMULATED data streams (use_live_data={})...", use_live_data);
        
        // ========== Realistic high-frequency simulated data streams ==========
        // We'll simulate 3 concurrent streams that fire at high rates to stress the frontend

        // Stream 1: Market price ticks (20-60 Hz) - simulates real-time price updates
    #[derive(Serialize)]
    struct PriceTick {
        #[serde(rename = "type")] t: &'static str,
        symbol: String,
        price: f64,
        volume: u64,
        ts: i64, // micros since epoch
    }

    let tx_price = tx.clone();
    let freq_ms = sleep_ms.clone(); // use sleep_ms as update frequency control
    tokio::spawn(async move {
        let symbols = vec!["BTC/USD", "ETH/USD", "SOL/USD", "AAPL", "TSLA"];
        let mut prices = vec![45000.0, 2500.0, 120.0, 175.0, 250.0];
        loop {
            {
                let mut rng = rand::thread_rng();
                for (idx, symbol) in symbols.iter().enumerate() {
                    // Random walk with volatility
                    let change = rng.gen_range(-0.002..0.002);
                    prices[idx] *= 1.0 + change;
                    let volume = rng.gen_range(100..10000);
                    let tick = PriceTick {
                        t: "price",
                        symbol: symbol.to_string(),
                        price: (prices[idx] * 100.0_f64).round() / 100.0,
                        volume,
                        ts: chrono::Utc::now().timestamp_micros(),
                    };
                    if let Ok(s) = serde_json::to_string(&tick) { let _ = tx_price.send(s); }
                }
            } // rng dropped here
            let interval = freq_ms.load(Ordering::Relaxed).max(10); // min 10ms = 100Hz
            tokio::time::sleep(Duration::from_millis(interval)).await;
        }
    });

    // Stream 2: Order book depth updates (10-30 Hz)
    #[derive(Serialize)]
    struct BookUpdate {
        #[serde(rename = "type")] t: &'static str,
        symbol: String,
        bids: Vec<(f64, f64)>, // price, size
        asks: Vec<(f64, f64)>,
        ts: i64,
    }

    let tx_book = tx.clone();
    let freq_book = sleep_ms.clone();
    tokio::spawn(async move {
        let symbols = vec!["BTC/USD", "ETH/USD"];
        loop {
            {
                let mut rng = rand::thread_rng();
                for symbol in &symbols {
                    let mid = if *symbol == "BTC/USD" { 45000.0 } else { 2500.0 };
                    let mut bids = Vec::new();
                    let mut asks = Vec::new();
                    for i in 0..5 {
                        let bid_price = mid - (i as f64) * mid * 0.0001;
                        let ask_price = mid + (i as f64) * mid * 0.0001;
                        bids.push((bid_price, rng.gen_range(0.1..10.0)));
                        asks.push((ask_price, rng.gen_range(0.1..10.0)));
                    }
                    let update = BookUpdate {
                        t: "book",
                        symbol: symbol.to_string(),
                        bids,
                        asks,
                        ts: chrono::Utc::now().timestamp_micros(),
                    };
                    if let Ok(s) = serde_json::to_string(&update) { let _ = tx_book.send(s); }
                }
            }
            let interval = (freq_book.load(Ordering::Relaxed) * 2).max(50);
            tokio::time::sleep(Duration::from_millis(interval)).await;
        }
    });

    // Stream 3: Trade executions (sporadic bursts, 5-20 Hz)
    #[derive(Serialize)]
    struct Trade {
        #[serde(rename = "type")] t: &'static str,
        symbol: String,
        price: f64,
        size: f64,
        side: &'static str, // "buy" or "sell"
        ts: i64,
    }

    let tx_trade = tx.clone();
    let freq_trade = sleep_ms.clone();
    tokio::spawn(async move {
        let symbols = vec!["BTC/USD", "ETH/USD", "SOL/USD"];
        loop {
            {
                let mut rng = rand::thread_rng();
                let symbol = symbols[rng.gen_range(0..symbols.len())];
                let price = match symbol {
                    "BTC/USD" => 45000.0 + rng.gen_range(-100.0..100.0),
                    "ETH/USD" => 2500.0 + rng.gen_range(-10.0..10.0),
                    _ => 120.0 + rng.gen_range(-1.0..1.0),
                };
                let trade = Trade {
                    t: "trade",
                    symbol: symbol.to_string(),
                    price: (price * 100.0_f64).round() / 100.0,
                    size: rng.gen_range(0.01..5.0),
                    side: if rng.gen_bool(0.5) { "buy" } else { "sell" },
                    ts: chrono::Utc::now().timestamp_micros(),
                };
                if let Ok(s) = serde_json::to_string(&trade) { let _ = tx_trade.send(s); }
            }
            let interval = (freq_trade.load(Ordering::Relaxed) * 3).max(50);
            tokio::time::sleep(Duration::from_millis(interval)).await;
        }
    });

    // Stream 4: System metrics (lower frequency but adds context)
    #[derive(Serialize)]
    struct SystemMetric {
        #[serde(rename = "type")] t: &'static str,
        cpu_pct: f64,
        mem_mb: u64,
        msg_rate: u64, // messages per second
        ts: i64,
    }

    let tx_sys = tx.clone();
    tokio::spawn(async move {
        let mut msg_count = 0u64;
        loop {
            {
                let mut rng = rand::thread_rng();
                msg_count += rng.gen_range(50..200); // simulate msg throughput
                let metric = SystemMetric {
                    t: "system",
                    cpu_pct: rng.gen_range(10.0..80.0),
                    mem_mb: rng.gen_range(500..2000),
                    msg_rate: msg_count,
                    ts: chrono::Utc::now().timestamp_micros(),
                };
                if let Ok(s) = serde_json::to_string(&metric) { let _ = tx_sys.send(s); }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
        
        println!("✅ Simulated data streams started!");
    }

    let state = AppState {
        leptos_options: leptos_options.clone(),
        tx,
        sleep_ms,
        use_live_data,
    };
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .leptos_routes(&state, routes, {
            let leptos_options = state.leptos_options.clone();
            move || shell(leptos_options.clone())
        })
    .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

// Allow leptos_axum to extract LeptosOptions from our composite AppState
impl axum::extract::FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> LeptosOptions {
        state.leptos_options.clone()
    }
}


async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_connection(socket, state))
}

#[derive(serde::Deserialize)]
struct ControlMsg {
    frequency_ms: Option<u64>, // controls update rate for all streams
}

async fn ws_connection(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();

    loop {
        tokio::select! {
            res = rx.recv() => {
                match res {
                    Ok(msg) => {
                        if socket.send(Message::Text(msg.into())).await.is_err() { break; }
                    }
                    Err(_) => break,
                }
            }
            maybe_in = socket.recv() => {
                match maybe_in {
                    Some(Ok(Message::Text(txt))) => {
                        if let Ok(ctrl) = serde_json::from_str::<ControlMsg>(&txt) {
                            if let Some(freq) = ctrl.frequency_ms {
                                state.sleep_ms.store(freq.max(10).min(1000), Ordering::Relaxed);
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}
