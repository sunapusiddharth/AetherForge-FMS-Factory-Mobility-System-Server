use anyhow::{anyhow, Result};
use async_trait::async_trait;
use bincode;
use serde::Serialize;
use tokio::task;
use tracing::{error, info, instrument};
use zmq::{Context, Socket, PUB};

use crate::{
    config::MessagingConfig,
    types::PerceptionFrame,
};

#[async_trait]
pub trait MessagePublisher {
    async fn publish(&mut self, frame: &PerceptionFrame) -> Result<()>;
    async fn send_heartbeat(&mut self) -> Result<()>;
    fn get_config(&self) -> &MessagingConfig;
}

pub struct ZmqPublisher {
    socket: Socket,
    config: MessagingConfig,
    sequence_number: u64,
    last_heartbeat: std::time::Instant,
}

impl ZmqPublisher {
    pub fn new(config: &MessagingConfig) -> Result<Self> {
        info!("Initializing ZeroMQ publisher on {}", config.zmq_pub_endpoint);
        
        let context = Context::new();
        let socket = context.socket(PUB)?;
        
        // Configure socket options
        socket.set_sndhwm(config.high_water_mark)?;
        socket.set_sndtimeo(config.send_timeout_ms)?;
        socket.set_reconnect_ivl(config.reconnect_interval_ms)?;
        
        // Bind the socket
        socket.bind(&config.zmq_pub_endpoint)?;
        
        // For PUB sockets, we need to wait a bit after binding before sending messages
        // This ensures subscribers have time to connect
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        info!("ZeroMQ publisher initialized successfully");
        
        Ok(Self {
            socket,
            config: config.clone(),
            sequence_number: 0,
            last_heartbeat: std::time::Instant::now(),
        })
    }
    fn serialize_message<T: Serialize>(&self, data: &T) -> Result<Vec<u8>> {
        bincode::serialize(data).map_err(|e| anyhow!("Serialization error: {}", e))
    }
    
    fn create_envelope(&self, message_type: &str) -> String {
        format!("{} {}", self.config.zmq_topic, message_type)
    }
}

#[async_trait]
impl MessagePublisher for ZmqPublisher {
    #[instrument(skip(self, frame), level = "debug")]
    async fn publish(&mut self, frame: &PerceptionFrame) -> Result<()> {
        let envelope = self.create_envelope("perception_frame");
        let serialized = self.serialize_message(frame)?;
        
        // ZeroMQ requires sending the envelope first, then the message
        self.socket.send(envelope.as_bytes(), zmq::SNDMORE)?;
        self.socket.send(&serialized, 0)?;
        
        self.sequence_number += 1;
        
        // Check if it's time to send a heartbeat
        let now = std::time::Instant::now();
        if now.duration_since(self.last_heartbeat).as_secs() >= self.config.heartbeat_interval_sec {
            self.send_heartbeat().await?;
            self.last_heartbeat = now;
        }
        
        Ok(())
    }
    
    async fn send_heartbeat(&mut self) -> Result<()> {
        let heartbeat_msg = HeartbeatMessage {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            sequence_number: self.sequence_number,
            node_id: "perception_node_1".to_string(), // In a real system, this would be configurable
        };
        
        let envelope = self.create_envelope("heartbeat");
        let serialized = self.serialize_message(&heartbeat_msg)?;
        
        self.socket.send(envelope.as_bytes(), zmq::SNDMORE)?;
        self.socket.send(&serialized, 0)?;
        
        info!("Sent heartbeat message, sequence: {}", self.sequence_number);
        
        Ok(())
    }
    
    fn get_config(&self) -> &MessagingConfig {
        &self.config
    }
}

// Heartbeat message structure
#[derive(Serialize)]
struct HeartbeatMessage {
    timestamp: u64,
    sequence_number: u64,
    node_id: String,
}

impl Drop for ZmqPublisher {
    fn drop(&mut self) {
        info!("Shutting down ZeroMQ publisher");
        // ZeroMQ sockets are automatically closed when they go out of scope
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BBox, Detection, PerceptionFrame};
    
    #[test]
    fn test_serialization() {
        let config = MessagingConfig {
            zmq_pub_endpoint: "inproc://test".to_string(),
            zmq_topic: "test".to_string(),
            heartbeat_interval_sec: 5,
        };
        
        let publisher = ZmqPublisher::new(&config).unwrap();
        
        // Test serialization of a perception frame
        let mut frame = PerceptionFrame::new(
            1,
            "test_camera".to_string(),
            640,
            480,
            "1.0".to_string(),
        );
        
        frame.add_detection(Detection {
            bbox: BBox::new(10.0, 10.0, 50.0, 50.0),
            confidence: 0.95,
            class_id: 1,
            class_label: "robot".to_string(),
            tracker_id: Some(123),
        });
        
        let serialized = publisher.serialize_message(&frame).unwrap();
        let deserialized: PerceptionFrame = bincode::deserialize(&serialized).unwrap();
        
        assert_eq!(deserialized.frame_id, frame.frame_id);
        assert_eq!(deserialized.detections.len(), 1);
        assert_eq!(deserialized.detections[0].class_label, "robot");
    }
}