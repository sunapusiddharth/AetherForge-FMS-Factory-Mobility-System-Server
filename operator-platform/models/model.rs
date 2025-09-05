use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Model {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub model_path: String,
    pub model_type: ModelType,
    pub input_shape: serde_json::Value,
    pub output_shape: serde_json::Value,
    pub classes: serde_json::Value,
    pub performance_metrics: serde_json::Value,
    pub training_job_id: Option<Uuid>,
    pub status: ModelStatus,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "model_type", rename_all = "lowercase")]
pub enum ModelType {
    ObjectDetection,
    SemanticSegmentation,
    InstanceSegmentation,
    Classification,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "model_status", rename_all = "lowercase")]
pub enum ModelStatus {
    Draft,
    Training,
    Trained,
    Validating,
    Validated,
    Deployed,
    Archived,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateModelRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    pub description: Option<String>,
    
    #[validate(length(min = 1))]
    pub version: String,
    
    pub model_type: ModelType,
    
    pub input_shape: serde_json::Value,
    
    pub output_shape: serde_json::Value,
    
    pub classes: serde_json::Value,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateModelRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    
    pub description: Option<String>,
    
    pub performance_metrics: Option<serde_json::Value>,
    
    pub status: Option<ModelStatus>,
}

#[derive(Debug, Serialize)]
pub struct ModelVersion {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub model_type: ModelType,
    pub status: ModelStatus,
    pub created_at: DateTime<Utc>,
    pub performance_metrics: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ModelDeployment {
    pub id: Uuid,
    pub model_id: Uuid,
    pub deployed_to: String,
    pub status: DeploymentStatus,
    pub deployed_at: DateTime<Utc>,
    pub deployed_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "deployment_status", rename_all = "lowercase")]
pub enum DeploymentStatus {
    Pending,
    Deploying,
    Active,
    Failed,
    Retiring,
    Retired,
}