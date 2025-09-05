use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::{
    config::{MessagingConfig, MessagingProtocol, CompressionType},
    error::{Result, PerceptionError},
    utils::metrics::Metrics,
};
use aetherforge_common::{PerceptionFrame, FusionResult, SystemHealth};

#[async_trait]
pub trait MessagePublisher: Send + Sync {
    async fn publish_perception_frame(&self, frame: &PerceptionFrame) -> Result<()>;
    async fn publish_fusion_result(&self, result: &FusionResult) -> Result<()>;
    async fn publish_system_health(&self, health: &SystemHealth) -> Result<()>;
    async fn publish_alert(&self, alert: &SystemAlert) -> Result<()>;
    async fn connect(&mut self) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    fn is_connected(&self) -> bool;
}

pub struct MultiProtocolPublisher {
    primary: Box<dyn MessagePublisher>,
    fallback: Option<Box<dyn MessagePublisher>>,
    config: MessagingConfig,
    metrics: Arc<Metrics>,
    connection_status: ConnectionStatus,
}

pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Degraded, // Using fallback
}

impl MultiProtocolPublisher {
    pub fn new(config: MessagingConfig, metrics: Arc<Metrics>) -> Result<Self> {
        let primary = Self::create_publisher(&config, &metrics)?;
        let fallback = if let Some(fallback_config) = &config.fallback_config {
            Some(Self::create_publisher(fallback_config, &metrics)?)
        } else {
            None
        };
        
        Ok(Self {
            primary,
            fallback,
            config,
            metrics,
            connection_status: ConnectionStatus::Disconnected,
        })
    }
    
    fn create_publisher(config: &MessagingConfig, metrics: &Arc<Metrics>) -> Result<Box<dyn MessagePublisher>> {
        match config.protocol {
            MessagingProtocol::ZeroMQ => Ok(Box::new(ZmqPublisher::new(config, metrics.clone())?)),
            MessagingProtocol::Redis => Ok(Box::new(RedisPublisher::new(config, metrics.clone())?)),
            MessagingProtocol::Kafka => Ok(Box::new(KafkaPublisher::new(config, metrics.clone())?)),
            MessagingProtocol::MQTT => Ok(Box::new(MqttPublisher::new(config, metrics.clone())?)),
        }
    }
    
    async fn try_publish<T, F>(&mut self, data: &T, publish_fn: F) -> Result<()>
    where
        F: Fn(&mut Box<dyn MessagePublisher>, &T) -> Result<()>,
    {
        // Try primary publisher
        match publish_fn(&mut self.primary, data) {
            Ok(()) => {
                self.connection_status = ConnectionStatus::Connected;
                Ok(())
            }
            Err(e) => {
                warn!("Primary publisher failed: {}", e);
                self.metrics.increment_message_failures();
                
                // Try fallback if available
                if let Some(ref mut fallback) = self.fallback {
                    match publish_fn(fallback, data) {
                        Ok(()) => {
                            self.connection_status = ConnectionStatus::Degraded;
                            Ok(())
                        }
                        Err(e) => {
                            error!("Fallback publisher also failed: {}", e);
                            self.connection_status = ConnectionStatus::Disconnected;
                            Err(e)
                        }
                    }
                } else {
                    self.connection_status = ConnectionStatus::Disconnected;
                    Err(e)
                }
            }
        }
    }
}

#[async_trait]
impl MessagePublisher for MultiProtocolPublisher {
    async fn publish_perception_frame(&self, frame: &PerceptionFrame) -> Result<()> {
        self.try_publish(frame, |publisher, data| publisher.publish_perception_frame(data)).await
    }
    
    async fn publish_fusion_result(&self, result: &FusionResult) -> Result<()> {
        self.try_publish(result, |publisher, data| publisher.publish_fusion_result(data)).await
    }
    
    async fn publish_system_health(&self, health: &SystemHealth) -> Result<()> {
        self.try_publish(health, |publisher, data| publisher.publish_system_health(data)).await
    }
    
    async fn publish_alert(&self, alert: &SystemAlert) -> Result<()> {
        self.try_publish(alert, |publisher, data| publisher.publish_alert(data)).await
    }
    
    async fn connect(&mut self) -> Result<()> {
        info!("Connecting messaging publishers");
        
        // Connect primary
        if let Err(e) = self.primary.connect().await {
            error!("Failed to connect primary publisher: {}", e);
            
            // Try fallback
            if let Some(ref mut fallback) = self.fallback {
                fallback.connect().await?;
                self.connection_status = ConnectionStatus::Degraded;
            } else {
                return Err(e);
            }
        } else {
            self.connection_status = ConnectionStatus::Connected;
        }
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting messaging publishers");
        
        let mut errors = Vec::new();
        
        if let Err(e) = self.primary.disconnect().await {
            errors.push(format!("Primary disconnect failed: {}", e));
        }
        
        if let Some(ref mut fallback) = self.fallback {
            if let Err(e) = fallback.disconnect().await {
                errors.push(format!("Fallback disconnect failed: {}", e));
            }
        }
        
        self.connection_status = ConnectionStatus::Disconnected;
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(PerceptionError::MessagingError(errors.join("; ")))
        }
    }
    
    fn is_connected(&self) -> bool {
        matches!(self.connection_status, ConnectionStatus::Connected | ConnectionStatus::Degraded)
    }
}

// Enhanced ZeroMQ implementation with compression
pub struct ZmqPublisher {
    context: zmq::Context,
    socket: Option<zmq::Socket>,
    config: MessagingConfig,
    metrics: Arc<Metrics>,
    sequence_number: u64,
    compression: CompressionStrategy,
}

impl ZmqPublisher {
    pub fn new(config: &MessagingConfig, metrics: Arc<Metrics>) -> Result<Self> {
        let context = zmq::Context::new();
        let compression = CompressionStrategy::from_config(&config.compression);
        
        Ok(Self {
            context,
            socket: None,
            config: config.clone(),
            metrics,
            sequence_number: 0,
            compression,
        })
    }
    
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        match self.compression {
            CompressionStrategy::None => Ok(data.to_vec()),
            CompressionStrategy::Zstd => {
                zstd::encode_all(data, 3)
                    .map_err(|e| PerceptionError::MessagingError(format!("Zstd compression failed: {}", e)))
            }
            CompressionStrategy::Lz4 => {
                lz4_flex::compress_prepend_size(data)
                    .map_err(|e| PerceptionError::MessagingError(format!("LZ4 compression failed: {}", e)))
            }
            CompressionStrategy::Gzip => {
                use flate2::{Compression, write::GzEncoder};
                use std::io::Write;
                
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(data)
                    .map_err(|e| PerceptionError::MessagingError(format!("Gzip compression failed: {}", e)))?;
                encoder.finish()
                    .map_err(|e| PerceptionError::MessagingError(format!("Gzip compression failed: {}", e)))
            }
        }
    }
}

#[async_trait]
impl MessagePublisher for ZmqPublisher {
    async fn publish_perception_frame(&mut self, frame: &PerceptionFrame) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        // Serialize frame
        let serialized = bincode::serialize(frame)
            .map_err(|e| PerceptionError::MessagingError(format!("Serialization failed: {}", e)))?;
        
        // Compress data
        let compressed = self.compress_data(&serialized)?;
        
        // Create message envelope
        let envelope = MessageEnvelope {
            message_type: MessageType::PerceptionFrame,
            camera_id: frame.source_camera_id.clone(),
            sequence_number: self.sequence_number,
            timestamp: frame.timestamp,
            compression: self.compression.to_string(),
            original_size: serialized.len(),
            compressed_size: compressed.len(),
        };
        
        let serialized_envelope = bincode::serialize(&envelope)
            .map_err(|e| PerceptionError::MessagingError(format!("Envelope serialization failed: {}", e)))?;
        
        // Send message
        if let Some(socket) = &self.socket {
            socket.send(&serialized_envelope, zmq::SNDMORE)
                .map_err(|e| PerceptionError::MessagingError(format!("Failed to send envelope: {}", e)))?;
            
            socket.send(&compressed, 0)
                .map_err(|e| PerceptionError::MessagingError(format!("Failed to send message: {}", e)))?;
            
            self.sequence_number += 1;
            self.metrics.record_message_sent(compressed.len(), start_time.elapsed());
            
            Ok(())
        } else {
            Err(PerceptionError::MessagingError("Not connected".to_string()))
        }
    }
    
    // Other publish methods implemented similarly
    
    async fn connect(&mut self) -> Result<()> {
        let socket = self.context.socket(zmq::PUB)
            .map_err(|e| PerceptionError::MessagingError(format!("Failed to create socket: {}", e)))?;
        
        // Configure socket
        socket.set_sndhwm(self.config.high_water_mark as i32)
            .map_err(|e| PerceptionError::MessagingError(format!("Failed to set HWM: {}", e)))?;
        
        socket.set_sndtimeo(self.config.send_timeout_ms)
            .map_err(|e| PerceptionError::MessagingError(format!("Failed to set timeout: {}", e)))?;
        
        // Bind or connect based on endpoint type
        if self.config.endpoint.starts_with("tcp://*:") {
            socket.bind(&self.config.endpoint)
                .map_err(|e| PerceptionError::MessagingError(format!("Failed to bind: {}", e)))?;
        } else {
            socket.connect(&self.config.endpoint)
                .map_err(|e| PerceptionError::MessagingError(format!("Failed to connect: {}", e)))?;
        }
        
        self.socket = Some(socket);
        info!("ZeroMQ publisher connected to {}", self.config.endpoint);
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        if let Some(socket) = self.socket.take() {
            // ZeroMQ sockets are automatically closed when dropped
            info!("ZeroMQ publisher disconnected");
        }
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.socket.is_some()
    }
    
    async fn publish_fusion_result(&self, result: &FusionResult) -> Result<()> {
        todo!()
    }
    
    async fn publish_system_health(&self, health: &SystemHealth) -> Result<()> {
        todo!()
    }
    
    async fn publish_alert(&self, alert: &SystemAlert) -> Result<()> {
        todo!()
    }
}

// Support for other protocols (Redis, Kafka, MQTT) would be implemented similarly

pub struct MessageEnvelope {
    pub message_type: MessageType,
    pub camera_id: String,
    pub sequence_number: u64,
    pub timestamp: u64,
    pub compression: String,
    pub original_size: usize,
    pub compressed_size: usize,
}

pub enum MessageType {
    PerceptionFrame,
    FusionResult,
    SystemHealth,
    Alert,
}

pub enum CompressionStrategy {
    None,
    Zstd,
    Lz4,
    Gzip,
}

impl CompressionStrategy {
    fn from_config(compression: &CompressionType) -> Self {
        match compression {
            CompressionType::None => Self::None,
            CompressionType::Zstd => Self::Zstd,
            CompressionType::Lz4 => Self::Lz4,
            CompressionType::Gzip => Self::Gzip,
        }
    }
    
    fn to_string(&self) -> String {
        match self {
            Self::None => "none".to_string(),
            Self::Zstd => "zstd".to_string(),
            Self::Lz4 => "lz4".to_string(),
            Self::Gzip => "gzip".to_string(),
        }
    }
}

// System health and alert structures
pub struct SystemHealth {
    pub node_id: String,
    pub status: NodeStatus,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub gpu_usage: Option<f32>,
    pub camera_status: Vec<CameraHealth>,
    pub inference_metrics: InferenceMetrics,
    pub timestamp: u64,
}

pub enum NodeStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

pub struct CameraHealth {
    pub camera_id: String,
    pub status: CameraStatus,
    pub fps: f32,
    pub latency_ms: f32,
}

pub struct SystemAlert {
    pub severity: AlertSeverity,
    pub source: String,
    pub message: String,
    pub timestamp: u64,
    pub details: Option<serde_json::Value>,
}

pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}