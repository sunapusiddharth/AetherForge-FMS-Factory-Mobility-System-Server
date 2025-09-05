use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Camera {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub device_id: String,
    pub location: String,
    pub zone: Option<String>,
    pub stream_url: String,
    pub rtsp_url: Option<String>,
    pub status: CameraStatus,
    pub health_status: CameraHealthStatus,
    pub last_ping: Option<DateTime<Utc>>,
    pub fps: Option<f32>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
    pub intrinsics: Option<serde_json::Value>,
    pub extrinsics: Option<serde_json::Value>,
    pub calibration_status: CalibrationStatus,
    pub last_calibration: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "camera_status", rename_all = "lowercase")]
pub enum CameraStatus {
    Online,
    Offline,
    Calibrating,
    Maintenance,
    Error,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "camera_health_status", rename_all = "lowercase")]
pub enum CameraHealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "calibration_status", rename_all = "lowercase")]
pub enum CalibrationStatus {
    NotCalibrated,
    Calibrating,
    Calibrated,
    NeedsRecalibration,
    Failed,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCameraRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    pub description: Option<String>,
    
    #[validate(length(min = 1))]
    pub device_id: String,
    
    #[validate(length(min = 1))]
    pub location: String,
    
    pub zone: Option<String>,
    
    #[validate(url)]
    pub stream_url: String,
    
    #[validate(url)]
    pub rtsp_url: Option<String>,
    
    pub fps: Option<f32>,
    
    pub resolution_width: Option<i32>,
    
    pub resolution_height: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCameraRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    
    pub description: Option<String>,
    
    #[validate(length(min = 1))]
    pub device_id: Option<String>,
    
    #[validate(length(min = 1))]
    pub location: Option<String>,
    
    pub zone: Option<String>,
    
    #[validate(url)]
    pub stream_url: Option<String>,
    
    #[validate(url)]
    pub rtsp_url: Option<String>,
    
    pub status: Option<CameraStatus>,
    
    pub health_status: Option<CameraHealthStatus>,
    
    pub fps: Option<f32>,
    
    pub resolution_width: Option<i32>,
    
    pub resolution_height: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct CameraCalibrationData {
    pub camera_id: Uuid,
    pub intrinsics: serde_json::Value,
    pub extrinsics: serde_json::Value,
    pub calibration_method: String,
    pub calibration_accuracy: f32,
    pub calibrated_at: DateTime<Utc>,
    pub calibrated_by: Uuid,
    pub calibration_images: Vec<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CalibrationRequest {
    pub calibration_method: String,
    
    #[validate(range(min = 0, max = 1))]
    pub target_accuracy: Option<f32>,
    
    pub calibration_pattern: CalibrationPattern,
    
    pub pattern_width: i32,
    
    pub pattern_height: i32,
    
    pub square_size: f32,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "calibration_pattern", rename_all = "lowercase")]
pub enum CalibrationPattern {
    Chessboard,
    Circles,
    AsymmetricCircles,
}

#[derive(Debug, Serialize)]
pub struct CameraHealthMetrics {
    pub camera_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub fps: f32,
    pub latency_ms: f32,
    pub packet_loss: f32,
    pub resolution_width: i32,
    pub resolution_height: i32,
    pub bitrate_kbps: f32,
    pub cpu_usage: f32,
    pub memory_usage: f32,
}

#[derive(Debug, Serialize)]
pub struct CameraStatusHistory {
    pub camera_id: Uuid,
    pub status: CameraStatus,
    pub health_status: CameraHealthStatus,
    pub timestamp: DateTime<Utc>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CameraZone {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub location: String,
    pub camera_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}