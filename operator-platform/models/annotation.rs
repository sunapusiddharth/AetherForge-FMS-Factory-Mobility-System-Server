use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Annotation {
    pub id: Uuid,
    pub image_path: String,
    pub camera_id: Uuid,
    pub created_by: Uuid,
    pub annotations: serde_json::Value,
    pub status: AnnotationStatus,
    pub reviewed: bool,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "annotation_status", rename_all = "lowercase")]
pub enum AnnotationStatus {
    Pending,
    Completed,
    Rejected,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAnnotationRequest {
    pub image_path: String,
    pub camera_id: Uuid,
    pub annotations: serde_json::Value,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAnnotationRequest {
    pub annotations: Option<serde_json::Value>,
    pub status: Option<AnnotationStatus>,
    pub reviewed: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct AnnotationTask {
    pub id: Uuid,
    pub image_path: String,
    pub camera_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub annotation_count: i64,
}

#[derive(Debug, Serialize)]
pub struct AnnotationStats {
    pub total: i64,
    pub pending: i64,
    pub completed: i64,
    pub rejected: i64,
}