use utoipa::{
    self,
    openapi::{ContactBuilder, InfoBuilder, LicenseBuilder},
};

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
    pub fn build_info(&self) -> utoipa::openapi::Info {
        InfoBuilder::new()
            .title(self.title.clone())
            .description(Some(self.description.clone()))
            .version(self.version.clone())
            .contact(Some(
                ContactBuilder::new()
                    .name(Some(self.contact.name.clone()))
                    .email(Some(self.contact.email.clone()))
                    .url(Some(self.contact.url.clone()))
                    .build(),
            ))
            .license(Some(
                LicenseBuilder::new()
                    .name(self.license.name.clone())
                    .url(Some(self.license.url.clone()))
                    .build(),
            ))
            .build()
    }

    // pub fn configure_app_scopes(
    //     &self,
    //     scopes: Vec<(String, Box<impl Default>, utoipa::openapi::OpenApi)>,
    // ) -> App {
    //     let mut app = App::new().into_utoipa_app().openapi()
    //     for (path, default, api) in scopes {
    //         app = app.configure_scope(path, default, api);
    //     }
    //     app
    // }
}
