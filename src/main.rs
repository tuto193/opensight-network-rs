mod models;
mod netplan;
mod opensight_os_api_lib;
mod routes;
use actix_web::{get, HttpResponse, Responder};
use utoipa::OpenApi;

use opensight_os_api_lib::{ContactInformation, LicenseInformation, OpenSightOSApiLib};

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/ethernets", api = ethernets::EthernesApi)
    ),
    tags(
        (name = "ethernets", description = "Operations related to Ethernet entries.")
    )
)]
pub struct ApiDoc;
/// # Function: index
///
/// ## Description
/// This function handles the GET request to the root endpoint ("/").
///
/// ## Returns
/// A static string slice with the message "Hello, world!".
#[utoipa::path(context_path = "/")]
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json("Hello, world!")
}

fn config_api() -> OpenSightOSApiLib {
    let contact = ContactInformation {
        name: "Carlos A. Parra F.".to_string(),
        email: "tuto193@example.com".to_string(),
        url: "https://example.com".to_string(),
    };
    let license = LicenseInformation {
        name: "MIT".to_string(),
        url: "https://opensource.org/licenses/MIT".to_string(),
    };

    let title = "OpenSight Network API".to_string();
    let description = "REST API for OpenSight Network".to_string();
    let version = "0.1.0".to_string();
    let server_args = vec![];
    let args = vec![];

    OpenSightOSApiLib::new(
        contact,
        license,
        title,
        description,
        version,
        server_args,
        args,
    )
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let opensight_os_api_lib = config_api();
    let openapi = ApiDoc::openapi();
    opensight_os_api_lib.launch(openapi).await?
    // rocket::build()
    // Add Routers
}
