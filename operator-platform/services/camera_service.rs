use anyhow::Result;
use sqlx::postgres::PgPool;
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

use crate::{
    models::{
        Camera, CameraStatus, CameraHealthStatus, CalibrationStatus, 
        CreateCameraRequest, UpdateCameraRequest, CameraCalibrationData,
        CalibrationRequest, CameraHealthMetrics, CameraStatusHistory, CameraZone
    },
    storage::file_storage::FileStorage,
};

#[derive(Clone)]
pub struct CameraService {
    db_pool: PgPool,
    file_storage: FileStorage,
}

impl CameraService {
    pub fn new(db_pool: PgPool, file_storage: FileStorage) -> Self {
        Self { db_pool, file_storage }
    }
    
    pub async fn get_all_cameras(&self) -> Result<Vec<Camera>> {
        let cameras = sqlx::query_as!(
            Camera,
            r#"
            SELECT * FROM cameras
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(cameras)
    }
    
    pub async fn get_camera_by_id(&self, id: Uuid) -> Result<Camera> {
        let camera = sqlx::query_as!(
            Camera,
            r#"
            SELECT * FROM cameras WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(camera)
    }
    
    pub async fn get_cameras_by_zone(&self, zone: &str) -> Result<Vec<Camera>> {
        let cameras = sqlx::query_as!(
            Camera,
            r#"
            SELECT * FROM cameras 
            WHERE zone = $1
            ORDER BY name
            "#,
            zone
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(cameras)
    }
    
    pub async fn get_cameras_by_status(&self, status: CameraStatus) -> Result<Vec<Camera>> {
        let cameras = sqlx::query_as!(
            Camera,
            r#"
            SELECT * FROM cameras 
            WHERE status = $1
            ORDER BY name
            "#,
            status as CameraStatus
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(cameras)
    }
    
    pub async fn create_camera(&self, data: CreateCameraRequest) -> Result<Camera> {
        let camera = sqlx::query_as!(
            Camera,
            r#"
            INSERT INTO cameras (
                name, description, device_id, location, zone, 
                stream_url, rtsp_url, fps, resolution_width, resolution_height,
                status, health_status, calibration_status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
            data.name,
            data.description,
            data.device_id,
            data.location,
            data.zone,
            data.stream_url,
            data.rtsp_url,
            data.fps,
            data.resolution_width,
            data.resolution_height,
            CameraStatus::Offline as CameraStatus,
            CameraHealthStatus::Unknown as CameraHealthStatus,
            CalibrationStatus::NotCalibrated as CalibrationStatus
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(camera)
    }
    
    pub async fn update_camera(&self, id: Uuid, data: UpdateCameraRequest) -> Result<Camera> {
        let camera = sqlx::query_as!(
            Camera,
            r#"
            UPDATE cameras 
            SET 
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                device_id = COALESCE($3, device_id),
                location = COALESCE($4, location),
                zone = COALESCE($5, zone),
                stream_url = COALESCE($6, stream_url),
                rtsp_url = COALESCE($7, rtsp_url),
                status = COALESCE($8, status),
                health_status = COALESCE($9, health_status),
                fps = COALESCE($10, fps),
                resolution_width = COALESCE($11, resolution_width),
                resolution_height = COALESCE($12, resolution_height),
                updated_at = $13
            WHERE id = $14
            RETURNING *
            "#,
            data.name,
            data.description,
            data.device_id,
            data.location,
            data.zone,
            data.stream_url,
            data.rtsp_url,
            data.status.map(|s| s as CameraStatus),
            data.health_status.map(|s| s as CameraHealthStatus),
            data.fps,
            data.resolution_width,
            data.resolution_height,
            Utc::now(),
            id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(camera)
    }
    
    pub async fn delete_camera(&self, id: Uuid) -> Result<()> {
        sqlx::query!(
            "DELETE FROM cameras WHERE id = $1",
            id
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn update_camera_status(&self, id: Uuid, status: CameraStatus, health_status: CameraHealthStatus) -> Result<Camera> {
        let camera = sqlx::query_as!(
            Camera,
            r#"
            UPDATE cameras 
            SET status = $1, health_status = $2, last_ping = $3, updated_at = $3
            WHERE id = $4
            RETURNING *
            "#,
            status as CameraStatus,
            health_status as CameraHealthStatus,
            Utc::now(),
            id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        // Log status change
        sqlx::query!(
            r#"
            INSERT INTO camera_status_history (camera_id, status, health_status, message)
            VALUES ($1, $2, $3, $4)
            "#,
            id,
            status as CameraStatus,
            health_status as CameraHealthStatus,
            "Status updated by system"
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(camera)
    }
    
    pub async fn save_calibration_data(
        &self, 
        camera_id: Uuid, 
        intrinsics: serde_json::Value, 
        extrinsics: serde_json::Value,
        calibration_method: &str,
        calibration_accuracy: f32,
        calibrated_by: Uuid,
        calibration_images: Vec<String>,
    ) -> Result<Camera> {
        let camera = sqlx::query_as!(
            Camera,
            r#"
            UPDATE cameras 
            SET 
                intrinsics = $1,
                extrinsics = $2,
                calibration_status = $3,
                last_calibration = $4,
                updated_at = $4
            WHERE id = $5
            RETURNING *
            "#,
            intrinsics,
            extrinsics,
            CalibrationStatus::Calibrated as CalibrationStatus,
            Utc::now(),
            camera_id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        // Save to calibration history
        sqlx::query!(
            r#"
            INSERT INTO camera_calibrations (
                camera_id, intrinsics, extrinsics, calibration_method, 
                calibration_accuracy, calibrated_by, calibration_images
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            camera_id,
            intrinsics,
            extrinsics,
            calibration_method,
            calibration_accuracy,
            calibrated_by,
            &calibration_images
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(camera)
    }
    
    pub async fn get_calibration_history(&self, camera_id: Uuid) -> Result<Vec<CameraCalibrationData>> {
        let calibrations = sqlx::query_as!(
            CameraCalibrationData,
            r#"
            SELECT 
                camera_id,
                intrinsics,
                extrinsics,
                calibration_method,
                calibration_accuracy,
                calibrated_at,
                calibrated_by,
                calibration_images
            FROM camera_calibrations
            WHERE camera_id = $1
            ORDER BY calibrated_at DESC
            "#,
            camera_id
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(calibrations)
    }
    
    pub async fn save_health_metrics(&self, metrics: CameraHealthMetrics) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO camera_health_metrics (
                camera_id, timestamp, fps, latency_ms, packet_loss,
                resolution_width, resolution_height, bitrate_kbps, cpu_usage, memory_usage
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            metrics.camera_id,
            metrics.timestamp,
            metrics.fps,
            metrics.latency_ms,
            metrics.packet_loss,
            metrics.resolution_width,
            metrics.resolution_height,
            metrics.bitrate_kbps,
            metrics.cpu_usage,
            metrics.memory_usage
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_health_metrics(&self, camera_id: Uuid, hours: i32) -> Result<Vec<CameraHealthMetrics>> {
        let metrics = sqlx::query_as!(
            CameraHealthMetrics,
            r#"
            SELECT 
                camera_id,
                timestamp,
                fps,
                latency_ms,
                packet_loss,
                resolution_width,
                resolution_height,
                bitrate_kbps,
                cpu_usage,
                memory_usage
            FROM camera_health_metrics
            WHERE camera_id = $1 AND timestamp >= NOW() - ($2 || ' hours')::INTERVAL
            ORDER BY timestamp DESC
            "#,
            camera_id,
            hours
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(metrics)
    }
    
    pub async fn get_status_history(&self, camera_id: Uuid, limit: Option<i64>) -> Result<Vec<CameraStatusHistory>> {
        let history = sqlx::query_as!(
            CameraStatusHistory,
            r#"
            SELECT 
                camera_id,
                status as "status: CameraStatus",
                health_status as "health_status: CameraHealthStatus",
                timestamp,
                message
            FROM camera_status_history
            WHERE camera_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
            camera_id,
            limit.unwrap_or(100)
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(history)
    }
    
    pub async fn get_camera_zones(&self) -> Result<Vec<CameraZone>> {
        let zones = sqlx::query_as!(
            CameraZone,
            r#"
            SELECT 
                z.id,
                z.name,
                z.description,
                z.location,
                COUNT(c.id) as camera_count,
                z.created_at,
                z.updated_at
            FROM camera_zones z
            LEFT JOIN cameras c ON z.name = c.zone
            GROUP BY z.id, z.name, z.description, z.location, z.created_at, z.updated_at
            ORDER BY z.name
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(zones)
    }
    
    pub async fn get_camera_stats(&self) -> Result<HashMap<String, i64>> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'online') as online,
                COUNT(*) FILTER (WHERE status = 'offline') as offline,
                COUNT(*) FILTER (WHERE status = 'error') as error,
                COUNT(*) FILTER (WHERE health_status = 'healthy') as healthy,
                COUNT(*) FILTER (WHERE health_status = 'warning') as warning,
                COUNT(*) FILTER (WHERE health_status = 'critical') as critical,
                COUNT(*) FILTER (WHERE calibration_status = 'calibrated') as calibrated,
                COUNT(*) FILTER (WHERE calibration_status = 'needs_recalibration') as needs_recalibration
            FROM cameras
            "#
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        let mut result = HashMap::new();
        result.insert("total".to_string(), stats.total.unwrap_or(0));
        result.insert("online".to_string(), stats.online.unwrap_or(0));
        result.insert("offline".to_string(), stats.offline.unwrap_or(0));
        result.insert("error".to_string(), stats.error.unwrap_or(0));
        result.insert("healthy".to_string(), stats.healthy.unwrap_or(0));
        result.insert("warning".to_string(), stats.warning.unwrap_or(0));
        result.insert("critical".to_string(), stats.critical.unwrap_or(0));
        result.insert("calibrated".to_string(), stats.calibrated.unwrap_or(0));
        result.insert("needs_recalibration".to_string(), stats.needs_recalibration.unwrap_or(0));
        
        Ok(result)
    }
    
    pub async fn start_calibration(&self, camera_id: Uuid, request: CalibrationRequest) -> Result<()> {
        // Update camera status to calibrating
        sqlx::query!(
            "UPDATE cameras SET calibration_status = $1, updated_at = $2 WHERE id = $3",
            CalibrationStatus::Calibrating as CalibrationStatus,
            Utc::now(),
            camera_id
        )
        .execute(&self.db_pool)
        .await?;
        
        // In a real implementation, this would trigger a calibration process
        // For now, we'll just simulate it by scheduling a background task
        
        Ok(())
    }
    
    pub async fn test_camera_connection(&self, camera_id: Uuid) -> Result<bool> {
        let camera = self.get_camera_by_id(camera_id).await?;
        
        // Try to connect to the camera stream
        // This is a simplified implementation
        let client = reqwest::Client::new();
        let timeout = std::time::Duration::from_secs(5);
        
        let result = tokio::time::timeout(timeout, async {
            if let Some(rtsp_url) = &camera.rtsp_url {
                // For RTSP streams, we'd use a specialized library
                // For now, we'll just check if the URL is accessible
                client.head(rtsp_url).send().await.is_ok()
            } else {
                client.head(&camera.stream_url).send().await.is_ok()
            }
        }).await;
        
        Ok(result.unwrap_or(false))
    }
}