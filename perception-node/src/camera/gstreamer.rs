use anyhow::{anyhow, Result};
use gstreamer::prelude::*;
use gstreamer_app::AppSink;
use gstreamer_video::{VideoInfo, VideoFormat};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use super::{Camera, CameraFrame};
use crate::config::CameraConfig;

pub struct GStreamerCamera {
    config: CameraConfig,
    pipeline: Option<gstreamer::Pipeline>,
    main_loop: Option<glib::MainLoop>,
    frame_tx: Option<mpsc::Sender<CameraFrame>>,
    frame_rx: Option<mpsc::Receiver<CameraFrame>>,
    is_running: bool,
    sequence_num: Arc<Mutex<u64>>,
}

impl GStreamerCamera {
    pub fn new(config: CameraConfig) -> Self {
        let (frame_tx, frame_rx) = mpsc::channel(10);
        
        Self {
            config,
            pipeline: None,
            main_loop: None,
            frame_tx: Some(frame_tx),
            frame_rx: Some(frame_rx),
            is_running: false,
            sequence_num: Arc::new(Mutex::new(0)),
        }
    }
    
    fn build_pipeline(&self) -> Result<gstreamer::Pipeline> {
        let pipeline_desc = if self.config.pipeline.is_empty() {
            // Default pipeline for USB camera
            format!(
                "v4l2src device={} ! video/x-raw,width={},height={},framerate={}/1 ! \
                 videoconvert ! video/x-raw,format=RGB ! appsink name=sink sync=false",
                self.config.device, self.config.width, self.config.height, self.config.framerate
            )
        } else {
            self.config.pipeline.clone()
        };
        
        info!("Creating GStreamer pipeline: {}", pipeline_desc);
        let pipeline = gstreamer::parse_launch(&pipeline_desc)?
            .downcast::<gstreamer::Pipeline>()
            .map_err(|_| anyhow!("Failed to downcast to pipeline"))?;
            
        Ok(pipeline)
    }
    
    fn setup_appsink(&self, pipeline: &gstreamer::Pipeline) -> Result<AppSink> {
        let appsink = pipeline
            .by_name("sink")
            .ok_or_else(|| anyhow!("No appsink element found in pipeline"))?
            .downcast::<AppSink>()
            .map_err(|_| anyhow!("Failed to downcast to AppSink"))?;
            
        // Configure appsink
        appsink.set_caps(Some(&gstreamer::Caps::new_simple(
            "video/x-raw",
            &[
                ("format", &gstreamer_video::VideoFormat::Rgb.to_string()),
                ("width", &(self.config.width as i32)),
                ("height", &(self.config.height as i32)),
            ],
        )));
        
        appsink.set_drop(true);
        appsink.set_max_buffers(5);
        appsink.set_emit_signals(true);
        
        Ok(appsink)
    }
    
    fn on_new_sample(
        &self,
        appsink: &AppSink,
        frame_tx: mpsc::Sender<CameraFrame>,
        sequence_num: Arc<Mutex<u64>>,
    ) -> Result<(), glib::error::Error> {
        let sample = appsink.pull_sample().map_err(|_| {
            glib::error::Error::new(gstreamer::CoreError::Failed, "Failed to pull sample")
        })?;
        
        let buffer = sample.buffer().ok_or_else(|| {
            glib::error::Error::new(gstreamer::CoreError::Failed, "Failed to get buffer")
        })?;
        
        let caps = sample.caps().ok_or_else(|| {
            glib::error::Error::new(gstreamer::CoreError::Failed, "Failed to get caps")
        })?;
        
        let video_info = VideoInfo::from_caps(&caps).map_err(|_| {
            glib::error::Error::new(gstreamer::CoreError::Failed, "Failed to get video info")
        })?;
        
        let width = video_info.width() as u32;
        let height = video_info.height() as u32;
        let format = video_info.format().to_string();
        
        // Map the buffer for reading
        let map = buffer.map_readable().map_err(|_| {
            glib::error::Error::new(gstreamer::CoreError::Failed, "Failed to map buffer")
        })?;
        
        let data = map.as_slice().to_vec();
        
        // Increment sequence number
        let mut seq_num = sequence_num.lock().unwrap();
        *seq_num += 1;
        let current_seq = *seq_num;
        
        // Create frame
        let frame = CameraFrame {
            data,
            width,
            height,
            format,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            sequence_num: current_seq,
        };
        
        // Send frame through channel (non-blocking)
        if let Err(e) = frame_tx.try_send(frame) {
            warn!("Failed to send frame: {}", e);
        }
        
        Ok(())
    }
}

#[async_trait]
impl Camera for GStreamerCamera {
    async fn start(&mut self) -> Result<()> {
        if self.is_running {
            return Ok(());
        }
        
        // Initialize GStreamer
        gstreamer::init().map_err(|e| anyhow!("Failed to initialize GStreamer: {}", e))?;
        
        // Build pipeline
        let pipeline = self.build_pipeline()?;
        let appsink = self.setup_appsink(&pipeline)?;
        
        // Clone needed values for callback
        let frame_tx = self.frame_tx.take().ok_or_else(|| anyhow!("Frame transmitter already taken"))?;
        let sequence_num = self.sequence_num.clone();
        
        // Connect to the new-sample signal
        appsink.connect_new_sample(move |appsink| {
            Self::on_new_sample(&Self, appsink, frame_tx.clone(), sequence_num.clone())
        });
        
        // Create and run main loop in a separate thread
        let main_loop = glib::MainLoop::new(None, false);
        let main_loop_clone = main_loop.clone();
        
        std::thread::spawn(move || {
            info!("Starting GStreamer main loop");
            main_loop_clone.run();
            info!("GStreamer main loop exited");
        });
        
        // Start the pipeline
        pipeline.set_state(gstreamer::State::Playing).map_err(|e| {
            anyhow!("Failed to set pipeline to playing state: {}", e)
        })?;
        
        self.pipeline = Some(pipeline);
        self.main_loop = Some(main_loop);
        self.is_running = true;
        
        info!("GStreamer camera started successfully");
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        if !self.is_running {
            return Ok(());
        }
        
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gstreamer::State::Null).map_err(|e| {
                anyhow!("Failed to set pipeline to null state: {}", e)
            })?;
        }
        
        if let Some(main_loop) = &self.main_loop {
            main_loop.quit();
        }
        
        self.is_running = false;
        info!("GStreamer camera stopped successfully");
        
        Ok(())
    }
    
    fn get_frame_rx(&self) -> Option<mpsc::Receiver<CameraFrame>> {
        self.frame_rx.clone()
    }
    
    fn get_config(&self) -> &CameraConfig {
        &self.config
    }
}

impl Drop for GStreamerCamera {
    fn drop(&mut self) {
        if self.is_running {
            let _ = self.stop();
        }
    }
}