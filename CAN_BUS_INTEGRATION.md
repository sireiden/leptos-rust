# 🚗 CAN Bus Integration Guide

## 📊 CAN Bus Performance Charakteristika

### **Message Rates pro CAN Bus:**
```
Low-Speed CAN (125 kbps):     50-200 msg/s
High-Speed CAN (500 kbps):    200-800 msg/s  
CAN-FD (1-8 Mbps):           500-2000+ msg/s
```

### **Typische Multi-Bus Automotive Szenarien:**
```rust
struct AutomotiveCANSetup {
    powertrain_bus: CANBus,    // 500 kbps, 300-500 msg/s
    body_bus: CANBus,          // 125 kbps, 50-150 msg/s  
    infotainment_bus: CANBus,  // 500 kbps, 100-300 msg/s
    diagnostic_bus: CANBus,    // 250 kbps, 10-50 msg/s
    // Total: 460-1000 msg/s
}

struct IndustrialCANSetup {
    machine_control: CANBus,   // 1 Mbps, 200-800 msg/s
    sensor_network: CANBus,    // 500 kbps, 100-400 msg/s
    safety_bus: CANBus,        // 250 kbps, 50-200 msg/s
    // Total: 350-1400 msg/s
}
```

## 🔧 CAN Message Structure

### **Standard CAN Frame:**
```rust
#[derive(Clone, Debug)]
struct CANMessage {
    id: u32,           // 11-bit (Standard) oder 29-bit (Extended)
    data: Vec<u8>,     // 0-8 bytes payload
    timestamp: u64,    // Hardware timestamp (µs precision)
    bus_id: u8,        // Which CAN bus (0-7)
    dlc: u8,          // Data Length Code
    flags: CANFlags,   // RTR, Error, etc.
}

#[derive(Clone, Debug)]
struct CANFlags {
    extended_id: bool,    // 29-bit ID
    remote_frame: bool,   // RTR frame
    error_frame: bool,    // Error indication
    fd_frame: bool,       // CAN-FD frame
}
```

### **CAN Data Interpretation:**
```rust
// Beispiel: Engine RPM (ID 0x201)
fn decode_engine_rpm(data: &[u8]) -> Option<f32> {
    if data.len() >= 2 {
        let raw = u16::from_be_bytes([data[0], data[1]]);
        Some(raw as f32 * 0.25) // RPM = raw * 0.25
    } else {
        None
    }
}

// Beispiel: Vehicle Speed (ID 0x300) 
fn decode_vehicle_speed(data: &[u8]) -> Option<f32> {
    if data.len() >= 2 {
        let raw = u16::from_be_bytes([data[0], data[1]]);
        Some(raw as f32 * 0.01) // km/h = raw * 0.01
    } else {
        None
    }
}
```

## 🎯 Performance Optimizations für CAN

### **1. Message Filtering:**
```rust
// Nur relevante CAN IDs verarbeiten
const MONITORED_IDS: &[u32] = &[
    0x201, // Engine RPM
    0x300, // Vehicle Speed  
    0x400, // Engine Temperature
    0x500, // Battery Voltage
];

fn should_process_message(id: u32) -> bool {
    MONITORED_IDS.contains(&id)
}
```

### **2. Burst Handling:**
```rust
// CAN Nachrichten kommen oft in Bursts
struct CANBatcher {
    buffer: Vec<CANMessage>,
    last_flush: Instant,
    max_batch_size: usize,
    max_batch_time: Duration,
}

impl CANBatcher {
    fn add_message(&mut self, msg: CANMessage) {
        self.buffer.push(msg);
        
        if self.buffer.len() >= self.max_batch_size || 
           self.last_flush.elapsed() >= self.max_batch_time {
            self.flush();
        }
    }
    
    fn flush(&mut self) {
        // Sende batch an WebSocket
        send_batch(&self.buffer);
        self.buffer.clear();
        self.last_flush = Instant::now();
    }
}
```

### **3. Timestamp Precision:**
```rust
// CAN braucht µs-genaue Timestamps für Timing-Analyse
#[derive(Clone)]
struct PreciseCANMessage {
    id: u32,
    data: Vec<u8>, 
    hw_timestamp: u64,    // Hardware timestamp (µs)
    sw_timestamp: u64,    // Software timestamp (µs)
    bus_id: u8,
}
```

## 🚀 Integration ins aktuelle System

Soll ich das aktuelle WebSocket System erweitern für:

1. **CAN Message Types** hinzufügen
2. **Multi-Bus Support** (4-8 parallel CAN Busse)  
3. **CAN-spezifische UI** (Signal-Dekodierung, Bus-Load)
4. **Burst-optimierte Batching** 
5. **Hardware Timestamp Präzision**

**Welche CAN Bus Konfiguration nutzt ihr?**
- Anzahl CAN Busse: 1-8?
- Bus Speed: 125k/250k/500k/1M bps?
- Message Rate pro Bus: ~500/s?
- Specific Vehicle/Industrial Protocol (J1939, CANopen, etc.)?