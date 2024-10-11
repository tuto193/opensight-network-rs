use crate::models::network::Network;
use serde_yaml;
use std::fs;
use std::io::{self, ErrorKind};
use std::process::{Command, Output};

const NETPLAN_CONFIG_PATH: &str = "/etc/netplan/config.yaml";
pub struct Netplan;

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
        let config_content = fs::read_to_string(NETPLAN_CONFIG_PATH).unwrap_or("".to_string());
        let mut netplan_config: serde_yaml::Value = serde_yaml::from_str(&config_content).unwrap();

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

        let network: Network = serde_yaml::from_value(netplan_config["network"].clone())
            .expect("Error: there was a problem while loading the config file.");
        Ok(network)
    }

    pub fn save_config(network: &Network) -> io::Result<()> {
        let data = serde_yaml::to_value(network)
            .expect("Error: there was a problem while serializing the Network into YAML.");
        let mut network_data = serde_yaml::Mapping::new();
        network_data.insert(serde_yaml::Value::String("network".to_string()), data);

        let yaml_string = serde_yaml::to_string(&network_data)
            .unwrap_or_else(|_| panic!("Error: couldn't convert YAML into string."));
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
