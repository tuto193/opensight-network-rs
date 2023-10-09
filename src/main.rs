// #![warn(missing_docs)]
// #![warn(clippy::missing_ docs_in_private_items)]
pub mod misc;
pub mod models;
pub mod netplan;
pub mod opensight_os_api_lib;
pub mod routes;
use crate::routes::ethernet::EthernetsApi;
use actix_web::{web::Data, App, HttpServer};
use std::net::Ipv4Addr;
use utoipa::{openapi::Info, OpenApi};
use utoipa_actix_web::AppExt;
use utoipa_swagger_ui::SwaggerUi;

use opensight_os_api_lib::OpenSightOSApiLib;

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
    #[derive(utoipa::OpenApi)]
    #[openapi(
        nest(
            (path = "/ethernets", api = EthernetsApi)
        ),
        tags(
            (name = "ethernets", description = "Operations related to Ethernet entries.")
        )
    )]
    pub struct ApiDoc;
    let mut openapi = ApiDoc::openapi();
    let api_info = config_api();
    openapi.info = api_info;

    let ethernet_routes_store = Data::new(netplan::NetplanStore::default());

    HttpServer::new(move || {
        App::new()
            .into_utoipa_app()
            .openapi(openapi.clone())
            .service(
                utoipa_actix_web::scope("/api/ethernets")
                    .configure(routes::ethernet::configure(ethernet_routes_store.clone())),
            )
            .openapi_service(|api| {
                SwaggerUi::new("/docs/{_:.*}").url("/api-docs/openapi.json", api)
            })
            .into_app()
    })
    .bind((Ipv4Addr::LOCALHOST, 8080))?
    .run()
    .await
}
