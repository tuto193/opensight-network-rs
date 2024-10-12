use std::net::Ipv4Addr;

use actix_web::{App, HttpServer};
use utoipa::openapi::OpenApi;
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
        contact: ContactInformation,
        license: LicenseInformation,
        title: String,
        description: String,
        version: String,
        server_args: Vec<String>,
        args: Vec<String>,
    ) -> Self {
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

    pub async fn start(&self, openapi: &OpenApi) -> Result<(), std::io::Error> {
        let baby_clone = openapi.clone();
        HttpServer::new(move || {
            App::new().service(
                SwaggerUi::new("/docs/{_:.*}").url("/api-docs/openapi.json", baby_clone.clone()),
            )
        })
        .bind((Ipv4Addr::UNSPECIFIED, 8080))?
        .run()
        .await
    }
}
