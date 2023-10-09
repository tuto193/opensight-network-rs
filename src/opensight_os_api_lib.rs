use std::net::Ipv4Addr;

use actix_web::{App, HttpServer};
use utoipa;
use utoipa_actix_web::AppExt;
use utoipa_swagger_ui::SwaggerUi;

pub struct ContactInformation {
    pub name: String,
    pub email: String,
    pub url: String,
}

pub struct LicenseInformation {
    pub name: String,
    pub url: String,
}

pub struct OpenSightOSApiLib {
    pub contact: ContactInformation,
    pub license: LicenseInformation,
    pub title: String,
    pub description: String,
    pub version: String,
    pub server_args: Vec<String>,
    pub args: Vec<String>,
}

impl OpenSightOSApiLib {
    pub fn new(
        title: String,
        description: String,
        version: String,
        server_args: Vec<String>,
        args: Vec<String>,
    ) -> Self {
        let contact = ContactInformation {
            name: "Greenbone AG".to_string(),
            email: "info@greenbone.net".to_string(),
            url: "https://www.greenbone.net".to_string(),
        };
        let license = LicenseInformation {
            name: "GNU Affero General Public License v3.0 or later".to_string(),
            url: "https://www.gnu.org/licenses/agpl-3.0-standalone.html".to_string(),
        };
        Self {
            contact,
            license,
            title,
            description,
            version,
            server_args,
            args,
        }
    }

    pub fn app(&self, openapi: utoipa::openapi::OpenApi) -> App {
        let baby_clone = openapi;
        App::new()
            .into_utoipa_app()
            .openapi(baby_clone)
            .openapi_service(|api| {
                SwaggerUi::new("/docs/{_:.*}").url("/api-docs/openapi.json", api)
            })
            .into_app()
    }
    pub async fn start(&self, openapi: utoipa::openapi::OpenApi) -> Result<(), std::io::Error> {
        let baby_clone = openapi;
        HttpServer::new(move || {
            App::new().service(
                SwaggerUi::new("/docs/{_:.*}").url("/api-docs/openapi.json", baby_clone.clone()),
            )
        })
        .bind((Ipv4Addr::LOCALHOST, 8080))?
        .run()
        .await
    }
}
