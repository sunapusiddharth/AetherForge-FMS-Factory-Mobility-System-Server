use anyhow::Result;
use sqlx::postgres::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{Annotation, AnnotationStatus, CreateAnnotationRequest, UpdateAnnotationRequest, AnnotationStats, AnnotationTask};

#[derive(Clone)]
pub struct AnnotationService {
    db_pool: PgPool,
}

impl AnnotationService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
    
    pub async fn get_annotation(&self, id: Uuid) -> Result<Annotation> {
        let annotation = sqlx::query_as!(
            Annotation,
            r#"
            SELECT * FROM annotations WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(annotation)
    }
    
    pub async fn get_annotations_by_camera(&self, camera_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Annotation>> {
        let annotations = sqlx::query_as!(
            Annotation,
            r#"
            SELECT * FROM annotations 
            WHERE camera_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            camera_id,
            limit.unwrap_or(100),
            offset.unwrap_or(0)
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(annotations)
    }
    
    pub async fn create_annotation(&self, user_id: Uuid, data: CreateAnnotationRequest) -> Result<Annotation> {
        let annotation = sqlx::query_as!(
            Annotation,
            r#"
            INSERT INTO annotations (image_path, camera_id, created_by, annotations, status)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            data.image_path,
            data.camera_id,
            user_id,
            data.annotations,
            AnnotationStatus::Pending as AnnotationStatus
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(annotation)
    }
    
    pub async fn update_annotation(&self, id: Uuid, user_id: Uuid, data: UpdateAnnotationRequest) -> Result<Annotation> {
        let annotation = sqlx::query_as!(
            Annotation,
            r#"
            UPDATE annotations 
            SET 
                annotations = COALESCE($1, annotations),
                status = COALESCE($2, status),
                reviewed = COALESCE($3, reviewed),
                reviewed_by = CASE WHEN $3 = true THEN $4 ELSE reviewed_by END,
                reviewed_at = CASE WHEN $3 = true THEN $5 ELSE reviewed_at END,
                updated_at = $5
            WHERE id = $6
            RETURNING *
            "#,
            data.annotations,
            data.status.map(|s| s as AnnotationStatus),
            data.reviewed,
            user_id,
            Utc::now(),
            id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(annotation)
    }
    
    pub async fn delete_annotation(&self, id: Uuid) -> Result<()> {
        sqlx::query!(
            "DELETE FROM annotations WHERE id = $1",
            id
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_annotation_stats(&self) -> Result<AnnotationStats> {
        let stats = sqlx::query_as!(
            AnnotationStats,
            r#"
            SELECT 
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'pending') as pending,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COUNT(*) FILTER (WHERE status = 'rejected') as rejected
            FROM annotations
            "#
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(stats)
    }
    
    pub async fn get_annotation_tasks(&self, limit: Option<i64>) -> Result<Vec<AnnotationTask>> {
        let tasks = sqlx::query_as!(
            AnnotationTask,
            r#"
            SELECT 
                a.id,
                a.image_path,
                a.camera_id,
                a.created_at,
                COUNT(ann.id) as annotation_count
            FROM annotation_tasks a
            LEFT JOIN annotations ann ON a.id = ann.task_id
            GROUP BY a.id, a.image_path, a.camera_id, a.created_at
            ORDER BY a.created_at DESC
            LIMIT $1
            "#,
            limit.unwrap_or(50)
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(tasks)
    }
    
    pub async fn export_annotations(&self, format: &str) -> Result<Vec<u8>> {
        // This would export annotations in the specified format (COCO, YOLO, etc.)
        // For now, we'll just return a simple CSV format
        let annotations = sqlx::query!(
            r#"
            SELECT 
                image_path,
                camera_id,
                annotations,
                created_at
            FROM annotations
            WHERE status = 'completed'
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        let mut csv_data = Vec::new();
        let mut wtr = csv::Writer::from_writer(&mut csv_data);
        
        wtr.write_record(&["image_path", "camera_id", "annotations", "created_at"])?;
        
        for ann in annotations {
            wtr.write_record(&[
                ann.image_path,
                ann.camera_id.to_string(),
                ann.annotations.to_string(),
                ann.created_at.to_string(),
            ])?;
        }
        
        wtr.flush()?;
        
        Ok(csv_data)
    }
}