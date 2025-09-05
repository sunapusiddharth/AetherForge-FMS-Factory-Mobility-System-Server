use actix_web::{web, HttpResponse, get, post};
use uuid::Uuid;
use serde_json::json;

use crate::{
    models::{SystemEventType, EventSeverity},
    services::system_service::SystemService,
    AppState,
};

#[get("/system/health")]
async fn get_system_health(
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let system_service = SystemService::new(state.db_pool.clone());
    
    let health = system_service.get_system_health()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(health))
}

#[get("/system/metrics")]
async fn get_system_metrics(
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let system_service = SystemService::new(state.db_pool.clone());
    
    let metrics = system_service.get_system_metrics()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(metrics))
}

#[get("/system/stats")]
async fn get_system_stats(
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let system_service = SystemService::new(state.db_pool.clone());
    
    let stats = system_service.get_system_stats()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(stats))
}

#[get("/system/events")]
async fn get_system_events(
    state: web::Data<AppState>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, actix_web::Error> {
    let system_service = SystemService::new(state.db_pool.clone());
    
    let limit = query.get("limit").and_then(|s| s.parse().ok());
    let acknowledged = query.get("acknowledged").and_then(|s| s.parse().ok());
    
    let events = system_service.get_events(limit, acknowledged)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(events))
}

#[post("/system/events/{id}/acknowledge")]
async fn acknowledge_event(
    state: web::Data<AppState>,
    user_id: web::ReqData<Uuid>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let system_service = SystemService::new(state.db_pool.clone());
    let event_id = path.into_inner();
    
    let event = system_service.acknowledge_event(event_id, *user_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(event))
}

#[post("/system/events")]
async fn create_system_event(
    state: web::Data<AppState>,
    event_data: web::Json<HashMap<String, String>>,
) -> Result<HttpResponse, actix_web::Error> {
    let system_service = SystemService::new(state.db_pool.clone());
    
    let event_type = event_data.get("type").map(|s| s.as_str()).unwrap_or("other");
    let severity = event_data.get("severity").map(|s| s.as_str()).unwrap_or("info");
    let message = event_data.get("message").map(|s| s.as_str()).unwrap_or("");
    let source = event_data.get("source").map(|s| s.as_str());
    let details = event_data.get("details").map(|s| serde_json::from_str(s).ok()).flatten();
    
    let event_type_enum = match event_type {
        "camera_offline" => SystemEventType::CameraOffline,
        "camera_error" => SystemEventType::CameraError,
        "inference_error" => SystemEventType::InferenceError,
        "training_error" => SystemEventType::TrainingError,
        "storage_low" => SystemEventType::StorageLow,
        "memory_high" => SystemEventType::MemoryHigh,
        "cpu_high" => SystemEventType::CpuHigh,
        "service_down" => SystemEventType::ServiceDown,
        "model_performance_degraded" => SystemEventType::ModelPerformanceDegraded,
        "security_alert" => SystemEventType::SecurityAlert,
        _ => SystemEventType::Other,
    };
    
    let severity_enum = match severity {
        "critical" => EventSeverity::Critical,
        "high" => EventSeverity::High,
        "medium" => EventSeverity::Medium,
        "low" => EventSeverity::Low,
        _ => EventSeverity::Info,
    };
    
    let event = system_service.log_event(event_type_enum, severity_enum, message, source, details)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(event))
}

#[get("/system/events/unacknowledged/count")]
async fn get_unacknowledged_events_count(
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let system_service = SystemService::new(state.db_pool.clone());
    
    let count = system_service.get_unacknowledged_events_count()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(json!({ "count": count })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_system_health)
        .service(get_system_metrics)
        .service(get_system_stats)
        .service(get_system_events)
        .service(acknowledge_event)
        .service(create_system_event)
        .service(get_unacknowledged_events_count);
}