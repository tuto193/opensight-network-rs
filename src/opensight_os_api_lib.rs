use rocket::Build;
use rocket_okapi::okapi::schemars::gen::SchemaSettings;
use rocket_okapi::settings::{OpenApiSettings, UrlObject};
use rocket_okapi::{rapidoc::*, swagger_ui::*};

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

    pub fn start(&self, rocket: rocket::Rocket<Build>) -> rocket::Rocket<Build> {
        rocket.mount(
            "/docs",
            make_swagger_ui(&SwaggerUIConfig {
                url: "/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
    }
}
