use actix_web::{web, HttpResponse, get, post, put, delete};
use uuid::Uuid;
use serde_json::json;
use std::collections::HashMap;

use crate::{
    models::{CreateCameraRequest, UpdateCameraRequest, CalibrationRequest},
    services::camera_service::CameraService,
    AppState,
};

#[get("/cameras")]
async fn get_cameras(
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let camera_service = CameraService::new(state.db_pool.clone(), state.file_storage.clone());
    
    let cameras = camera_service.get_all_cameras()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(cameras))
}

#[get("/cameras/{id}")]
async fn get_camera(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let camera_service = CameraService::new(state.db_pool.clone(), state.file_storage.clone());
    let camera_id = path.into_inner();
    
    let camera = camera_service.get_camera_by_id(camera_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(camera))
}

#[get("/cameras/zone/{zone}")]
async fn get_cameras_by_zone(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let camera_service = CameraService::new(state.db_pool.clone(), state.file_storage.clone());
    let zone = path.into_inner();
    
    let cameras = camera_service.get_cameras_by_zone(&zone)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(cameras))
}

#[get("/cameras/status/{status}")]
async fn get_cameras_by_status(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let camera_service = CameraService::new(state.db_pool.clone(), state.file_storage.clone());
    let status_str = path.into_inner();
    
    let status = match status_str.as_str() {
        "online" => crate::models::CameraStatus::Online,
        "offline" => crate::models::CameraStatus::Offline,
        "calibrating" => crate::models::CameraStatus::Calibrating,
        "maintenance" => crate::models::CameraStatus::Maintenance,
        "error" => crate::models::CameraStatus::Error,
        _ => return Err(actix_web::error::ErrorBadRequest("Invalid status")),
    };
    
    let cameras = camera_service.get_cameras_by_status(status)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(cameras))
}

#[post("/cameras")]
async fn create_camera(
    state: web::Data<AppState>,
    camera_data: web::Json<CreateCameraRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let camera_service = CameraService::new(state.db_pool.clone(), state.file_storage.clone());
    
    let camera = camera_service.create_camera(camera_data.into_inner())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(camera))
}

#[put("/cameras/{id}")]
async fn update_camera(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    camera_data: web::Json<UpdateCameraRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let camera_service = CameraService::new(state.db_pool.clone(), state.file_storage.clone());
    let camera_id = path.into_inner();
    
    let camera = camera_service.update_camera(camera_id, camera_data.into_inner())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(camera))
}

#[delete("/cameras/{id}")]
async fn delete_camera(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let camera_service = CameraService::new(state.db_pool.clone(), state.file_storage.clone());
    let camera_id = path.into_inner();
    
    camera_service.delete_camera(camera_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::NoContent().finish())
}

#[get("/cameras/{id}/calibration/history")]
async fn get_calibration_history(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let camera_service = CameraService::new(state.db_pool.clone(), state.file_storage.clone());
    let camera_id = path.into_inner();
    
    let history = camera_service.get_calibration_history(camera_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(history))
}

#[post("/cameras/{id}/calibration/start")]
async fn start_calibration(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    calibration_data: web::Json<CalibrationRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let camera_service = CameraService::new(state.db_pool.clone(), state.file_storage.clone());
    let camera_id = path.into_inner();
    
    camera_service.start_calibration(camera_id, calibration_data.into_inner())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Accepted().json(json!({"message": "Calibration started"})))
}

#[get("/cameras/{id}/health/metrics")]
async fn get_health_metrics(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<HashMap<String, i32>>,
) -> Result<HttpResponse, actix_web::Error> {
    let camera_service = CameraService::new(state.db_pool.clone(), state.file_storage.clone());
    let camera_id = path.into_inner();
    
    let hours = query.get("hours").cloned().unwrap_or(24);
    
    let metrics = camera_service.get_health_metrics(camera_id, hours)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(metrics))
}

#[get("/cameras/{id}/status/history")]
async fn get_status_history(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<HashMap<String, i64>>,
) -> Result<HttpResponse, actix_web::Error> {
    let camera_service = CameraService::new(state.db_pool.clone(), state.file_storage.clone());
    let camera_id = path.into_inner();
    
    let limit = query.get("limit").cloned();
    
    let history = camera_service.get_status_history(camera_id, limit)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(history))
}

#[get("/cameras/zones")]
async fn get_camera_zones(
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let camera_service = CameraService::new(state.db_pool.clone(), state.file_storage.clone());
    
    let zones = camera_service.get_camera_zones()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(zones))
}

#[get("/cameras/stats")]
async fn get_camera_stats(
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let camera_service = CameraService::new(state.db_pool.clone(), state.file_storage.clone());
    
    let stats = camera_service.get_camera_stats()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(stats))
}

#[get("/cameras/{id}/test-connection")]
async fn test_camera_connection(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let camera_service = CameraService::new(state.db_pool.clone(), state.file_storage.clone());
    let camera_id = path.into_inner();
    
    let is_connected = camera_service.test_camera_connection(camera_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(json!({"connected": is_connected})))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_cameras)
        .service(get_camera)
        .service(get_cameras_by_zone)
        .service(get_cameras_by_status)
        .service(create_camera)
        .service(update_camera)
        .service(delete_camera)
        .service(get_calibration_history)
        .service(start_calibration)
        .service(get_health_metrics)
        .service(get_status_history)
        .service(get_camera_zones)
        .service(get_camera_stats)
        .service(test_camera_connection);
}