use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OperatorConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub storage: StorageConfig,
    pub ml: MLPipelineConfig,
    pub monitoring: MonitoringConfig,
    pub annotation: AnnotationConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub api_prefix: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connect_timeout_sec: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthConfig {
    pub secret_key: String,
    pub token_expiration: i64, // in hours
    pub password_hash_cost: u32,
    pub session_timeout_min: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub models_dir: PathBuf,
    pub annotations_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub max_upload_size: usize,
    pub retention_days: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MLPipelineConfig {
    pub training_queue: String,
    pub max_training_jobs: usize,
    pub default_hyperparameters: serde_json::Value,
    pub validation_split: f32,
    pub early_stopping_patience: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitoringConfig {
    pub health_check_interval_sec: u64,
    pub metrics_collection_interval_sec: u64,
    pub alert_retention_days: u32,
    pub performance_thresholds: PerformanceThresholds,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerformanceThresholds {
    pub cpu_warning: f32,
    pub cpu_critical: f32,
    pub memory_warning: f32,
    pub memory_critical: f32,
    pub disk_warning: f32,
    pub disk_critical: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnnotationConfig {
    pub default_annotation_tool: String,
    pub supported_formats: Vec<String>,
    pub auto_review_threshold: f32,
    pub min_annotations_per_image: u32,
}

impl Default for OperatorConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                cors_origins: vec!["http://localhost:3000".to_string()],
                api_prefix: "/api/v1".to_string(),
            },
            database: DatabaseConfig {
                url: "postgres://postgres:password@localhost/aetherforge".to_string(),
                max_connections: 5,
                connect_timeout_sec: 30,
            },
            auth: AuthConfig {
                secret_key: "default-secret-key-change-in-production".to_string(),
                token_expiration: 24,
                password_hash_cost: 12,
                session_timeout_min: 30,
            },
            storage: StorageConfig {
                data_dir: PathBuf::from("/var/lib/aetherforge/data"),
                models_dir: PathBuf::from("/var/lib/aetherforge/models"),
                annotations_dir: PathBuf::from("/var/lib/aetherforge/annotations"),
                temp_dir: PathBuf::from("/tmp/aetherforge"),
                max_upload_size: 100 * 1024 * 1024, // 100MB
                retention_days: 90,
            },
            ml: MLPipelineConfig {
                training_queue: "training_jobs".to_string(),
                max_training_jobs: 3,
                default_hyperparameters: serde_json::json!({
                    "batch_size": 16,
                    "epochs": 50,
                    "learning_rate": 0.001,
                }),
                validation_split: 0.2,
                early_stopping_patience: 10,
            },
            monitoring: MonitoringConfig {
                health_check_interval_sec: 60,
                metrics_collection_interval_sec: 30,
                alert_retention_days: 30,
                performance_thresholds: PerformanceThresholds {
                    cpu_warning: 70.0,
                    cpu_critical: 90.0,
                    memory_warning: 75.0,
                    memory_critical: 90.0,
                    disk_warning: 80.0,
                    disk_critical: 95.0,
                },
            },
            annotation: AnnotationConfig {
                default_annotation_tool: "labelstudio".to_string(),
                supported_formats: vec!["coco".to_string(), "yolo".to_string(), "pascalvoc".to_string()],
                auto_review_threshold: 0.95,
                min_annotations_per_image: 3,
            },
        }
    }
}