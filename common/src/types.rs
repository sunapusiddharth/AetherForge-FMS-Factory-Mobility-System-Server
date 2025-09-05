// use serde::{Serialize, Deserialize};
// use std::time::{SystemTime, UNIX_EPOCH};

// #[derive(Debug, Serialize, Deserialize, Clone, Copy)]
// pub struct BBox {
//     pub xmin: f32,
//     pub ymin: f32,
//     pub xmax: f32,
//     pub ymax: f32,
// }

// impl BBox {
//     pub fn new(xmin: f32, ymin: f32, xmax: f32, ymax: f32) -> Self {
//         Self { xmin, ymin, xmax, ymax }
//     }
    
//     pub fn width(&self) -> f32 {
//         self.xmax - self.xmin
//     }
    
//     pub fn height(&self) -> f32 {
//         self.ymax - self.ymin
//     }
    
//     pub fn area(&self) -> f32 {
//         self.width() * self.height()
//     }
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct Detection {
//     pub bbox: BBox,
//     pub confidence: f32,
//     pub class_id: u32,
//     pub class_label: String,
//     pub tracker_id: Option<u64>, // For tracking objects across frames
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct PerceptionFrame {
//     pub frame_id: u64,
//     pub timestamp: u64, // Milliseconds since epoch
//     pub source_camera_id: String,
//     pub image_width: u32,
//     pub image_height: u32,
//     pub model_version: String,
//     pub inference_time_ms: f32,
//     pub detections: Vec<Detection>,
//     pub camera_intrinsics: Option<CameraIntrinsics>,
//     pub camera_extrinsics: Option<CameraExtrinsics>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct CameraIntrinsics {
//     pub fx: f32, // Focal length x
//     pub fy: f32, // Focal length y
//     pub cx: f32, // Principal point x
//     pub cy: f32, // Principal point y
//     pub distortion: [f32; 5], // Radial and tangential distortion coefficients
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct CameraExtrinsics {
//     pub rotation: [f32; 3], // Euler angles or rotation vector
//     pub translation: [f32; 3],
// }

// impl PerceptionFrame {
//     pub fn new(
//         frame_id: u64,
//         source_camera_id: String,
//         image_width: u32,
//         image_height: u32,
//         model_version: String,
//     ) -> Self {
//         Self {
//             frame_id,
//             timestamp: SystemTime::now()
//                 .duration_since(UNIX_EPOCH)
//                 .unwrap()
//                 .as_millis() as u64,
//             source_camera_id,
//             image_width,
//             image_height,
//             model_version,
//             inference_time_ms: 0.0,
//             detections: Vec::new(),
//             camera_intrinsics: None,
//             camera_extrinsics: None,
//         }
//     }
    
//     pub fn add_detection(&mut self, detection: Detection) {
//         self.detections.push(detection);
//     }
    
//     pub fn set_inference_time(&mut self, time_ms: f32) {
//         self.inference_time_ms = time_ms;
//     }
    
//     pub fn set_camera_parameters(&mut self, intrinsics: CameraIntrinsics, extrinsics: CameraExtrinsics) {
//         self.camera_intrinsics = Some(intrinsics);
//         self.camera_extrinsics = Some(extrinsics);
//     }
// }




// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct MessagingConfig {
//     pub zmq_pub_endpoint: String,
//     pub zmq_topic: String,
//     pub heartbeat_interval_sec: u64,
//     pub high_water_mark: u32, // ZeroMQ HWM setting
//     pub send_timeout_ms: i32, // ZeroMQ send timeout
//     pub reconnect_interval_ms: i32, // For subscriber reconnection
// }

// impl Default for MessagingConfig {
//     fn default() -> Self {
//         Self {
//             zmq_pub_endpoint: "tcp://*:5555".to_string(),
//             zmq_topic: "perception_frames".to_string(),
//             heartbeat_interval_sec: 5,
//             high_water_mark: 1000,
//             send_timeout_ms: 1000,
//             reconnect_interval_ms: 100,
//         }
//     }
// }
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct BBox {
    pub xmin: f32,
    pub ymin: f32,
    pub xmax: f32,
    pub ymax: f32,
}

impl BBox {
    pub fn new(xmin: f32, ymin: f32, xmax: f32, ymax: f32) -> Self {
        Self { xmin, ymin, xmax, ymax }
    }
    
    pub fn width(&self) -> f32 {
        self.xmax - self.xmin
    }
    
    pub fn height(&self) -> f32 {
        self.ymax - self.ymin
    }
    
    pub fn area(&self) -> f32 {
        self.width() * self.height()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Detection {
    pub bbox: BBox,
    pub confidence: f32,
    pub class_id: u32,
    pub class_label: String,
    pub tracker_id: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerceptionFrame {
    pub frame_id: u64,
    pub timestamp: u64,
    pub source_camera_id: String,
    pub image_width: u32,
    pub image_height: u32,
    pub model_version: String,
    pub inference_time_ms: f32,
    pub detections: Vec<Detection>,
    pub camera_intrinsics: Option<CameraIntrinsics>,
    pub camera_extrinsics: Option<CameraExtrinsics>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CameraIntrinsics {
    pub fx: f32,
    pub fy: f32,
    pub cx: f32,
    pub cy: f32,
    pub distortion: [f32; 5],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CameraExtrinsics {
    pub rotation: [f32; 3],
    pub translation: [f32; 3],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CameraFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub timestamp: u64,
    pub sequence_num: u64,
}

// Shared camera status enums
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum CameraStatus {
    Online,
    Offline,
    Calibrating,
    Maintenance,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum CameraHealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum CalibrationStatus {
    NotCalibrated,
    Calibrating,
    Calibrated,
    NeedsRecalibration,
    Failed,
}

// Common configuration structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CameraConfig {
    pub device: String,
    pub width: u32,
    pub height: u32,
    pub framerate: u32,
    pub pipeline: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InferenceConfig {
    pub model_path: String,
    pub model_version: String,
    pub confidence_threshold: f32,
    pub nms_threshold: f32,
    pub input_width: u32,
    pub input_height: u32,
    pub use_gpu: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessagingConfig {
    pub zmq_pub_endpoint: String,
    pub zmq_topic: String,
    pub heartbeat_interval_sec: u64,
    pub high_water_mark: u32,
    pub send_timeout_ms: i32,
    pub reconnect_interval_ms: i32,
}