use std::process::Command;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, Default)]
#[serde(rename_all = "kebab-case")]
pub struct HostInfo;

#[derive(Default)]
pub struct HostInfoStore {
    pub host_info: Mutex<HostInfo>,
}

impl HostInfo {
    fn _run_hostnamectl(args: &[&str]) -> Result<String, std::io::Error> {
        // Implementation of _run_hostnamectl
        let result = Command::new("hostnamectl")
            .args(args)
            .output()?
            .stdout
            .into_iter()
            .map(|byte| byte as char)
            .collect::<String>();
        Ok(result)
    }

    pub fn get_hostname() -> Result<String, std::io::Error> {
        Self::_run_hostnamectl(&["hostname"])
    }

    pub fn set_hostname(hostname: &String) {
        Self::_run_hostnamectl(&[hostname]);
    }
}
