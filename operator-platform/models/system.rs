use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SystemEvent {
    pub id: Uuid,
    pub event_type: SystemEventType,
    pub severity: EventSeverity,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub source: Option<String>,
    pub acknowledged: bool,
    pub acknowledged_by: Option<Uuid>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "system_event_type", rename_all = "snake_case")]
pub enum SystemEventType {
    CameraOffline,
    CameraError,
    InferenceError,
    TrainingError,
    StorageLow,
    MemoryHigh,
    CpuHigh,
    ServiceDown,
    ModelPerformanceDegraded,
    SecurityAlert,
    Other,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "event_severity", rename_all = "lowercase")]
pub enum EventSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Serialize)]
pub struct SystemHealth {
    pub status: SystemStatus,
    pub components: Vec<ComponentHealth>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: ComponentStatus,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "system_status", rename_all = "lowercase")]
pub enum SystemStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "component_status", rename_all = "lowercase")]
pub enum ComponentStatus {
    Ok,
    Warning,
    Error,
    Offline,
}

#[derive(Debug, Serialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub network_in: f32,
    pub network_out: f32,
    pub gpu_usage: Option<f32>,
    pub gpu_memory: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct SystemStats {
    pub total_cameras: i64,
    pub online_cameras: i64,
    pub total_models: i64,
    pub deployed_models: i64,
    pub total_annotations: i64,
    pub completed_annotations: i64,
    pub active_training_jobs: i64,
    pub system_uptime: i64,
}