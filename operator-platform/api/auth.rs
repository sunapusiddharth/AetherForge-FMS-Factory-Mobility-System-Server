use actix_web::{web, HttpResponse, post};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
use uuid::Uuid;

use crate::{
    models::{LoginRequest, AuthResponse, CreateUserRequest, User, UserRole},
    services::user_service::UserService,
    AppState,
};

#[post("/auth/register")]
async fn register(
    state: web::Data<AppState>,
    user_data: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_service = UserService::new(state.db_pool.clone());
    
    // Check if user already exists
    if user_service.get_user_by_email(&user_data.email).await.is_ok() {
        return Ok(HttpResponse::Conflict().json(json!({
            "error": "User with this email already exists"
        })));
    }
    
    // Hash password
    let password_hash = hash(&user_data.password, state.config.auth.password_hash_cost)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    // Create user
    let user = user_service.create_user(
        &user_data.username,
        &user_data.email,
        &password_hash,
        user_data.role.clone(),
    ).await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    // Generate token
    let token = generate_token(&user, &state.config.auth.secret_key)?;
    
    Ok(HttpResponse::Ok().json(AuthResponse { token, user }))
}

#[post("/auth/login")]
async fn login(
    state: web::Data<AppState>,
    login_data: web::Json<LoginRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_service = UserService::new(state.db_pool.clone());
    
    // Get user by email
    let user = user_service.get_user_by_email(&login_data.email)
        .await
        .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid credentials"))?;
    
    // Verify password
    let valid = verify(&login_data.password, &user.password_hash)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    if !valid {
        return Err(actix_web::error::ErrorUnauthorized("Invalid credentials"));
    }
    
    // Generate token
    let token = generate_token(&user, &state.config.auth.secret_key)?;
    
    Ok(HttpResponse::Ok().json(AuthResponse { token, user }))
}

fn generate_token(user: &User, secret_key: &str) -> Result<String, actix_web::Error> {
    let expiration = Utc::now() + Duration::hours(24);
    
    let claims = json!({
        "sub": user.id.to_string(),
        "email": user.email,
        "role": user.role,
        "exp": expiration.timestamp(),
    });
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    ).map_err(|e| actix_web::error::ErrorInternalServerError(e))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(register).service(login);
}