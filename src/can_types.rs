// 🚗 CAN Bus Message Types für WebSocket Integration

use serde::{Deserialize, Serialize};

/// CAN Bus spezifische Message Typen
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CANMessage {
    /// Standard CAN Data Frame
    DataFrame {
        bus_id: u8,              // CAN Bus ID (0-7)
        can_id: u32,            // CAN Message ID (11 oder 29 bit)
        data: Vec<u8>,          // Payload (0-8 bytes)
        timestamp_us: u64,       // Hardware timestamp (Mikrosekunden)
        dlc: u8,                // Data Length Code
        extended: bool,         // 29-bit Extended ID
    },
    
    /// CAN Error Frame
    ErrorFrame {
        bus_id: u8,
        error_type: CANErrorType,
        timestamp_us: u64,
    },
    
    /// Bus Status Information
    BusStatus {
        bus_id: u8,
        load_percent: f32,      // Bus Load 0-100%
        error_count: u32,       // Error Counter
        messages_per_sec: u32,  // Aktuelle Message Rate
        timestamp_us: u64,
    },
    
    /// Dekodierte Fahrzeug-Signale
    VehicleSignal {
        signal_name: String,    // "engine_rpm", "vehicle_speed", etc.
        value: f64,            // Dekodierter Wert
        unit: String,          // "rpm", "km/h", "°C", etc.  
        bus_id: u8,
        source_id: u32,        // Original CAN ID
        timestamp_us: u64,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CANErrorType {
    StuffError,
    FormError,
    AckError,
    BitError,
    CrcError,
    BusOff,
    ErrorPassive,
    TxTimeout,
}

/// CAN Bus Konfiguration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CANBusConfig {
    pub bus_id: u8,
    pub name: String,           // "Powertrain", "Body", "Infotainment"
    pub bitrate: u32,          // 125000, 250000, 500000, 1000000
    pub fd_enabled: bool,       // CAN-FD Support
    pub listen_only: bool,      // Nur empfangen, nicht senden
}

/// Batch von CAN Messages für Performance
#[derive(Clone, Debug, Serialize, Deserialize)]  
pub struct CANMessageBatch {
    pub messages: Vec<CANMessage>,
    pub batch_id: u32,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
}

/// CAN Signal Database Entry (für Dekodierung)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CANSignal {
    pub name: String,           // "Engine_RPM"
    pub can_id: u32,           // CAN Message ID
    pub start_bit: u8,         // Start Bit im CAN Frame
    pub length: u8,            // Anzahl Bits
    pub byte_order: ByteOrder, // Big/Little Endian
    pub scale: f64,            // Skalierungsfaktor
    pub offset: f64,           // Offset
    pub unit: String,          // "rpm", "km/h"
    pub min_val: f64,          // Minimum Wert
    pub max_val: f64,          // Maximum Wert
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ByteOrder {
    BigEndian,
    LittleEndian,
}

/// Performance Monitoring für CAN Busse
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CANPerformanceStats {
    pub bus_id: u8,
    pub messages_per_second: f32,
    pub bytes_per_second: f32,
    pub bus_load_percent: f32,
    pub error_rate: f32,           // Errors per second
    pub max_latency_us: u64,       // Maximale Message Latenz
    pub avg_latency_us: u64,       // Durchschnittliche Latenz
}

// Beispiel: Typische Automotive CAN IDs
pub mod automotive_ids {
    pub const ENGINE_RPM: u32 = 0x201;
    pub const VEHICLE_SPEED: u32 = 0x300;
    pub const ENGINE_TEMP: u32 = 0x400;
    pub const BATTERY_VOLTAGE: u32 = 0x500;
    pub const GEAR_POSITION: u32 = 0x600;
    pub const STEERING_ANGLE: u32 = 0x700;
}

// Beispiel: Signal Dekodierung
impl CANSignal {
    /// Dekodiert einen Wert aus CAN Data
    pub fn decode(&self, data: &[u8]) -> Option<f64> {
        if data.is_empty() {
            return None;
        }
        
        // Vereinfachte Dekodierung für Demo
        match self.can_id {
            automotive_ids::ENGINE_RPM => {
                if data.len() >= 2 {
                    let raw = u16::from_be_bytes([data[0], data[1]]);
                    Some(raw as f64 * self.scale + self.offset)
                } else {
                    None
                }
            }
            automotive_ids::VEHICLE_SPEED => {
                if data.len() >= 2 {
                    let raw = u16::from_be_bytes([data[0], data[1]]);
                    Some(raw as f64 * 0.01) // km/h
                } else {
                    None
                }
            }
            _ => None
        }
    }
}