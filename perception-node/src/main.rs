mod camera;
mod inference;
mod messaging;
mod processing;
mod utils;
mod config;
mod error;

use clap::Parser;
use config::PerceptionConfig;
use error::Result;
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Config file path
    #[arg(short, long, default_value = "config/perception.yaml")]
    config: String,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Initialize logging
    init_logging(&args.log_level)?;
    
    // Load configuration
    let config = load_config(&args.config).await?;
    
    info!("Starting AetherForge Perception Node {}", config.node_id);
    
    // Create application state
    let app_state = AppState::new(config).await?;
    
    // Start health monitoring
    let health_monitor = utils::health_check::HealthMonitor::new(app_state.clone());
    tokio::spawn(async move {
        if let Err(e) = health_monitor.start().await {
            error!("Health monitor failed: {}", e);
        }
    });
    
    // Start metrics server if enabled
    if app_state.config.monitoring.enable_metrics {
        let metrics_addr = format!("0.0.0.0:{}", app_state.config.monitoring.metrics_port);
        tokio::spawn(async move {
            if let Err(e) = utils::metrics::start_metrics_server(metrics_addr).await {
                error!("Metrics server failed: {}", e);
            }
        });
    }
    
    // Start processing pipeline
    let processor = processing::frame_processor::FrameProcessor::new(app_state.clone());
    processor.start().await?;
    
    // Wait for shutdown signal
    wait_for_shutdown().await;
    
    info!("Shutting down AetherForge Perception Node");
    Ok(())
}

fn init_logging(level: &str) -> Result<()> {
    let log_level = match level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };
    
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| error::PerceptionError::ConfigError(e.to_string()))?;
    
    Ok(())
}

async fn load_config(path: &str) -> Result<PerceptionConfig> {
    use config::Config;
    
    let settings = Config::builder()
        .add_source(config::File::with_name(path))
        .add_source(config::Environment::with_prefix("AETHERFORGE"))
        .build()
        .map_err(|e| error::PerceptionError::ConfigError(e.to_string()))?;
    
    settings.try_deserialize()
        .map_err(|e| error::PerceptionError::ConfigError(e.to_string()))
}

async fn wait_for_shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };
    
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };
    
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    
    info!("Shutdown signal received");
}

// Application state shared across components
#[derive(Clone)]
pub struct AppState {
    pub config: PerceptionConfig,
    pub camera_manager: Arc<camera::multi_camera::MultiCameraManager>,
    pub inference_engine: Arc<inference::ort_engine::OrtEngine>,
    pub message_publisher: Arc<messaging::zmq_pub::ZmqPublisher>,
    pub metrics: Arc<utils::metrics::Metrics>,
}

impl AppState {
    pub async fn new(config: PerceptionConfig) -> Result<Self> {
        // Initialize metrics
        let metrics = Arc::new(utils::metrics::Metrics::new());
        
        // Initialize camera manager
        let camera_manager = Arc::new(
            camera::multi_camera::MultiCameraManager::new(config.cameras.clone(), metrics.clone()).await?
        );
        
        // Initialize inference engine
        let inference_engine = Arc::new(
            inference::ort_engine::OrtEngine::new(&config.inference, metrics.clone()).await?
        );
        
        // Initialize message publisher
        let message_publisher = Arc::new(
            messaging::zmq_pub::ZmqPublisher::new(&config.messaging, metrics.clone())?
        );
        
        Ok(Self {
            config,
            camera_manager,
            inference_engine,
            message_publisher,
            metrics,
        })
    }
}