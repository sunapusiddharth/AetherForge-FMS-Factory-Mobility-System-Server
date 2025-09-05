use anyhow::Result;
use sqlx::postgres::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{Model, ModelType, ModelStatus, CreateModelRequest, UpdateModelRequest, ModelVersion, ModelDeployment, DeploymentStatus};

#[derive(Clone)]
pub struct ModelService {
    db_pool: PgPool,
}

impl ModelService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
    
    pub async fn get_all_models(&self) -> Result<Vec<Model>> {
        let models = sqlx::query_as!(
            Model,
            r#"
            SELECT * FROM models
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(models)
    }
    
    pub async fn get_model(&self, id: Uuid) -> Result<Model> {
        let model = sqlx::query_as!(
            Model,
            r#"
            SELECT * FROM models WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(model)
    }
    
    pub async fn get_model_versions(&self, name: &str) -> Result<Vec<ModelVersion>> {
        let versions = sqlx::query_as!(
            ModelVersion,
            r#"
            SELECT 
                id,
                name,
                version,
                model_type as "model_type: ModelType",
                status as "status: ModelStatus",
                created_at,
                performance_metrics
            FROM models
            WHERE name = $1
            ORDER BY created_at DESC
            "#,
            name
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(versions)
    }
    
    pub async fn create_model(&self, user_id: Uuid, data: CreateModelRequest) -> Result<Model> {
        let model = sqlx::query_as!(
            Model,
            r#"
            INSERT INTO models (name, description, version, model_type, input_shape, output_shape, classes, created_by, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            data.name,
            data.description,
            data.version,
            data.model_type as ModelType,
            data.input_shape,
            data.output_shape,
            data.classes,
            user_id,
            ModelStatus::Draft as ModelStatus
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(model)
    }
    
    pub async fn update_model(&self, id: Uuid, data: UpdateModelRequest) -> Result<Model> {
        let model = sqlx::query_as!(
            Model,
            r#"
            UPDATE models 
            SET 
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                performance_metrics = COALESCE($3, performance_metrics),
                status = COALESCE($4, status),
                updated_at = $5
            WHERE id = $6
            RETURNING *
            "#,
            data.name,
            data.description,
            data.performance_metrics,
            data.status.map(|s| s as ModelStatus),
            Utc::now(),
            id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(model)
    }
    
    pub async fn delete_model(&self, id: Uuid) -> Result<()> {
        sqlx::query!(
            "DELETE FROM models WHERE id = $1",
            id
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn deploy_model(&self, model_id: Uuid, deployed_to: &str, user_id: Uuid) -> Result<ModelDeployment> {
        let deployment = sqlx::query_as!(
            ModelDeployment,
            r#"
            INSERT INTO model_deployments (model_id, deployed_to, status, deployed_by)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            model_id,
            deployed_to,
            DeploymentStatus::Pending as DeploymentStatus,
            user_id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        // Update model status
        sqlx::query!(
            "UPDATE models SET status = $1 WHERE id = $2",
            ModelStatus::Deployed as ModelStatus,
            model_id
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(deployment)
    }
    
    pub async fn get_model_deployments(&self, model_id: Uuid) -> Result<Vec<ModelDeployment>> {
        let deployments = sqlx::query_as!(
            ModelDeployment,
            r#"
            SELECT * FROM model_deployments
            WHERE model_id = $1
            ORDER BY deployed_at DESC
            "#,
            model_id
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(deployments)
    }
    
    pub async fn update_deployment_status(&self, deployment_id: Uuid, status: DeploymentStatus) -> Result<ModelDeployment> {
        let deployment = sqlx::query_as!(
            ModelDeployment,
            r#"
            UPDATE model_deployments 
            SET status = $1
            WHERE id = $2
            RETURNING *
            "#,
            status as DeploymentStatus,
            deployment_id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(deployment)
    }
}