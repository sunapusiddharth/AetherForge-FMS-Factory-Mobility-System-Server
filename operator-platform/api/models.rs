use actix_web::{web, HttpResponse, get, post, put, delete};
use uuid::Uuid;
use serde_json::json;

use crate::{
    models::{CreateModelRequest, UpdateModelRequest, DeploymentStatus},
    services::model_service::ModelService,
    AppState,
};

#[get("/models")]
async fn get_models(
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let model_service = ModelService::new(state.db_pool.clone());
    
    let models = model_service.get_all_models()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(models))
}

#[get("/models/{id}")]
async fn get_model(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let model_service = ModelService::new(state.db_pool.clone());
    let model_id = path.into_inner();
    
    let model = model_service.get_model(model_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(model))
}

#[get("/models/{name}/versions")]
async fn get_model_versions(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let model_service = ModelService::new(state.db_pool.clone());
    let model_name = path.into_inner();
    
    let versions = model_service.get_model_versions(&model_name)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(versions))
}

#[post("/models")]
async fn create_model(
    state: web::Data<AppState>,
    user_id: web::ReqData<Uuid>,
    model_data: web::Json<CreateModelRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let model_service = ModelService::new(state.db_pool.clone());
    
    let model = model_service.create_model(*user_id, model_data.into_inner())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(model))
}

#[put("/models/{id}")]
async fn update_model(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    model_data: web::Json<UpdateModelRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let model_service = ModelService::new(state.db_pool.clone());
    let model_id = path.into_inner();
    
    let model = model_service.update_model(model_id, model_data.into_inner())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(model))
}

#[delete("/models/{id}")]
async fn delete_model(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let model_service = ModelService::new(state.db_pool.clone());
    let model_id = path.into_inner();
    
    model_service.delete_model(model_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::NoContent().finish())
}

#[post("/models/{id}/deploy")]
async fn deploy_model(
    state: web::Data<AppState>,
    user_id: web::ReqData<Uuid>,
    path: web::Path<Uuid>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, actix_web::Error> {
    let model_service = ModelService::new(state.db_pool.clone());
    let model_id = path.into_inner();
    
    let deployed_to = query.get("deployed_to").map(|s| s.as_str()).unwrap_or("production");
    
    let deployment = model_service.deploy_model(model_id, deployed_to, *user_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(deployment))
}

#[get("/models/{id}/deployments")]
async fn get_model_deployments(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let model_service = ModelService::new(state.db_pool.clone());
    let model_id = path.into_inner();
    
    let deployments = model_service.get_model_deployments(model_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(deployments))
}

#[put("/deployments/{id}/status")]
async fn update_deployment_status(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, actix_web::Error> {
    let model_service = ModelService::new(state.db_pool.clone());
    let deployment_id = path.into_inner();
    
    let status_str = query.get("status").map(|s| s.as_str()).unwrap_or("active");
    let status = match status_str {
        "pending" => DeploymentStatus::Pending,
        "deploying" => DeploymentStatus::Deploying,
        "active" => DeploymentStatus::Active,
        "failed" => DeploymentStatus::Failed,
        "retiring" => DeploymentStatus::Retiring,
        "retired" => DeploymentStatus::Retired,
        _ => return Err(actix_web::error::ErrorBadRequest("Invalid status")),
    };
    
    let deployment = model_service.update_deployment_status(deployment_id, status)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(deployment))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_models)
        .service(get_model)
        .service(get_model_versions)
        .service(create_model)
        .service(update_model)
        .service(delete_model)
        .service(deploy_model)
        .service(get_model_deployments)
        .service(update_deployment_status);
}