use anyhow::Result;
use sqlx::postgres::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{TrainingJob, TrainingStatus, CreateTrainingJobRequest, UpdateTrainingJobRequest, TrainingJobStats, TrainingJobSummary};

#[derive(Clone)]
pub struct TrainingService {
    db_pool: PgPool,
}

impl TrainingService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
    
    pub async fn get_all_training_jobs(&self) -> Result<Vec<TrainingJob>> {
        let jobs = sqlx::query_as!(
            TrainingJob,
            r#"
            SELECT * FROM training_jobs
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(jobs)
    }
    
    pub async fn get_training_job(&self, id: Uuid) -> Result<TrainingJob> {
        let job = sqlx::query_as!(
            TrainingJob,
            r#"
            SELECT * FROM training_jobs WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(job)
    }
    
    pub async fn create_training_job(&self, user_id: Uuid, data: CreateTrainingJobRequest) -> Result<TrainingJob> {
        let job = sqlx::query_as!(
            TrainingJob,
            r#"
            INSERT INTO training_jobs (name, description, model_id, dataset_id, hyperparameters, status, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            data.name,
            data.description,
            data.model_id,
            data.dataset_id,
            data.hyperparameters,
            TrainingStatus::Pending as TrainingStatus,
            user_id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(job)
    }
    
    pub async fn update_training_job(&self, id: Uuid, data: UpdateTrainingJobRequest) -> Result<TrainingJob> {
        let job = sqlx::query_as!(
            TrainingJob,
            r#"
            UPDATE training_jobs 
            SET 
                status = COALESCE($1, status),
                progress = COALESCE($2, progress),
                metrics = COALESCE($3, metrics),
                val_metrics = COALESCE($4, val_metrics),
                logs = COALESCE($5, logs),
                started_at = CASE WHEN $1 = 'training' AND started_at IS NULL THEN $6 ELSE started_at END,
                completed_at = CASE WHEN $1 IN ('completed', 'failed', 'cancelled') AND completed_at IS NULL THEN $6 ELSE completed_at END,
                updated_at = $6
            WHERE id = $7
            RETURNING *
            "#,
            data.status.map(|s| s as TrainingStatus),
            data.progress,
            data.metrics,
            data.val_metrics,
            data.logs,
            Utc::now(),
            id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(job)
    }
    
    pub async fn delete_training_job(&self, id: Uuid) -> Result<()> {
        sqlx::query!(
            "DELETE FROM training_jobs WHERE id = $1",
            id
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_training_job_stats(&self) -> Result<TrainingJobStats> {
        let stats = sqlx::query_as!(
            TrainingJobStats,
            r#"
            SELECT 
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'pending') as pending,
                COUNT(*) FILTER (WHERE status = 'training') as training,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COUNT(*) FILTER (WHERE status = 'failed') as failed
            FROM training_jobs
            "#
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(stats)
    }
    
    pub async fn get_training_job_summaries(&self, limit: Option<i64>) -> Result<Vec<TrainingJobSummary>> {
        let summaries = sqlx::query_as!(
            TrainingJobSummary,
            r#"
            SELECT 
                id,
                name,
                model_id,
                status as "status: TrainingStatus",
                progress,
                created_at,
                completed_at
            FROM training_jobs
            ORDER BY created_at DESC
            LIMIT $1
            "#,
            limit.unwrap_or(50)
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(summaries)
    }
    
    pub async fn add_training_log(&self, id: Uuid, log: &str) -> Result<TrainingJob> {
        let job = sqlx::query_as!(
            TrainingJob,
            r#"
            UPDATE training_jobs 
            SET logs = array_append(logs, $1), updated_at = $2
            WHERE id = $3
            RETURNING *
            "#,
            log,
            Utc::now(),
            id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(job)
    }
    
    pub async fn get_active_training_jobs(&self) -> Result<Vec<TrainingJob>> {
        let jobs = sqlx::query_as!(
            TrainingJob,
            r#"
            SELECT * FROM training_jobs
            WHERE status IN ('pending', 'preparing', 'training', 'validating')
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(jobs)
    }
}