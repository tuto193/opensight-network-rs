use crate::models::ethernet::Ethernet;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs;
use std::io::{self, ErrorKind};
use std::process::{Command, Output};

const NETPLAN_CONFIG_PATH: &str = "/path/to/netplan/config.yaml";

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkRenderer {
    NetworkD,
    NetworkManager,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Network {
    pub version: usize,
    pub renderer: NetworkRenderer,
    pub ethernets: Vec<Ethernet>,
    // pub vlans: Vec<Vlan>,
}

pub struct Netplan {
    pub network: Network,
}

impl Netplan {
    fn run_command(cmd: Vec<&str>) -> io::Result<Output> {
        let output = Command::new("netplan").args(&cmd).output()?;

        if !output.status.success() {
            eprintln!("Command failed: {:?}", output);
            return Err(io::Error::new(ErrorKind::Other, "Command execution failed"));
        }

        println!("{}", String::from_utf8_lossy(&output.stdout));
        Ok(output)
    }

    pub fn apply() -> io::Result<()> {
        let cmd = vec!["apply"];
        Self::run_command(cmd)?;
        Ok(())
    }

    pub fn tryout() -> io::Result<()> {
        let cmd = vec![
            "try",
            "--timeout",
            "5",
            "--config-file",
            NETPLAN_CONFIG_PATH,
        ];
        Self::run_command(cmd)?;
        Ok(())
    }

    pub fn load_config() -> io::Result<Network> {
        let config_content = fs::read_to_string(NETPLAN_CONFIG_PATH)?;
        let mut netplan_config: serde_yaml::Value = serde_yaml::from_str(&config_content).unwrap_;

        if let Some(network) = netplan_config.get_mut("network") {
            if let Some(ethernets) = network.get_mut("ethernets") {
                if let Some(ethernets_map) = ethernets.as_mapping_mut() {
                    for (ethernet, value) in ethernets_map.iter_mut() {
                        if let Some(ethernet_map) = value.as_mapping_mut() {
                            ethernet_map.insert(
                                serde_yaml::Value::String("name".to_string()),
                                ethernet.clone(),
                            );
                        }
                    }
                }
            }
        }

        let network: Network = serde_yaml::from_value(netplan_config["network"].clone())?;
        Ok(network)
    }

    pub fn save_config(network: &Network) -> io::Result<()> {
        let data = serde_yaml::to_value(network)?;
        let mut network_data = serde_yaml::Mapping::new();
        network_data.insert(serde_yaml::Value::String("network".to_string()), data);

        let yaml_string = serde_yaml::to_string(&network_data)?;
        fs::write(NETPLAN_CONFIG_PATH, yaml_string)?;
        Ok(())
    }

    pub fn get_diff(compact: bool) -> io::Result<String> {
        let mut cmd = vec!["status", "--diff"];
        if compact {
            cmd[1] = "--diff-only";
        }

        let output = Self::run_command(cmd)?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
