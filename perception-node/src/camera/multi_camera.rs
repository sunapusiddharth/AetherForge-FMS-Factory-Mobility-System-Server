use super::{Camera, CameraFrame};
use crate::error::Result;
use aetherforge_common::CameraConfig;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

pub struct MultiCameraManager {
    cameras: DashMap<String, Arc<dyn Camera>>,
    frame_receivers: DashMap<String, mpsc::Receiver<CameraFrame>>,
    metrics: Arc<crate::utils::metrics::Metrics>,
}

impl MultiCameraManager {
    pub async fn new(configs: Vec<CameraConfig>, metrics: Arc<crate::utils::metrics::Metrics>) -> Result<Self> {
        let cameras = DashMap::new();
        let frame_receivers = DashMap::new();
        
        for config in configs {
            if !config.enabled {
                info!("Camera {} is disabled, skipping", config.id);
                continue;
            }
            
            match Self::create_camera(config, metrics.clone()).await {
                Ok((camera, receiver)) => {
                    cameras.insert(camera.get_id().to_string(), Arc::new(camera));
                    frame_receivers.insert(camera.get_id().to_string(), receiver);
                    info!("Camera {} initialized successfully", camera.get_id());
                }
                Err(e) => {
                    error!("Failed to initialize camera {}: {}", config.id, e);
                }
            }
        }
        
        Ok(Self {
            cameras,
            frame_receivers,
            metrics,
        })
    }
    
    async fn create_camera(config: CameraConfig, metrics: Arc<crate::utils::metrics::Metrics>) 
        -> Result<(Box<dyn Camera>, mpsc::Receiver<CameraFrame>)> 
    {
        use super::gstreamer::GStreamerCamera;
        
        let camera = GStreamerCamera::new(config, metrics).await?;
        let receiver = camera.get_frame_receiver().ok_or_else(|| {
            crate::error::PerceptionError::CameraError("Failed to get frame receiver".to_string())
        })?;
        
        Ok((Box::new(camera), receiver))
    }
    
    pub fn get_camera(&self, camera_id: &str) -> Option<Arc<dyn Camera>> {
        self.cameras.get(camera_id).map(|c| c.value().clone())
    }
    
    pub fn get_frame_receiver(&self, camera_id: &str) -> Option<mpsc::Receiver<CameraFrame>> {
        self.frame_receivers.get(camera_id).map(|r| r.value().clone())
    }
    
    pub fn list_cameras(&self) -> Vec<String> {
        self.cameras.iter().map(|c| c.key().clone()).collect()
    }
    
    pub async fn start_all(&self) -> Result<()> {
        for camera in self.cameras.iter() {
            if let Err(e) = camera.value().start().await {
                error!("Failed to start camera {}: {}", camera.key(), e);
            }
        }
        Ok(())
    }
    
    pub async fn stop_all(&self) -> Result<()> {
        for camera in self.cameras.iter() {
            if let Err(e) = camera.value().stop().await {
                error!("Failed to stop camera {}: {}", camera.key(), e);
            }
        }
        Ok(())
    }
    
    pub fn get_health_status(&self) -> HashMap<String, CameraHealthStatus> {
        let mut status = HashMap::new();
        
        for camera in self.cameras.iter() {
            status.insert(camera.key().clone(), camera.value().get_health_status());
        }
        
        status
    }
}

#[async_trait::async_trait]
pub trait CameraManager {
    async fn start_all(&self) -> Result<()>;
    async fn stop_all(&self) -> Result<()>;
    fn get_camera(&self, camera_id: &str) -> Option<Arc<dyn Camera>>;
    fn list_cameras(&self) -> Vec<String>;
    fn get_health_status(&self) -> HashMap<String, CameraHealthStatus>;
}