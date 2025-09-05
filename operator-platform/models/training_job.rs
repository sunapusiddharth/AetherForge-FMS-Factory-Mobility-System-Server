use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TrainingJob {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub model_id: Uuid,
    pub dataset_id: Uuid,
    pub hyperparameters: serde_json::Value,
    pub status: TrainingStatus,
    pub progress: f32,
    pub metrics: serde_json::Value,
    val_metrics: serde_json::Value,
    pub logs: Vec<String>,
    pub created_by: Uuid,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "training_status", rename_all = "lowercase")]
pub enum TrainingStatus {
    Pending,
    Preparing,
    Training,
    Validating,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTrainingJobRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    pub description: Option<String>,
    
    pub model_id: Uuid,
    
    pub dataset_id: Uuid,
    
    pub hyperparameters: serde_json::Value,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTrainingJobRequest {
    pub status: Option<TrainingStatus>,
    
    pub progress: Option<f32>,
    
    pub metrics: Option<serde_json::Value>,
    
    pub val_metrics: Option<serde_json::Value>,
    
    pub logs: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct TrainingJobStats {
    pub total: i64,
    pub pending: i64,
    pub training: i64,
    pub completed: i64,
    pub failed: i64,
}

#[derive(Debug, Serialize)]
pub struct TrainingJobSummary {
    pub id: Uuid,
    pub name: String,
    pub model_id: Uuid,
    pub status: TrainingStatus,
    pub progress: f32,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}