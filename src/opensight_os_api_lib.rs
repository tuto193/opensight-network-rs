use rocket::{Build, Route};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::{rapidoc::*, swagger_ui::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Range<const MIN: i32, const MAX: i32> {
    value: i32,
}

impl<const MIN: i32, const MAX: i32> Range<MIN, MAX> {
    // Constructor method to create a new Range
    pub fn new(value: i32) -> Result<Self, String> {
        if value >= MIN && value <= MAX {
            Ok(Range { value })
        } else {
            Err(format!("Value {} is out of range ({}-{})", value, MIN, MAX))
        }
    }

    // Getter method to access the value
    pub fn value(&self) -> i32 {
        self.value
    }
}
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
    pub rocket: rocket::Rocket<Build>,
}

impl OpenSightOSApiLib {
    pub fn new(
        contact: ContactInformation,
        license: LicenseInformation,
        title: String,
        description: String,
        version: String,
        routers: Vec<Route>,
        server_args: Vec<String>,
        args: Vec<String>,
    ) -> Self {
        let rocket = rocket::build()
            .mount("/", routers.clone())
            .mount(
                "/docs",
                make_swagger_ui(&SwaggerUIConfig {
                    url: "/openapi.json".to_owned(),
                    ..Default::default()
                }),
            )
            .mount(
                "/rapidoc",
                make_rapidoc(&RapiDocConfig {
                    general: GeneralConfig {
                        spec_urls: vec![UrlObject::new("General", "/openapi.json")],
                        ..Default::default()
                    },
                    hide_show: HideShowConfig {
                        allow_spec_url_load: false,
                        allow_spec_file_load: false,
                        allow_search: false,
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            );

        OpenSightOSApiLib {
            contact,
            license,
            title,
            description,
            version,
            server_args,
            args,
            rocket,
        }
    }

    pub fn launch(self) {
        self.rocket.launch();
    }
}
