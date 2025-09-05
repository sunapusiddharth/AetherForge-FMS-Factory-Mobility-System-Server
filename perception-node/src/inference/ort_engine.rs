use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use async_trait::async_trait;
use ort::{Session, SessionBuilder, ExecutionProvider};
use ndarray::{Array4, Axis};
use tracing::{debug, error, info, instrument, warn};

use crate::{
    config::{InferenceConfig, InferenceBackend},
    error::{Result, PerceptionError},
    utils::metrics::Metrics,
    processing::fusion_engine::FusionResult,
};
use aetherforge_common::{CameraFrame, Detection, BBox, PerceptionFrame};

#[derive(Clone)]
pub struct OrtEngine {
    sessions: Arc<DashMap<String, Session>>, // Multiple models by name
    config: InferenceConfig,
    metrics: Arc<Metrics>,
    current_model: String,
    batch_processor: BatchProcessor,
}

pub struct BatchProcessor {
    max_batch_size: usize,
    batch_timeout: Duration,
    pending_frames: Vec<(CameraFrame, Instant)>,
}

impl OrtEngine {
    pub async fn new(config: &InferenceConfig, metrics: Arc<Metrics>) -> Result<Self> {
        info!("Initializing ORT inference engine with config: {:?}", config);
        
        let mut sessions = DashMap::new();
        
        // Load primary detection model
        let detection_session = Self::create_session(&config.model_path, config).await?;
        sessions.insert("detection".to_string(), detection_session);
        
        // Load segmentation model if configured
        if let Some(seg_model_path) = &config.segmentation_model_path {
            let seg_config = config.clone(); // Would have different config in reality
            let seg_session = Self::create_session(seg_model_path, &seg_config).await?;
            sessions.insert("segmentation".to_string(), seg_session);
        }
        
        // Load robot identification model if configured
        if let Some(robot_model_path) = &config.robot_identification_model_path {
            let robot_config = config.clone();
            let robot_session = Self::create_session(robot_model_path, &robot_config).await?;
            sessions.insert("robot_identification".to_string(), robot_session);
        }
        
        let batch_processor = BatchProcessor {
            max_batch_size: config.max_batch_size,
            batch_timeout: Duration::from_millis(config.batch_timeout_ms),
            pending_frames: Vec::with_capacity(config.max_batch_size),
        };
        
        Ok(Self {
            sessions: Arc::new(sessions),
            config: config.clone(),
            metrics,
            current_model: "detection".to_string(),
            batch_processor,
        })
    }
    
    async fn create_session(model_path: &std::path::Path, config: &InferenceConfig) -> Result<Session> {
        let mut session_builder = SessionBuilder::new()?;
        
        // Configure hardware acceleration based on backend
        match config.inference_backend {
            InferenceBackend::Cpu => {
                // Use CPU provider with optimizations
                session_builder = session_builder
                    .with_optimization_level(ort::GraphOptimizationLevel::Level3)?
                    .with_intra_threads(num_cpus::get() as i16)?;
            }
            InferenceBackend::Cuda => {
                #[cfg(feature = "cuda")]
                {
                    session_builder = session_builder
                        .with_execution_providers([ExecutionProvider::CUDA(Default::default())])?;
                }
                #[cfg(not(feature = "cuda"))]
                {
                    warn!("CUDA requested but not available in build. Falling back to CPU.");
                }
            }
            InferenceBackend::TensorRT => {
                #[cfg(feature = "tensorrt")]
                {
                    session_builder = session_builder
                        .with_execution_providers([ExecutionProvider::TensorRT(Default::default())])?;
                }
                #[cfg(not(feature = "tensorrt"))]
                {
                    warn!("TensorRT requested but not available. Falling back to CPU.");
                }
            }
            InferenceBackend::OpenVINO => {
                #[cfg(feature = "openvino")]
                {
                    session_builder = session_builder
                        .with_execution_providers([ExecutionProvider::OpenVINO(Default::default())])?;
                }
                #[cfg(not(feature = "openvino"))]
                {
                    warn!("OpenVINO requested but not available. Falling back to CPU.");
                }
            }
        }
        
        let session = session_builder
            .with_model_from_file(model_path)
            .map_err(|e| PerceptionError::InferenceError(format!("Failed to load model: {}", e)))?;
            
        info!("Model loaded successfully: {}", model_path.display());
        Ok(session)
    }
    
    #[instrument(skip(self, frame), level = "debug")]
    pub async fn process_frame(&mut self, frame: CameraFrame) -> Result<PerceptionFrame> {
        let start_time = Instant::now();
        
        // Add to batch processor
        self.batch_processor.pending_frames.push((frame, start_time));
        
        // Check if we should process the batch
        if self.batch_processor.pending_frames.len() >= self.batch_processor.max_batch_size ||
           start_time.duration_since(self.batch_processor.pending_frames.first().unwrap().1) 
           >= self.batch_processor.batch_timeout 
        {
            return self.process_batch().await;
        }
        
        // For single frame processing, we'll still process immediately
        // In a real implementation, we might use a background task for batching
        self.process_batch().await
    }
    
    async fn process_batch(&mut self) -> Result<PerceptionFrame> {
        if self.batch_processor.pending_frames.is_empty() {
            return Err(PerceptionError::InferenceError("No frames to process".to_string()));
        }
        
        let batch_size = self.batch_processor.pending_frames.len();
        let mut batch_tensors = Vec::with_capacity(batch_size);
        let mut frames = Vec::with_capacity(batch_size);
        
        // Preprocess all frames in the batch
        for (frame, _) in self.batch_processor.pending_frames.drain(..) {
            let input_tensor = self.preprocess(&frame)?;
            batch_tensors.push(input_tensor);
            frames.push(frame);
        }
        
        // Stack batch tensors
        let batch_input = self.create_batch_input(batch_tensors)?;
        
        // Run inference
        let session = self.sessions.get(&self.current_model)
            .ok_or_else(|| PerceptionError::InferenceError("Model not found".to_string()))?;
        
        let outputs = self.run_inference(session.value(), batch_input).await?;
        
        // Postprocess results
        let results = self.postprocess_batch(outputs, &frames)?;
        
        // For now, return the first result
        // In a real implementation, we'd return all results
        Ok(results.into_iter().next()
            .ok_or_else(|| PerceptionError::InferenceError("No results from batch".to_string()))?)
    }
    
    fn create_batch_input(&self, tensors: Vec<Array4<f32>>) -> Result<Array4<f32>> {
        let batch_size = tensors.len();
        if batch_size == 0 {
            return Err(PerceptionError::InferenceError("Empty batch".to_string()));
        }
        
        let shape = tensors[0].shape();
        let mut batch_array = Array4::zeros((batch_size, shape[1], shape[2], shape[3]));
        
        for (i, tensor) in tensors.into_iter().enumerate() {
            batch_array.slice_mut(s![i, .., .., ..]).assign(&tensor);
        }
        
        Ok(batch_array)
    }
    
    async fn run_inference(&self, session: &Session, input: Array4<f32>) -> Result<Vec<ort::Value>> {
        let input_tensor = ort::Value::from_array(session.allocator(), &input)
            .map_err(|e| PerceptionError::InferenceError(format!("Failed to create input tensor: {}", e)))?;
        
        let outputs = session.run(vec![input_tensor])
            .map_err(|e| PerceptionError::InferenceError(format!("Inference failed: {}", e)))?;
        
        Ok(outputs)
    }
    
    fn postprocess_batch(&self, outputs: Vec<ort::Value>, frames: &[CameraFrame]) -> Result<Vec<PerceptionFrame>> {
        let mut results = Vec::with_capacity(frames.len());
        
        for (i, frame) in frames.iter().enumerate() {
            // Extract results for this batch item
            let mut detections = Vec::new();
            
            // This is a simplified postprocessing - actual implementation depends on model output format
            let output = &outputs[0];
            let output_array = output.try_extract_tensor::<f32>()
                .map_err(|e| PerceptionError::InferenceError(format!("Failed to extract tensor: {}", e)))?;
            
            let num_detections = output_array.shape()[1];
            
            for j in 0..num_detections {
                let confidence = output_array[[i, j, 4]];
                
                if confidence < self.config.confidence_threshold {
                    continue;
                }
                
                // Extract detection details (simplified)
                let x = output_array[[i, j, 0]];
                let y = output_array[[i, j, 1]];
                let w = output_array[[i, j, 2]];
                let h = output_array[[i, j, 3]];
                
                // Convert to pixel coordinates
                let xmin = (x - w / 2.0) * frame.width as f32;
                let ymin = (y - h / 2.0) * frame.height as f32;
                let xmax = (x + w / 2.0) * frame.width as f32;
                let ymax = (y + h / 2.0) * frame.height as f32;
                
                // Find class with highest score
                let mut max_class = 0;
                let mut max_score = 0.0;
                let num_classes = output_array.shape()[2] - 5;
                
                for c in 0..num_classes {
                    let score = output_array[[i, j, 5 + c]];
                    if score > max_score {
                        max_score = score;
                        max_class = c;
                    }
                }
                
                let final_confidence = confidence * max_score;
                
                if final_confidence < self.config.confidence_threshold {
                    continue;
                }
                
                let class_label = if max_class < self.config.class_names.len() {
                    self.config.class_names[max_class].clone()
                } else {
                    format!("class_{}", max_class)
                };
                
                let detection = Detection {
                    bbox: BBox::new(xmin, ymin, xmax, ymax),
                    confidence: final_confidence,
                    class_id: max_class as u32,
                    class_label,
                    tracker_id: None,
                };
                
                detections.push(detection);
            }
            
            // Apply NMS
            let detections = self.apply_nms(detections);
            
            // Create perception frame
            let mut perception_frame = PerceptionFrame::new(
                0, // Will be set by main loop
                frame.sequence_num,
                frame.timestamp,
                frame.camera_id.clone(),
                frame.width,
                frame.height,
                self.config.model_version.clone(),
            );
            
            perception_frame.detections = detections;
            perception_frame.inference_time_ms = start_time.elapsed().as_secs_f32() * 1000.0;
            
            results.push(perception_frame);
        }
        
        Ok(results)
    }
    
    // Additional methods for multi-model processing
    pub async fn process_segmentation(&self, frame: &CameraFrame) -> Result<SegmentationResult> {
        let session = self.sessions.get("segmentation")
            .ok_or_else(|| PerceptionError::InferenceError("Segmentation model not loaded".to_string()))?;
        
        // Similar processing pipeline but for segmentation
        let input_tensor = self.preprocess(frame)?;
        let outputs = self.run_inference(session.value(), input_tensor).await?;
        let segmentation = self.postprocess_segmentation(outputs, frame)?;
        
        Ok(segmentation)
    }
    
    pub async fn identify_robot(&self, frame: &CameraFrame, detection: &Detection) -> Result<RobotIdentification> {
        let session = self.sessions.get("robot_identification")
            .ok_or_else(|| PerceptionError::InferenceError("Robot identification model not loaded".to_string()))?;
        
        // Extract ROI based on detection
        let roi = self.extract_roi(frame, detection);
        let input_tensor = self.preprocess_roi(&roi)?;
        let outputs = self.run_inference(session.value(), input_tensor).await?;
        let robot_id = self.postprocess_robot_identification(outputs)?;
        
        Ok(robot_id)
    }
    
    pub fn switch_model(&mut self, model_name: &str) -> Result<()> {
        if self.sessions.contains_key(model_name) {
            self.current_model = model_name.to_string();
            Ok(())
        } else {
            Err(PerceptionError::InferenceError(format!("Model {} not found", model_name)))
        }
    }
    
    pub fn get_available_models(&self) -> Vec<String> {
        self.sessions.iter().map(|s| s.key().clone()).collect()
    }
    
    // Health monitoring
    pub fn get_inference_metrics(&self) -> InferenceMetrics {
        InferenceMetrics {
            batch_size: self.batch_processor.pending_frames.len(),
            model_memory_usage: self.get_model_memory_usage(),
            inference_latency: self.metrics.get_average_latency(),
            throughput: self.metrics.get_throughput(),
        }
    }
}

// Support for different model types
pub enum ModelType {
    ObjectDetection,
    SemanticSegmentation,
    InstanceSegmentation,
    RobotIdentification,
    PoseEstimation,
}

pub struct SegmentationResult {
    pub mask: Vec<u8>, // Segmentation mask
    pub classes: Vec<String>, // Class labels for each segment
    pub confidence: f32,
}

pub struct RobotIdentification {
    pub robot_id: String,
    pub model: String,
    pub confidence: f32,
    pub pose: Option<PoseEstimation>,
}

pub struct PoseEstimation {
    pub keypoints: Vec<Keypoint>,
    pub skeleton: Vec<(usize, usize)>,
}

pub struct Keypoint {
    pub x: f32,
    pub y: f32,
    pub confidence: f32,
    pub id: usize,
}

pub struct InferenceMetrics {
    pub batch_size: usize,
    pub model_memory_usage: u64,
    pub inference_latency: f32,
    pub throughput: f32,
}