use actix_web::{
    get, patch,
    web::{Data, Json},
    HttpResponse, Responder,
};
use utoipa::{path as api_path, OpenApi};
use utoipa_actix_web::service_config::ServiceConfig;

use crate::models::host_info::{HostInfo, HostInfoStore};

#[derive(OpenApi)]
#[openapi(paths(get_host_info, update_host_info,))]
pub struct HostInfoApi;

/// Configures the Actix web service with the provided `HostInfoStore`.
///
/// This function sets up the application data and registers the necessary
/// services for handling host information.
///
/// # Arguments
///
/// * `store` - A `Data` instance containing the `HostInfoStore`.
///
/// # Returns
///
/// A closure that takes a mutable reference to `ServiceConfig` and configures
/// it with the provided `HostInfoStore` and services.
pub fn configure(store: Data<HostInfoStore>) -> impl FnOnce(&mut ServiceConfig) {
    move |cfg: &mut ServiceConfig| {
        cfg.app_data(store)
            .service(get_host_info)
            .service(update_host_info);
    }
}

#[api_path(operation_id = "get-host-information")]
#[get("")]
/// Retrieves the host information.
///
/// This function handles GET requests to retrieve the current host information
/// stored in the `HostInfoStore`.
///
/// # Arguments
///
/// * `store` - A `Data` instance containing the `HostInfoStore`.
///
/// # Returns
///
/// An `HttpResponse` containing the current host information in JSON format.
pub async fn get_host_info(store: Data<HostInfoStore>) -> impl Responder {
    let host_info = store.host_info.lock().unwrap();
    HttpResponse::Ok().json(&*host_info)
}

#[api_path(operation_id = "update-host-information")]
#[patch("")]
/// Updates the host information.
///
/// This function handles PATCH requests to update the current host information
/// stored in the `HostInfoStore`.
///
/// # Arguments
///
/// * `store` - A `Data` instance containing the `HostInfoStore`.
/// * `new_host_info` - A `Json` instance containing the new `HostInfo`.
///
/// # Returns
///
/// An `HttpResponse` containing the updated host information in JSON format.
pub async fn update_host_info(
    store: Data<HostInfoStore>,
    new_host_info: Json<HostInfo>,
) -> impl Responder {
    let new_host_info = new_host_info.into_inner();
    let mut store = store.host_info.lock().unwrap();
    store.set_hostname(new_host_info.get_hostname().clone());
    HttpResponse::Ok().json(&*store)
}
