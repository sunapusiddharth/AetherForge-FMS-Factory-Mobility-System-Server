mod auth;
mod cameras;
mod calibration;
mod annotations;
mod models;
mod training;
mod system;
mod datasets;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(auth::configure)
            .configure(cameras::configure)
            .configure(calibration::configure)
            .configure(annotations::configure)
            .configure(models::configure)
            .configure(training::configure)
            .configure(system::configure)
            .configure(datasets::configure)
    );
}