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

pub fn configure(store: Data<HostInfoStore>) -> impl FnOnce(&mut ServiceConfig) {
    move |cfg: &mut ServiceConfig| {
        cfg.app_data(store)
            .service(get_host_info)
            .service(update_host_info);
    }
}

#[api_path(operation_id = "get-host-information")]
#[get("")]
pub async fn get_host_info(store: Data<HostInfoStore>) -> impl Responder {
    let host_info = store.host_info.lock().unwrap();
    HttpResponse::Ok().json(&*host_info)
}

#[api_path(operation_id = "update-host-information")]
#[patch("")]
pub async fn update_host_info(
    store: Data<HostInfoStore>,
    new_host_info: Json<HostInfo>,
) -> impl Responder {
    let new_host_info = new_host_info.into_inner();
    let mut store = store.host_info.lock().unwrap();
    store.set_hostname(new_host_info.get_hostname().clone());
    HttpResponse::Ok().json(&*store)
}
