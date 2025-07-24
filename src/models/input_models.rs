use std::net::{AddrParseError, SocketAddr};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

use super::device::{MTU, MTUV6};

#[derive(Deserialize)]
pub struct ScopeQuery {
    pub scope: String,
}

#[derive(Serialize, Deserialize, ToSchema, Validate)]
pub struct InputDevice {
    pub accept_ra: Option<bool>,
    pub dhcp4: Option<bool>,
    pub dhcp6: Option<bool>,
    pub mtu: Option<MTU>,
    pub ipv6_mtu: Option<MTUV6>,
}

#[derive(Serialize, Deserialize, ToSchema, Validate)]
pub struct InputRoute {
    #[validate(custom(function = "validate_to"))]
    pub to: String,
    #[validate(ip)]
    pub from: Option<String>,
    #[validate(ip)]
    pub via: Option<String>,
}

fn validate_to(to: &str) -> Result<(), ValidationError> {
    if to == "default" {
        Ok(())
    } else {
        let result: Result<SocketAddr, AddrParseError> = to.parse();
        if result.is_ok() {
            return Ok(());
        }
        Err(ValidationError::new("invalid_to"))
    }
}
