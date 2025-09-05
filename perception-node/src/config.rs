use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerceptionConfig {
    pub node_id: String,
    pub cameras: Vec<CameraConfig>,
    pub inference: InferenceConfig,
    pub messaging: MessagingConfig,
    pub processing: ProcessingConfig,
    pub monitoring: MonitoringConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CameraConfig {
    pub id: String,
    pub name: String,
    pub source: String,
    pub width: u32,
    pub height: u32,
    pub framerate: u32,
    pub enabled: bool,
    pub calibration: Option<CameraCalibration>,
    pub roi: Option<RegionOfInterest>,
    pub rtsp_url: Option<String>,
    pub zone: Option<String>,
    pub health_check_interval_sec: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CameraCalibration {
    pub intrinsics: Intrinsics,
    pub extrinsics: Extrinsics,
    pub distortion: DistortionCoefficients,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Intrinsics {
    pub fx: f64,
    pub fy: f64,
    pub cx: f64,
    pub cy: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Extrinsics {
    pub rotation: [f64; 3],
    pub translation: [f64; 3],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DistortionCoefficients {
    pub k1: f64,
    pub k2: f64,
    pub p1: f64,
    pub p2: f64,
    pub k3: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegionOfInterest {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InferenceConfig {
    pub model_path: PathBuf,
    pub model_version: String,
    pub confidence_threshold: f32,
    pub nms_threshold: f32,
    pub input_width: u32,
    pub input_height: u32,
    pub use_gpu: bool,
    pub inference_backend: InferenceBackend,
    pub class_names: Vec<String>,
    
    // New additions
    pub segmentation_model_path: Option<PathBuf>,
    pub robot_identification_model_path: Option<PathBuf>,
    pub pose_estimation_model_path: Option<PathBuf>,
    pub max_batch_size: usize,
    pub batch_timeout_ms: u64,
    pub enable_dynamic_batching: bool,
    pub model_warmup: bool,
    pub model_cache_size: usize,
    pub gpu_memory_limit_mb: Option<u32>,
    pub enable_fp16: bool,
    pub enable_int8: bool,
    pub optimization_level: OptimizationLevel,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InferenceBackend {
    Cpu,
    Cuda,
    TensorRT,
    OpenVINO,
    DirectML,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OptimizationLevel {
    Disabled,
    Level1,
    Level2,
    Level3,
    Extreme,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessagingConfig {
    pub enabled: bool,
    pub protocol: MessagingProtocol,
    pub endpoint: String,
    pub topic: String,
    pub heartbeat_interval_sec: u64,
    pub max_queue_size: usize,
    pub compression: CompressionType,
    
    // New additions
    pub fallback_config: Option<Box<MessagingConfig>>,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
    pub enable_compression: bool,
    pub compression_level: u32,
    pub message_timeout_ms: u64,
    pub enable_heartbeats: bool,
    pub high_water_mark: u32,
    pub send_timeout_ms: i32,
    pub reconnect_interval_ms: i32,
    pub security: MessagingSecurity,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MessagingProtocol {
    ZeroMQ,
    Redis,
    Kafka,
    MQTT,
    ROS2,
    WebSocket,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CompressionType {
    None,
    Zstd,
    Lz4,
    Gzip,
    Brotli,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessagingSecurity {
    pub enable_authentication: bool,
    pub enable_encryption: bool,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ssl_cert_path: Option<PathBuf>,
    pub ssl_key_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessingConfig {
    pub max_queue_size: usize,
    pub num_worker_threads: usize,
    pub enable_batch_processing: bool,
    pub batch_timeout_ms: u64,
    pub enable_data_fusion: bool,
    pub fusion_algorithm: FusionAlgorithm,
    
    // New additions
    pub enable_tracking: bool,
    pub tracker_type: TrackerType,
    pub max_track_age: u32,
    pub min_detection_confidence: f32,
    pub enable_memory_optimization: bool,
    pub frame_skip_interval: u32,
    pub enable_roi_processing: bool,
    pub enable_multi_scale_processing: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FusionAlgorithm {
    EarlyFusion,
    LateFusion,
    WeightedAverage,
    Bayesian,
    DempsterShafer,
    KalmanFilter,
    ParticleFilter,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TrackerType {
    Sort,
    DeepSort,
    ByteTrack,
    IOU,
    Kalman,
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitoringConfig {
    pub enable_metrics: bool,
    pub metrics_port: u16,
    pub health_check_interval_sec: u64,
    pub performance_metrics_interval_sec: u64,
    pub enable_alerting: bool,
    pub alert_endpoints: Vec<String>,
    
    // New additions
    pub enable_profiling: bool,
    pub profile_output_path: PathBuf,
    pub enable_resource_monitoring: bool,
    pub resource_check_interval_sec: u64,
    pub enable_performance_counters: bool,
    pub enable_latency_tracking: bool,
    pub enable_throughput_monitoring: bool,
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlertThresholds {
    pub cpu_usage_warning: f32,
    pub cpu_usage_critical: f32,
    pub memory_usage_warning: f32,
    pub memory_usage_critical: f32,
    pub gpu_usage_warning: f32,
    pub gpu_usage_critical: f32,
    pub inference_latency_warning_ms: f32,
    pub inference_latency_critical_ms: f32,
    pub frame_processing_latency_warning_ms: f32,
    pub frame_processing_latency_critical_ms: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub enable_file_logging: bool,
    pub log_file_path: PathBuf,
    pub max_log_files: usize,
    pub max_file_size_mb: u64,
    pub enable_structured_logging: bool,
    
    // New additions
    pub enable_remote_logging: bool,
    pub remote_logging_endpoint: Option<String>,
    pub log_buffer_size: usize,
    pub enable_log_rotation: bool,
    pub log_rotation_interval: String,
    pub enable_audit_logging: bool,
    pub audit_log_path: PathBuf,
}

impl Default for PerceptionConfig {
    fn default() -> Self {
        Self {
            node_id: "perception-node-1".to_string(),
            cameras: vec![CameraConfig::default()],
            inference: InferenceConfig::default(),
            messaging: MessagingConfig::default(),
            processing: ProcessingConfig::default(),
            monitoring: MonitoringConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            id: "camera-1".to_string(),
            name: "Front Camera".to_string(),
            source: "v4l2src device=/dev/video0".to_string(),
            width: 640,
            height: 480,
            framerate: 30,
            enabled: true,
            calibration: None,
            roi: None,
            rtsp_url: None,
            zone: Some("production-line-1".to_string()),
            health_check_interval_sec: 30,
        }
    }
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/yolov5s.onnx"),
            model_version: "1.0".to_string(),
            confidence_threshold: 0.5,
            nms_threshold: 0.5,
            input_width: 640,
            input_height: 480,
            use_gpu: true,
            inference_backend: InferenceBackend::Cuda,
            class_names: vec![
                "person".to_string(),
                "robot".to_string(),
                "pallet".to_string(),
                "forklift".to_string(),
                "obstacle".to_string(),
            ],
            segmentation_model_path: None,
            robot_identification_model_path: None,
            pose_estimation_model_path: None,
            max_batch_size: 8,
            batch_timeout_ms: 100,
            enable_dynamic_batching: true,
            model_warmup: true,
            model_cache_size: 2,
            gpu_memory_limit_mb: Some(2048),
            enable_fp16: true,
            enable_int8: false,
            optimization_level: OptimizationLevel::Level3,
        }
    }
}

impl Default for MessagingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            protocol: MessagingProtocol::ZeroMQ,
            endpoint: "tcp://*:5555".to_string(),
            topic: "perception_frames".to_string(),
            heartbeat_interval_sec: 5,
            max_queue_size: 1000,
            compression: CompressionType::Zstd,
            fallback_config: None,
            retry_attempts: 3,
            retry_delay_ms: 100,
            enable_compression: true,
            compression_level: 3,
            message_timeout_ms: 1000,
            enable_heartbeats: true,
            high_water_mark: 1000,
            send_timeout_ms: 1000,
            reconnect_interval_ms: 100,
            security: MessagingSecurity::default(),
        }
    }
}

impl Default for MessagingSecurity {
    fn default() -> Self {
        Self {
            enable_authentication: false,
            enable_encryption: false,
            username: None,
            password: None,
            ssl_cert_path: None,
            ssl_key_path: None,
        }
    }
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 100,
            num_worker_threads: 4,
            enable_batch_processing: true,
            batch_timeout_ms: 100,
            enable_data_fusion: false,
            fusion_algorithm: FusionAlgorithm::LateFusion,
            enable_tracking: true,
            tracker_type: TrackerType::DeepSort,
            max_track_age: 30,
            min_detection_confidence: 0.3,
            enable_memory_optimization: true,
            frame_skip_interval: 0,
            enable_roi_processing: true,
            enable_multi_scale_processing: false,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            metrics_port: 9090,
            health_check_interval_sec: 30,
            performance_metrics_interval_sec: 5,
            enable_alerting: false,
            alert_endpoints: vec![],
            enable_profiling: false,
            profile_output_path: PathBuf::from("/var/log/aetherforge/profiles"),
            enable_resource_monitoring: true,
            resource_check_interval_sec: 10,
            enable_performance_counters: true,
            enable_latency_tracking: true,
            enable_throughput_monitoring: true,
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_usage_warning: 70.0,
            cpu_usage_critical: 90.0,
            memory_usage_warning: 75.0,
            memory_usage_critical: 90.0,
            gpu_usage_warning: 80.0,
            gpu_usage_critical: 95.0,
            inference_latency_warning_ms: 50.0,
            inference_latency_critical_ms: 100.0,
            frame_processing_latency_warning_ms: 100.0,
            frame_processing_latency_critical_ms: 200.0,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            enable_file_logging: true,
            log_file_path: PathBuf::from("/var/log/aetherforge/perception.log"),
            max_log_files: 10,
            max_file_size_mb: 100,
            enable_structured_logging: true,
            enable_remote_logging: false,
            remote_logging_endpoint: None,
            log_buffer_size: 1000,
            enable_log_rotation: true,
            log_rotation_interval: "daily".to_string(),
            enable_audit_logging: false,
            audit_log_path: PathBuf::from("/var/log/aetherforge/audit.log"),
        }
    }
}