use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use anyhow::Result;
use sqlx::postgres::PgPool;
use std::sync::Arc;
use tokio::time::Duration;

mod api;
mod config;
mod models;
mod services;
mod storage;

use config::OperatorConfig;
use storage::{create_db_pool, FileStorage};
use services::camera_monitor::CameraMonitor;

pub struct AppState {
    db_pool: PgPool,
    file_storage: FileStorage,
    config: OperatorConfig,
}

#[actix_web::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = OperatorConfig::default();
    
    // Initialize database
    let db_pool = create_db_pool(&config.database.url, config.database.max_connections).await?;
    
    // Initialize file storage
    let file_storage = FileStorage::new(config.storage.data_dir.clone());
    
    // Start camera monitor
    let camera_monitor = CameraMonitor::new(
        db_pool.clone(),
        Duration::from_secs(config.monitoring.health_check_interval_sec),
    );
    
    tokio::spawn(async move {
        if let Err(e) = camera_monitor.start().await {
            tracing::error!("Camera monitor failed: {}", e);
        }
    });
    
    // Create app state
    let app_state = web::Data::new(AppState {
        db_pool,
        file_storage,
        config,
    });
    
    // Start HTTP server
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origins(&app_state.config.server.cors_origins)
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec!["Content-Type", "Authorization"])
            .max_age(3600);
        
        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .configure(api::configure)
    })
    .bind((app_state.config.server.host.clone(), app_state.config.server.port))?
    .run();
    
    tracing::info!(
        "Operator platform server started on {}:{}",
        app_state.config.server.host,
        app_state.config.server.port
    );
    
    server.await?;
    
    Ok(())
}