// 🚀 Multi-User Measurement System - Core Types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// User Session Management
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: Uuid,
    pub session_id: Uuid, 
    pub username: String,
    pub created_at: u64,
    pub last_activity: u64,
    pub active_measurements: Vec<MeasurementId>,
    pub resource_usage: ResourceUsage,
    pub permissions: UserPermissions,
}

/// Measurement Instance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeasurementSession {
    pub measurement_id: MeasurementId,
    pub user_id: Uuid,
    pub name: String,
    pub measurement_type: MeasurementType,
    pub config: MeasurementConfig,
    pub status: MeasurementStatus,
    pub started_at: u64,
    pub sample_count: u64,
    pub last_sample_time: u64,
}

pub type MeasurementId = Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MeasurementType {
    Voltage,
    Current, 
    Temperature,
    Pressure,
    Acceleration,
    Custom { name: String, unit: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeasurementConfig {
    pub sample_rate_hz: u32,        // 1 Hz - 100 kHz
    pub duration_seconds: Option<u32>, // None = continuous
    pub channels: Vec<ChannelConfig>,
    pub trigger_config: Option<TriggerConfig>,
    pub auto_scale: bool,
    pub data_retention: DataRetention,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub channel_id: u8,
    pub name: String,
    pub unit: String,
    pub range_min: f64,
    pub range_max: f64,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]  
pub enum MeasurementStatus {
    Configuring,
    Starting,
    Running,
    Paused,
    Stopping,
    Completed,
    Error { message: String },
}

/// Real-time Measurement Data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeasurementData {
    pub measurement_id: MeasurementId,
    pub timestamp_ns: u64,          // Nanosecond precision
    pub sample_index: u64,
    pub channels: Vec<ChannelData>,
    pub metadata: SampleMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelData {
    pub channel_id: u8,
    pub value: f64,
    pub quality: DataQuality,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DataQuality {
    Good,
    Questionable,
    Bad,
    Overflow,
    Underflow,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SampleMetadata {
    pub trigger_events: Vec<TriggerEvent>,
    pub system_events: Vec<SystemEvent>,
}

/// Resource Management  
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub bandwidth_bps: u64,
    pub storage_mb: u64,
    pub active_measurements: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPermissions {
    pub max_concurrent_measurements: u8,
    pub max_sample_rate_hz: u32,
    pub max_bandwidth_bps: u64,
    pub max_storage_mb: u64,
    pub max_session_duration_hours: u32,
    pub allowed_measurement_types: Vec<MeasurementType>,
}

/// System-wide Statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemStats {
    pub total_users: u32,
    pub active_users: u32,
    pub total_measurements: u32,
    pub active_measurements: u32,
    pub total_samples_per_second: u32,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: u64,
    pub network_throughput_bps: u64,
}

/// WebSocket Messages
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MultiUserMessage {
    // Session Management
    UserLogin { username: String, password: String },
    UserLogout,
    SessionHeartbeat,
    
    // Measurement Control
    StartMeasurement { config: MeasurementConfig },
    StopMeasurement { measurement_id: MeasurementId },
    PauseMeasurement { measurement_id: MeasurementId },
    ResumeMeasurement { measurement_id: MeasurementId },
    
    // Real-time Data
    MeasurementData(MeasurementData),
    MeasurementBatch { measurements: Vec<MeasurementData> },
    
    // System Status
    SystemStats(SystemStats),
    UserStats(ResourceUsage),
    MeasurementStatus { 
        measurement_id: MeasurementId, 
        status: MeasurementStatus 
    },
    
    // Errors & Events
    Error { message: String, error_code: u32 },
    ResourceLimitExceeded { resource: String, limit: u64 },
    SystemMaintenance { message: String, eta_seconds: u32 },
}

/// Kernel Interface Types
#[derive(Clone, Debug)]
pub struct KernelCommand {
    pub user_id: Uuid,
    pub measurement_id: MeasurementId,
    pub command: KernelCommandType,
}

#[derive(Clone, Debug)]
pub enum KernelCommandType {
    StartMeasurement(MeasurementConfig),
    StopMeasurement,
    PauseMeasurement,
    ResumeMeasurement,
    UpdateConfig(MeasurementConfig),
}

/// High-Performance Data Structures
#[repr(C)]
pub struct SharedMemoryHeader {
    pub magic: u32,                 // Validation
    pub version: u32,
    pub total_size: u64,
    pub write_offset: u64,
    pub read_offset: u64,
    pub measurement_count: u32,
    pub sample_rate: u32,
}

#[repr(C)]
pub struct KernelSample {
    pub measurement_id: [u8; 16],   // UUID as bytes
    pub timestamp_ns: u64,
    pub sample_index: u64,
    pub channel_count: u8,
    pub data: [f64; 16],           // Max 16 channels
}

/// Trigger System
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TriggerConfig {
    pub trigger_type: TriggerType,
    pub channel_id: u8,
    pub threshold: f64,
    pub hysteresis: f64,
    pub pre_trigger_samples: u32,
    pub post_trigger_samples: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TriggerType {
    Rising,
    Falling,
    Both,
    Level,
    Window { min: f64, max: f64 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TriggerEvent {
    pub trigger_type: TriggerType,
    pub timestamp_ns: u64,
    pub channel_id: u8,
    pub trigger_value: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SystemEvent {
    MeasurementStarted,
    MeasurementStopped,  
    DataOverrun,
    TimestampJump,
    ResourceWarning,
}