pub mod misc;
pub mod models;
pub mod netplan;
pub mod opensight_os_api_lib;
pub mod routes;
use crate::routes::ethernet::EthernetsApi;
use actix_web::{get, HttpResponse, Responder};
use utoipa::OpenApi;

use opensight_os_api_lib::{ContactInformation, LicenseInformation, OpenSightOSApiLib};

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

fn config_api() -> OpenSightOSApiLib {
    let title = "Greenbone OpenSight Network Management API".to_string();
    let description = "API for Greenbone OpenSight Network Management Module".to_string();
    let version = "pensight-network".to_string();
    let server_args = vec![];
    let args = vec![];
    OpenSightOSApiLib::new(title, description, version, server_args, args)
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let opensight_os_api_lib = config_api();
    let openapi = ApiDoc::openapi();
    opensight_os_api_lib.start(openapi).await
}
