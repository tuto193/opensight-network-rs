use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, Default)]
#[serde(rename_all = "kebab-case")]
pub struct HostInfo {
    hostname: String,
}

#[derive(Default)]
pub struct HostInfoStore {
    pub host_info: Mutex<HostInfo>,
}

impl HostInfo {
    pub fn new(hostname: String) -> Self {
        Self { hostname }
    }

    pub fn get_hostname(&self) -> &String {
        &self.hostname
    }

    pub fn set_hostname(&mut self, hostname: String) {
        self.hostname = hostname;
    }
}
