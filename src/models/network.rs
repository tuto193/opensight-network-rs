use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::ethernet::Ethernet;

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkRenderer {
    NetworkD,
    NetworkManager,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Network {
    pub version: usize,
    pub renderer: NetworkRenderer,
    pub ethernets: HashMap<String, Ethernet>,
    // pub vlans: Vec<Vlan>,
}

impl Network {
    pub fn new() -> Self {
        Self {
            version: 2,
            renderer: NetworkRenderer::NetworkD,
            ethernets: HashMap::new(),
        }
    }
}
