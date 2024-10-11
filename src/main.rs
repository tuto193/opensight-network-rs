mod models;
mod netplan;
mod opensight_os_api_lib;
mod routes;
#[macro_use]
extern crate rocket;
use crate::routes::ethernet::ETHERNET_ROUTES;

use opensight_os_api_lib::{ContactInformation, LicenseInformation, OpenSightOSApiLib};
use rocket_okapi::{openapi, openapi_get_routes};

/// # Function: index
///
/// ## Description
/// This function handles the GET request to the root endpoint ("/").
///
/// ## Returns
/// A static string slice with the message "Hello, world!".
#[openapi]
#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
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

#[rocket::main]
async fn main() {
    let opensight_os_api_lib = config_api();
    // rocket::build()
    let rocket = rocket::build()
        .mount("/", openapi_get_routes![index])
        .mount("/ethernets", ETHERNET_ROUTES.clone());
    opensight_os_api_lib.start(rocket).launch().await.unwrap();
    // Add Routers
}
