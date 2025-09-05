use anyhow::Result;
use sqlx::postgres::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{SystemEvent, SystemEventType, EventSeverity, SystemHealth, ComponentHealth, SystemStatus, ComponentStatus, SystemMetrics, SystemStats};

#[derive(Clone)]
pub struct SystemService {
    db_pool: PgPool,
}

impl SystemService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
    
    pub async fn log_event(&self, event_type: SystemEventType, severity: EventSeverity, message: &str, source: Option<&str>, details: Option<serde_json::Value>) -> Result<SystemEvent> {
        let event = sqlx::query_as!(
            SystemEvent,
            r#"
            INSERT INTO system_events (event_type, severity, message, source, details)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            event_type as SystemEventType,
            severity as EventSeverity,
            message,
            source,
            details
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(event)
    }
    
    pub async fn get_events(&self, limit: Option<i64>, acknowledged: Option<bool>) -> Result<Vec<SystemEvent>> {
        let events = sqlx::query_as!(
            SystemEvent,
            r#"
            SELECT * FROM system_events
            WHERE ($1::boolean IS NULL OR acknowledged = $1)
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            acknowledged,
            limit.unwrap_or(100)
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(events)
    }
    
    pub async fn acknowledge_event(&self, event_id: Uuid, user_id: Uuid) -> Result<SystemEvent> {
        let event = sqlx::query_as!(
            SystemEvent,
            r#"
            UPDATE system_events 
            SET acknowledged = true, acknowledged_by = $1, acknowledged_at = $2
            WHERE id = $3
            RETURNING *
            "#,
            user_id,
            Utc::now(),
            event_id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(event)
    }
    
    pub async fn get_system_health(&self) -> Result<SystemHealth> {
        // Check database health
        let db_health = match sqlx::query!("SELECT 1 as test").fetch_one(&self.db_pool).await {
            Ok(_) => ComponentHealth {
                name: "database".to_string(),
                status: ComponentStatus::Ok,
                details: None,
            },
            Err(e) => ComponentHealth {
                name: "database".to_string(),
                status: ComponentStatus::Error,
                details: Some(serde_json::json!({ "error": e.to_string() })),
            },
        };
        
        // Check storage health (simplified)
        let storage_health = ComponentHealth {
            name: "storage".to_string(),
            status: ComponentStatus::Ok,
            details: None,
        };
        
        // Check camera health (simplified)
        let camera_health = ComponentHealth {
            name: "cameras".to_string(),
            status: ComponentStatus::Ok,
            details: None,
        };
        
        let components = vec![db_health, storage_health, camera_health];
        
        // Determine overall system status
        let status = if components.iter().any(|c| matches!(c.status, ComponentStatus::Error)) {
            SystemStatus::Unhealthy
        } else if components.iter().any(|c| matches!(c.status, ComponentStatus::Warning)) {
            SystemStatus::Degraded
        } else {
            SystemStatus::Healthy
        };
        
        Ok(SystemHealth {
            status,
            components,
            timestamp: Utc::now(),
        })
    }
    
    pub async fn get_system_metrics(&self) -> Result<SystemMetrics> {
        // In a real implementation, this would collect actual system metrics
        // For now, we'll return some mock data
        Ok(SystemMetrics {
            timestamp: Utc::now(),
            cpu_usage: 25.5,
            memory_usage: 45.2,
            disk_usage: 60.1,
            network_in: 10.2,
            network_out: 5.7,
            gpu_usage: Some(35.0),
            gpu_memory: Some(45.5),
        })
    }
    
    pub async fn get_system_stats(&self) -> Result<SystemStats> {
        let stats = sqlx::query_as!(
            SystemStats,
            r#"
            SELECT 
                (SELECT COUNT(*) FROM cameras) as total_cameras,
                (SELECT COUNT(*) FROM cameras WHERE status = 'online') as online_cameras,
                (SELECT COUNT(*) FROM models) as total_models,
                (SELECT COUNT(*) FROM models WHERE status = 'deployed') as deployed_models,
                (SELECT COUNT(*) FROM annotations) as total_annotations,
                (SELECT COUNT(*) FROM annotations WHERE status = 'completed') as completed_annotations,
                (SELECT COUNT(*) FROM training_jobs WHERE status IN ('pending', 'preparing', 'training', 'validating')) as active_training_jobs,
                EXTRACT(EPOCH FROM (NOW() - MIN(created_at))) as system_uptime
            FROM system_events
            "#
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(stats)
    }
    
    pub async fn get_unacknowledged_events_count(&self) -> Result<i64> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM system_events WHERE acknowledged = false"
        )
        .fetch_one(&self.db_pool)
        .await?
        .count
        .unwrap_or(0);
        
        Ok(count)
    }
}