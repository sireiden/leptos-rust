# 🔥 Live Data Integration - Real Market Data Feeds

## Available Free APIs

### 1. **Binance WebSocket Streams** (✅ Empfohlen)
- **URL**: `wss://stream.binance.us:9443`  
- **Kostenlos**: Ja, keine API Key erforderlich
- **Rate Limits**: 5 messages/second pro connection
- **Datentypen**: 
  - Real-time prices: `<symbol>@ticker`
  - Live trades: `<symbol>@trade`  
  - Order book: `<symbol>@depth`
  - Klines: `<symbol>@kline_1m`

### 2. **Alternative APIs**
- **Coinbase WebSocket**: `wss://ws-feed.pro.coinbase.com`
- **Kraken WebSocket**: `wss://ws.kraken.com`
- **Alpha Vantage**: REST API with free tier (5 calls/minute)

## Implementation Strategy

### **Phase 1**: Replace Price Stream
```rust
// Current: Random walk simulation
let change = rng.gen_range(-0.002..0.002);
prices[idx] *= 1.0 + change;

// New: Binance WebSocket feed
// {"stream":"btcusdt@ticker","data":{"c":"45123.45",...}}
```

### **Phase 2**: Replace Trade Stream  
```rust
// Current: Simulated trades
let trade = Trade { symbol, price, size, side, ts };

// New: Live trade data
// {"stream":"btcusdt@trade","data":{"p":"45123.45","q":"0.001",...}}
```

### **Phase 3**: Replace Order Book
```rust
// Current: Simulated bid/ask levels
bids.push((bid_price, rng.gen_range(0.1..10.0)));

// New: Real order book depth
// {"stream":"btcusdt@depth5","data":{"bids":[["45000","1.2"]],...}}
```

## Integration Benefits

### **Real Market Dynamics**
- **Authentic volatility patterns**
- **Real trading volume spikes**  
- **Actual market maker behavior**
- **Correlated price movements**

### **Performance Testing**
- **Variable message rates** (market hours vs. quiet periods)
- **Burst handling** (major news events)
- **Network latency** (real-world conditions)
- **Data quality issues** (missing messages, reconnections)

## Example Symbols
- **Crypto**: BTCUSDT, ETHUSDT, ADAUSDT, SOLUSDT
- **Volumes**: BTC (~50K msgs/day), ETH (~30K msgs/day)
- **Update frequency**: 1-10 Hz depending on market activity

## Testing Scenarios

### **1. Peak Hours Testing**
- US market open: 9:30-16:00 EST
- Crypto high activity: Evening US time  
- Expected: 5-20 messages/second

### **2. Quiet Period Testing**  
- Weekend crypto markets
- US overnight hours
- Expected: 0.1-2 messages/second

### **3. Volatility Testing**
- Major economic announcements
- Fed meetings, earnings releases
- Expected: 50-100+ messages/second bursts

## Performance Benchmarks

### **Target Metrics with Real Data**
- **Latency p95**: < 50ms (network + processing)
- **Throughput**: Handle 100+ msgs/sec bursts
- **Memory**: Stable under variable load
- **Reconnection**: < 5 second recovery time

### **Stress Test Conditions**
1. **Multiple symbols**: 5-10 concurrent streams
2. **Network instability**: Simulated disconnections
3. **Memory pressure**: Long-running sessions (hours)  
4. **CPU load**: Background tasks during trading

## Implementation Files

- `server/src/live_data.rs` - WebSocket client for external APIs
- `server/src/main.rs` - Integration with existing broadcast system
- `app/src/lib.rs` - Frontend message handling (no changes needed)

## Next Steps

1. **Create WebSocket client** for Binance streams
2. **Replace simulation loops** with real data handlers  
3. **Add error handling** for network issues
4. **Performance comparison** between simulated vs. real data
5. **Documentation update** with real-world testing procedures