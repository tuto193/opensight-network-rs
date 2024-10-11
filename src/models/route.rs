use std::net::IpAddr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Route {
    pub origin: Option<IpAddr>,
    pub to: IpAddr,
    pub via: Option<IpAddr>,
}

impl Route {
    pub fn new(origin: Option<IpAddr>, to: IpAddr, via: Option<IpAddr>) -> Self {
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
