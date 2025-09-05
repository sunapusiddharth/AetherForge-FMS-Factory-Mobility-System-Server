use anyhow::{anyhow, Result};
use async_trait::async_trait;
use gstreamer::prelude::*;
use gstreamer_app::AppSink;
use gstreamer_video::VideoInfo;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::config::CameraConfig;

#[derive(Debug, Clone)]
pub struct CameraFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub timestamp: u64,
    pub sequence_num: u64,
}

#[async_trait]
pub trait Camera {
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    fn get_frame_rx(&self) -> Option<tokio::sync::mpsc::Receiver<CameraFrame>>;
    fn get_config(&self) -> &CameraConfig;
}

pub mod gstreamer_camera;