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
    pub max_batch_size: usize,
    pub inference_backend: InferenceBackend,
    pub class_names: Vec<String>,
    pub segmentation_model_path: Option<PathBuf>,
    pub robot_identification_model_path: Option<PathBuf>,
    pub pose_estimation_model_path: Option<PathBuf>,
    pub max_batch_size: usize,
    pub batch_timeout_ms: u64,
    pub enable_dynamic_batching: bool,
    pub model_warmup: bool,
    pub model_cache_size: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InferenceBackend {
    Cpu,
    Cuda,
    TensorRT,
    OpenVINO,
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
    pub fallback_config: Option<Box<MessagingConfig>>,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
    pub enable_compression: bool,
    pub compression_level: u32,
    pub message_timeout_ms: u64,
    pub enable_heartbeats: bool,
    pub heartbeat_interval_sec: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MessagingProtocol {
    ZeroMQ,
    Redis,
    Kafka,
    MQTT,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CompressionType {
    None,
    Zstd,
    Lz4,
    Gzip,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessingConfig {
    pub max_queue_size: usize,
    pub num_worker_threads: usize,
    pub enable_batch_processing: bool,
    pub batch_timeout_ms: u64,
    pub enable_data_fusion: bool,
    pub fusion_algorithm: FusionAlgorithm,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FusionAlgorithm {
    EarlyFusion,
    LateFusion,
    WeightedAverage,
    Bayesian,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitoringConfig {
    pub enable_metrics: bool,
    pub metrics_port: u16,
    pub health_check_interval_sec: u64,
    pub performance_metrics_interval_sec: u64,
    pub enable_alerting: bool,
    pub alert_endpoints: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub enable_file_logging: bool,
    pub log_file_path: PathBuf,
    pub max_log_files: usize,
    pub max_file_size_mb: u64,
    pub enable_structured_logging: bool,
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
            max_batch_size: 8,
            inference_backend: InferenceBackend::Cuda,
            class_names: vec![
                "person".to_string(),
                "robot".to_string(),
                "pallet".to_string(),
                "forklift".to_string(),
                "obstacle".to_string(),
            ],
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
        }
    }
}