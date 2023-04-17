//! Health and Dockerflow routes
use std::thread;

use actix_web::http::StatusCode;
use actix_web::web::{self, Data, Json};
use actix_web::HttpResponse;
use serde_json::json;

use crate::settings::Settings;

/// Heartbeat is called regularly to access the system state. This call should return quickly
/// but can be used to do a health check for required systems.
pub async fn heartbeat(_state: Data<Settings>) -> Json<serde_json::Value> {
    //TODO: query local state and report results
    Json(json!({
        "status": "OK",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Used by the load balancer to indicate the server can respond to
/// requests. Should just return OK.
pub async fn lbheartbeat() -> HttpResponse {
    // Used by the load balancers, just return OK.
    HttpResponse::Ok().finish()
}

/// Version returns the filled content of the version.json file
/// It allows us to determine the version and release of code running on a given server.
pub async fn version() -> HttpResponse {
    // Return the contents of the version.json file created by circleci
    // and stored in the docker root
    HttpResponse::Ok()
        .content_type("application/json")
        .body(include_str!("../../version.json"))
}

/// Generate a test error to check logging.
pub async fn test_error() -> HttpResponse {
    // Note: This generally appears as an error in vsCode. It is safe to ignore that.
    error!(
        "Test Critical Message";
        "status_code" => StatusCode::IM_A_TEAPOT.as_u16(),
        "errno" => 999,
    );

    thread::spawn(|| {
        panic!("LogCheck");
    });

    HttpResponse::new(StatusCode::IM_A_TEAPOT)
}

/// Handles required Dockerflow Endpoints.
pub fn configure(config: &mut web::ServiceConfig) {
    config
        .service(web::resource("__lbheartbeat__").route(web::get().to(lbheartbeat)))
        .service(web::resource("__heartbeat__").route(web::get().to(heartbeat)))
        .service(web::resource("__version__").route(web::get().to(version)))
        .service(web::resource("__error__").route(web::get().to(test_error)));
}
