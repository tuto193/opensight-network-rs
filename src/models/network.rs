use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::ethernet::Ethernet;

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkRenderer {
    #[serde(rename = "networkd")]
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

impl Default for Network {
    fn default() -> Self {
        Self::new()
    }
}

impl Network {
    pub fn new() -> Self {
        Self {
            version: 2,
            renderer: NetworkRenderer::NetworkD,
            ethernets: HashMap::new(),
        }
    }

    pub fn get_ethernets(&self) -> &HashMap<String, Ethernet> {
        &self.ethernets
    }

    pub fn add_ethernet(&mut self, ethernet: Ethernet) {
        self.ethernets.insert(ethernet.get_name().clone(), ethernet);
    }
}
