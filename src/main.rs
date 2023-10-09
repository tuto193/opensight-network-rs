// #![warn(missing_docs)]
// #![warn(clippy::missing_ docs_in_private_items)]

pub mod misc;
pub mod models;
pub mod netplan;
pub mod opensight_os_api_lib;
pub mod routes;
use crate::routes::{ethernet, host_info};
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use opensight_os_api_lib::OpenSightOSApiLib;
use std::net::Ipv4Addr;
use utoipa::{openapi::Info, OpenApi};
use utoipa_actix_web::AppExt;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

/// Configures the API documentation information.
///
/// This function sets up the title, description, and version for the Greenbone OpenSight Network Management API.
/// It initializes the `OpenSightOSApiLib` with these parameters and builds the API documentation info.
///
/// # Returns
///
/// * `Info` - The API documentation information.
fn config_api() -> Info {
    let title = "Greenbone OpenSight Network Management API".to_string();
    let description = "API for Greenbone OpenSight Network Management Module".to_string();
    let version = "opensight-network".to_string();
    let server_args = vec![];
    let args = vec![];
    let base_app = OpenSightOSApiLib::new(title, description, version, server_args, args);
    base_app.build_info()
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    // The OpenApi main struct that should hold the whole documentation of the API
    #[derive(utoipa::OpenApi)]
    #[openapi(
        // Nesting allows for grouping of routes in the documentation at different levels
        nest(
            // Each path has its own documentation (<Path>Api)
            (path = "/ethernets", api = ethernet::EthernetsApi),
            (path = "/host-info", api = host_info::HostInfoApi)
        ),
    )]
    pub struct ApiDoc;
    let mut openapi = ApiDoc::openapi();
    // Documentation's information is compiled from the OpenSight OS API Library
    // and this application's specific information
    let api_info = config_api();
    openapi.info = api_info;

    // Each route has its own store to hold the data (many routes can share the same store)
    let ethernet_routes_store = Data::new(netplan::NetplanStore::default());
    let host_info_routes_store = Data::new(models::host_info::HostInfoStore::default());
    HttpServer::new(move || {
        // The server's application must be started and configured from within this closure
        App::new()
            .into_utoipa_app()
            .openapi(openapi.clone())
            // Add some logging if wanted, so we can see what's happening
            .map(|app| app.wrap(Logger::default()))
            // The application's routes/scopes are configured here independently
            .service(
                utoipa_actix_web::scope("/ethernets")
                    .configure(routes::ethernet::configure(ethernet_routes_store.clone())),
            )
            .service(
                utoipa_actix_web::scope("/host-info")
                    .configure(routes::host_info::configure(host_info_routes_store.clone())),
            )
            .openapi_service(|api| {
                SwaggerUi::new("/docs/{_:.*}").url("/api-docs/openapi.json", api)
            })
            .openapi_service(|api| Redoc::with_url("/redoc", api))
            .map(|app| app.service(RapiDoc::new("api-docs/openapi.json").path("/rapidoc")))
            .openapi_service(|api| Scalar::with_url("/scalar", api))
            .into_app()
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await
}
