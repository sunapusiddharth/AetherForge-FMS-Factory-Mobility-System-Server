use actix_web::{web, HttpResponse, get, post, put, delete};
use uuid::Uuid;
use serde_json::json;

use crate::{
    models::{CreateTrainingJobRequest, UpdateTrainingJobRequest},
    services::training_service::TrainingService,
    AppState,
};

#[get("/training/jobs")]
async fn get_training_jobs(
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let training_service = TrainingService::new(state.db_pool.clone());
    
    let jobs = training_service.get_all_training_jobs()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(jobs))
}

#[get("/training/jobs/{id}")]
async fn get_training_job(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let training_service = TrainingService::new(state.db_pool.clone());
    let job_id = path.into_inner();
    
    let job = training_service.get_training_job(job_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(job))
}

#[post("/training/jobs")]
async fn create_training_job(
    state: web::Data<AppState>,
    user_id: web::ReqData<Uuid>,
    job_data: web::Json<CreateTrainingJobRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let training_service = TrainingService::new(state.db_pool.clone());
    
    let job = training_service.create_training_job(*user_id, job_data.into_inner())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(job))
}

#[put("/training/jobs/{id}")]
async fn update_training_job(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    job_data: web::Json<UpdateTrainingJobRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let training_service = TrainingService::new(state.db_pool.clone());
    let job_id = path.into_inner();
    
    let job = training_service.update_training_job(job_id, job_data.into_inner())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(job))
}

#[delete("/training/jobs/{id}")]
async fn delete_training_job(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let training_service = TrainingService::new(state.db_pool.clone());
    let job_id = path.into_inner();
    
    training_service.delete_training_job(job_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::NoContent().finish())
}

#[get("/training/stats")]
async fn get_training_stats(
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let training_service = TrainingService::new(state.db_pool.clone());
    
    let stats = training_service.get_training_job_stats()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(stats))
}

#[get("/training/summaries")]
async fn get_training_summaries(
    state: web::Data<AppState>,
    query: web::Query<HashMap<String, i64>>,
) -> Result<HttpResponse, actix_web::Error> {
    let training_service = TrainingService::new(state.db_pool.clone());
    
    let limit = query.get("limit").cloned();
    let summaries = training_service.get_training_job_summaries(limit)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(summaries))
}

#[post("/training/jobs/{id}/logs")]
async fn add_training_log(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    log_data: web::Json<HashMap<String, String>>,
) -> Result<HttpResponse, actix_web::Error> {
    let training_service = TrainingService::new(state.db_pool.clone());
    let job_id = path.into_inner();
    
    let log = log_data.get("log").map(|s| s.as_str()).unwrap_or("");
    
    let job = training_service.add_training_log(job_id, log)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(job))
}

#[get("/training/active")]
async fn get_active_training_jobs(
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let training_service = TrainingService::new(state.db_pool.clone());
    
    let jobs = training_service.get_active_training_jobs()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(jobs))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_training_jobs)
        .service(get_training_job)
        .service(create_training_job)
        .service(update_training_job)
        .service(delete_training_job)
        .service(get_training_stats)
        .service(get_training_summaries)
        .service(add_training_log)
        .service(get_active_training_jobs);
}