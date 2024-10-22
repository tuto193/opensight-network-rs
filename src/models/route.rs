use crate::misc::{deserialize_ip, deserialize_ip_option, serialize_ip, serialize_ip_option};
use std::net::IpAddr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Route {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_ip_option",
        deserialize_with = "deserialize_ip_option"
    )]
    pub origin: Option<IpAddr>,
    #[serde(serialize_with = "serialize_ip", deserialize_with = "deserialize_ip")]
    pub to: IpAddr,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_ip_option",
        deserialize_with = "deserialize_ip_option"
    )]
    pub via: Option<IpAddr>,
}

impl Route {
    pub fn new(to: IpAddr, via: Option<IpAddr>, origin: Option<IpAddr>) -> Self {
        Route { origin, to, via }
    }

    pub fn set_origin(&mut self, origin: IpAddr) {
        self.origin = Some(origin);
    }

    pub fn clear_origin(&mut self) {
        self.origin = None;
    }

    pub fn set_via(&mut self, via: IpAddr) {
        self.via = Some(via);
    }

    pub fn clear_via(&mut self) {
        self.via = None;
    }

    pub fn display(&self) {
        println!("Route:");
        if let Some(origin) = &self.origin {
            println!("  Origin: {}", origin);
        } else {
            println!("  Origin: None");
        }
        println!("  To: {}", self.to);
        if let Some(via) = &self.via {
            println!("  Via: {}", via);
        } else {
            println!("  Via: None");
        }
    }
}
