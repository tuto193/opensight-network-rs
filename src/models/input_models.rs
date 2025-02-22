use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::device::{MTU, MTUV6};

#[derive(Deserialize)]
pub struct ScopeQuery {
    pub scope: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct InputDevice {
    pub accept_ra: Option<bool>,
    pub dhcp4: Option<bool>,
    pub dhcp6: Option<bool>,
    pub mtu: Option<MTU>,
    pub ipv6_mtu: Option<MTUV6>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct InputRoute {
    pub to: String,
    pub from: Option<String>,
    pub via: Option<String>,
}
