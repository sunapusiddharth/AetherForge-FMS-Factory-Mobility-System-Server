use actix_web::{web, HttpResponse, get, post, put, delete};
use uuid::Uuid;
use serde_json::json;

use crate::{
    models::{CreateAnnotationRequest, UpdateAnnotationRequest},
    services::annotation_service::AnnotationService,
    AppState,
};

#[get("/annotations/{id}")]
async fn get_annotation(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let annotation_service = AnnotationService::new(state.db_pool.clone());
    let annotation_id = path.into_inner();
    
    let annotation = annotation_service.get_annotation(annotation_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(annotation))
}

#[get("/cameras/{camera_id}/annotations")]
async fn get_camera_annotations(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<HashMap<String, i64>>,
) -> Result<HttpResponse, actix_web::Error> {
    let annotation_service = AnnotationService::new(state.db_pool.clone());
    let camera_id = path.into_inner();
    
    let limit = query.get("limit").cloned();
    let offset = query.get("offset").cloned();
    
    let annotations = annotation_service.get_annotations_by_camera(camera_id, limit, offset)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(annotations))
}

#[post("/annotations")]
async fn create_annotation(
    state: web::Data<AppState>,
    user_id: web::ReqData<Uuid>, // Assuming we have authentication middleware
    annotation_data: web::Json<CreateAnnotationRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let annotation_service = AnnotationService::new(state.db_pool.clone());
    
    let annotation = annotation_service.create_annotation(*user_id, annotation_data.into_inner())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(annotation))
}

#[put("/annotations/{id}")]
async fn update_annotation(
    state: web::Data<AppState>,
    user_id: web::ReqData<Uuid>,
    path: web::Path<Uuid>,
    annotation_data: web::Json<UpdateAnnotationRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let annotation_service = AnnotationService::new(state.db_pool.clone());
    let annotation_id = path.into_inner();
    
    let annotation = annotation_service.update_annotation(annotation_id, *user_id, annotation_data.into_inner())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(annotation))
}

#[delete("/annotations/{id}")]
async fn delete_annotation(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let annotation_service = AnnotationService::new(state.db_pool.clone());
    let annotation_id = path.into_inner();
    
    annotation_service.delete_annotation(annotation_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::NoContent().finish())
}

#[get("/annotations/stats")]
async fn get_annotation_stats(
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let annotation_service = AnnotationService::new(state.db_pool.clone());
    
    let stats = annotation_service.get_annotation_stats()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(stats))
}

#[get("/annotations/export")]
async fn export_annotations(
    state: web::Data<AppState>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, actix_web::Error> {
    let annotation_service = AnnotationService::new(state.db_pool.clone());
    
    let format = query.get("format").map(|s| s.as_str()).unwrap_or("csv");
    let data = annotation_service.export_annotations(format)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok()
        .content_type("text/csv")
        .header("Content-Disposition", "attachment; filename=annotations.csv")
        .body(data))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_annotation)
        .service(get_camera_annotations)
        .service(create_annotation)
        .service(update_annotation)
        .service(delete_annotation)
        .service(get_annotation_stats)
        .service(export_annotations);
}