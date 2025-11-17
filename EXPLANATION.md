# 📊 Leptos Real-Time Market Data System

## 🎯 System Overview

This is a high-performance **real-time market data streaming application** built with Rust, Leptos, and WebSockets. It demonstrates **low-latency data streaming**, **frontend performance optimization**, and **real-time visualization** capabilities.

### Key Features
- 🚀 **High-frequency WebSocket streaming** (10-100 Hz configurable)
- 📈 **Real-time market data simulation** (prices, trades, order books)
- ⚡ **Performance monitoring** (latency, FPS, throughput)
- 🎛️ **Live configuration controls** (frequency, buffer sizes)
- 📊 **Interactive SVG charts** (no external dependencies)
- 🔄 **Reactive UI** with Leptos signals

---

## 📁 File Structure & Responsibilities

### **Root Level**
```
leptos-rust/
├── Cargo.toml              # Workspace configuration & dependencies
├── rust-toolchain.toml     # Rust version specification
├── src/main.rs             # Unused entry point (Hello World)
└── EXPLANATION.md          # This documentation
```

### **Server Package** (`server/`)
```
server/
├── Cargo.toml              # Server-specific dependencies
└── src/
    └── main.rs             # 🔥 MAIN SERVER - Axum + WebSocket server
```

**Server Responsibilities:**
- **WebSocket server** on `/ws` endpoint
- **4 concurrent data streams** generating market data
- **Broadcast channel** for message distribution
- **Configurable update frequencies**
- **Static file serving** for frontend assets

### **App Package** (`app/`)
```
app/
├── Cargo.toml              # Shared logic dependencies
└── src/
    └── lib.rs              # 🔥 MAIN FRONTEND - Leptos components & UI
```

**App Responsibilities:**
- **WebSocket client** connection & message handling
- **Reactive state management** (prices, trades, metrics)
- **Real-time UI rendering** (charts, feeds, controls)
- **Performance measurement** (latency, FPS tracking)
- **User interface** components & styling

### **Frontend Package** (`frontend/`)
```
frontend/
├── Cargo.toml              # WASM compilation settings
└── src/
    └── lib.rs              # 🔥 WASM HYDRATION - Client-side initialization
```

**Frontend Responsibilities:**
- **WASM compilation target** (`cdylib`)
- **Client-side hydration** of server-rendered HTML
- **Browser API initialization** (console logging, error handling)

### **Supporting Files**
```
public/                     # Static assets directory
style/main.scss            # Main stylesheet
end2end/                   # Playwright E2E tests
target/                    # Build output directory
```

---

## 🏗️ Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        BROWSER                              │
│  ┌─────────────────┐    WebSocket    ┌─────────────────┐    │
│  │   Frontend      │ ◄─────────────► │   Server        │    │
│  │   (WASM)        │     /ws         │   (Axum)        │    │
│  │                 │                 │                 │    │
│  │ • UI Rendering  │                 │ • Data Streams  │    │
│  │ • State Mgmt    │                 │ • Broadcasting  │    │
│  │ • Performance   │                 │ • HTTP Server   │    │
│  └─────────────────┘                 └─────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

---

## 📊 Data Flow Architecture

### **Server-Side Data Generation**

#### **1. Price Stream** (20-60 Hz)
```rust
PriceTick {
    type: "price",
    symbol: String,    // "BTC/USD", "ETH/USD", etc.
    price: f64,        // Random walk with 0.2% volatility
    volume: u64,       // 100-10,000 random volume
    ts: i64,          // Microsecond timestamp
}
```

#### **2. Trade Stream** (5-20 Hz)
```rust
Trade {
    type: "trade",
    symbol: String,    // Random symbol selection
    price: f64,        // Market price ± random spread
    size: f64,         // 0.01-5.0 trade size
    side: String,      // "buy" or "sell"
    ts: i64,
}
```

#### **3. Order Book Stream** (10-30 Hz)
```rust
BookUpdate {
    type: "book",
    symbol: String,    // "BTC/USD", "ETH/USD"
    bids: Vec<(f64, f64)>,  // 5 bid levels [price, size]
    asks: Vec<(f64, f64)>,  // 5 ask levels [price, size]
    ts: i64,
}
```

#### **4. System Metrics** (1 Hz)
```rust
SystemMetric {
    type: "system",
    cpu_pct: f64,      // 10-80% simulated CPU
    mem_mb: u64,       // 500-2000 MB simulated memory
    msg_rate: u64,     // Cumulative messages/second
    ts: i64,
}
```

### **WebSocket Communication**

#### **Server → Client Messages**
All messages are JSON-serialized and sent via WebSocket:
```json
{"type": "price", "symbol": "BTC/USD", "price": 45123.45, "volume": 1250, "ts": 1637123456789}
{"type": "trade", "symbol": "ETH/USD", "price": 2501.23, "size": 2.5, "side": "buy", "ts": 1637123456790}
{"type": "book", "symbol": "BTC/USD", "bids": [[45000, 1.2], [44999, 0.8]], "asks": [[45010, 0.9]], "ts": 1637123456791}
{"type": "system", "cpu_pct": 45.2, "mem_mb": 1200, "msg_rate": 1250, "ts": 1637123456792}
```

#### **Client → Server Control**
```json
{"frequency_ms": 25}  // Change update frequency for all streams
```

### **Frontend State Management**

#### **Reactive Signals** (Leptos)
```rust
// Market Data Storage
let prices = RwSignal::new(HashMap<String, Vec<f64>>);      // Symbol → Price History
let trades = RwSignal::new(Vec<(String, f64, String)>);     // Recent Trades (last 100)
let book_depth = RwSignal::new(HashMap<String, (Vec<f64>, Vec<f64>)>); // Bid/Ask Prices

// Performance Metrics
let msg_rate = RwSignal::new(Vec<u64>);           // Messages/second history
let latency_values = RwSignal::new(Vec<f64>);     // End-to-end latency (ms)
let fps_values = RwSignal::new(Vec<f64>);         // Render performance (FPS)
let msg_count = RwSignal::new(0u64);              // Total message counter

// Configuration
let sample_max = RwSignal::new(200usize);         // Rolling buffer size
```

---

## ⚡ Performance Monitoring System

### **Latency Measurement**
```rust
// 1. Message receive timestamp
let t_recv = web_sys::window().unwrap().performance().unwrap().now();

// 2. Schedule measurement after next paint
let cb = Closure::wrap(Box::new(move |_: f64| {
    let t_paint = web_sys::window().unwrap().performance().unwrap().now();
    let latency = t_paint - t_recv;  // End-to-end latency
    latency_values.update(|v| v.push(latency));
}));
web_sys::window().unwrap().request_animation_frame(cb.as_ref().unchecked_ref());
```

### **FPS Tracking**
```rust
// Count frames via requestAnimationFrame
let frames = RwSignal::new(0u32);
let start_time = RwSignal::new(performance.now());

// Calculate FPS every 1000ms
let raf_callback = Closure::wrap(Box::new(move |_: f64| {
    frames.update(|f| *f += 1);
    let elapsed = performance.now() - *start_time.read();
    
    if elapsed >= 1000.0 {
        let fps = *frames.read() as f64 * 1000.0 / elapsed;
        fps_values.update(|v| v.push(fps));
        // Reset counters...
    }
}));
```

### **Statistics Calculation**
```rust
fn stats(data: &[f64]) -> (f64, f64, f64) {  // (mean, p50, p95)
    let mean = data.iter().sum::<f64>() / (data.len() as f64);
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    
    let p50 = sorted[((sorted.len() - 1) as f64 * 0.50) as usize];
    let p95 = sorted[((sorted.len() - 1) as f64 * 0.95) as usize];
    
    (mean, p50, p95)
}
```

---

## 🎛️ Configuration & Controls

### **Update Frequency Control**
- **Default**: 50ms (20 Hz)
- **Range**: 10-1000ms (100 Hz - 1 Hz)
- **Effect**: Lower values = higher message rate = more stress testing

### **Sample Window Sizes**
- **200 samples**: ~10-second window at 20 Hz
- **500 samples**: ~25-second window at 20 Hz  
- **1000 samples**: ~50-second window at 20 Hz

### **Memory Management**
```rust
// Automatic buffer trimming to prevent memory leaks
if entry.len() > cap { 
    entry.drain(0..entry.len() - cap); 
}
```

---

## 🚀 Build & Run Instructions

### **Development Mode**
```bash
# Install dependencies
cargo build

# Run server with hot reload
cargo leptos watch

# Server runs on http://127.0.0.1:3000
# WebSocket endpoint: ws://127.0.0.1:3000/ws
```

### **Production Build**
```bash
# Optimized build
cargo leptos build --release

# Run production server
cargo run --release --bin server
```

### **Testing**
```bash
# Run E2E tests
cd end2end
npm test

# Unit tests
cargo test
```

---

## 📈 Performance Benchmarking

### **What to Monitor**
1. **Message Throughput**: Messages/second received
2. **Render Performance**: FPS under load
3. **End-to-End Latency**: WebSocket → DOM paint time
4. **Memory Usage**: Browser DevTools memory tab
5. **CPU Usage**: Browser task manager

### **Stress Testing Scenarios**
1. **High Frequency**: Set to 10ms (100 Hz) and monitor FPS
2. **Large Buffers**: Increase sample window to 1000+ points
3. **Multiple Tabs**: Open same page in multiple browser tabs
4. **Background Load**: Run other applications while testing
5. **Network Throttling**: Use DevTools to simulate slow networks

### **Performance Targets**
- **Latency p95**: < 16ms (60 FPS budget)
- **Sustained FPS**: > 55 FPS under load
- **Message Rate**: 1000+ msgs/sec without drops
- **Memory**: Stable (no leaks over time)

---

## 🔧 Technical Implementation Details

### **WebSocket Connection Management**
```rust
// Server: Broadcast channel for 1:N message distribution
let (tx, _rx) = broadcast::channel::<String>(500);  // 500 message buffer

// Client: Auto-reconnection and error handling
if let Ok(ws) = WebSocket::new(&ws_url) {
    // Save reference for control messages
    js_sys::Reflect::set(window.as_ref(), &"__leptos_ws".into(), ws.as_ref());
}
```

### **Async Task Coordination**
```rust
// Server: 4 concurrent tokio tasks for data generation
tokio::spawn(async move { /* Price stream */ });
tokio::spawn(async move { /* Trade stream */ });  
tokio::spawn(async move { /* Book stream */ });
tokio::spawn(async move { /* System metrics */ });

// Shared frequency control via Arc<AtomicU64>
let sleep_ms = Arc::new(AtomicU64::new(50));
```

### **Memory-Efficient Data Structures**
```rust
// Rolling buffers with automatic cleanup
let mut entry = prices_map.entry(symbol).or_insert_with(Vec::new);
entry.push(new_price);

// Prevent unbounded growth
let cap = *sample_max.read();
if entry.len() > cap { 
    entry.drain(0..entry.len() - cap); 
}
```

### **SVG Chart Rendering**
```rust
fn sparkline_points(data: &[f64], width: f64, height: f64) -> String {
    // Normalize data to viewport coordinates
    let (min, max) = data.iter().fold((f64::INFINITY, f64::NEG_INFINITY), 
        |(mn, mx), &v| (mn.min(v), mx.max(v)));
    let range = if (max - min).abs() < 1e-9 { 1.0 } else { max - min };
    
    // Generate SVG polyline points
    data.iter().enumerate().map(|(i, &v)| {
        let x = (width / (data.len() - 1) as f64) * i as f64;
        let y = height - ((v - min) / range) * height;
        format!("{:.1},{:.1}", x, y)
    }).collect::<Vec<_>>().join(" ")
}
```

---

## 🎯 Use Cases & Applications

### **Financial Technology**
- **Trading Platform Prototyping**: Real-time price feeds
- **Market Data Analysis**: Performance under high-frequency updates  
- **Latency Optimization**: Sub-millisecond trading requirements

### **Performance Testing**
- **WebSocket Stress Testing**: High-throughput message handling
- **Frontend Optimization**: Render performance under continuous updates
- **Memory Leak Detection**: Long-running data stream monitoring
- **Browser Compatibility**: Cross-platform performance validation

### **Real-Time Dashboards**
- **IoT Data Monitoring**: Sensor data visualization
- **System Metrics**: Server monitoring and alerting
- **Live Analytics**: User activity and engagement tracking
- **Gaming**: Real-time player statistics and leaderboards

---

## 📊 Architecture Decisions & Trade-offs

### **Why Leptos?**
✅ **Compile-time optimizations** (smaller WASM bundles)  
✅ **Fine-grained reactivity** (efficient DOM updates)  
✅ **Server-side rendering** (fast initial page loads)  
✅ **Type safety** (Rust's type system prevents runtime errors)

### **Why WebSockets over SSE/HTTP?**
✅ **Bidirectional communication** (client can control server)  
✅ **Lower latency** (no HTTP overhead per message)  
✅ **Binary support** (future extensibility)  
⚠️ **More complex** (connection management, reconnection logic)

### **Why In-Memory Simulation vs Real APIs?**
✅ **Deterministic testing** (controlled data patterns)  
✅ **No external dependencies** (always available)  
✅ **Configurable load** (adjustable message rates)  
⚠️ **Not realistic** (missing real market dynamics)

---

## 🔮 Future Enhancements

### **Real Data Integration** (Next Steps!)
- [ ] **Cryptocurrency APIs**: Binance WebSocket, Coinbase Pro
- [ ] **Stock Market APIs**: Alpha Vantage, IEX Cloud
- [ ] **Forex Data**: OANDA, ForexConnect APIs
- [ ] **Multiple Data Sources**: Aggregated feeds with failover

### **Performance Optimizations**
- [ ] **WebWorker Integration**: Off-main-thread message processing
- [ ] **Canvas Rendering**: Hardware-accelerated charts
- [ ] **Compression**: WebSocket message compression (deflate)
- [ ] **Buffering Strategies**: Adaptive batching based on load

### **Advanced Features**
- [ ] **Time Series Database**: Historical data storage (InfluxDB)
- [ ] **Real-time Analytics**: Moving averages, technical indicators
- [ ] **Multi-user Support**: Shared sessions and collaborative views
- [ ] **Alert System**: Price thresholds and notifications

---

*This system demonstrates production-ready patterns for building high-performance real-time web applications with Rust and Leptos.* 🚀