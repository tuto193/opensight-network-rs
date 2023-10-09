use crate::models::network::Network;
use serde_yml;
use std::fs;
use std::io::{self, ErrorKind};
use std::process::{Command, Output};
use std::sync::Mutex;

const NETPLAN_CONFIG_PATH: &str = "/etc/netplan/config.yaml";

struct Netplan;

pub struct NetplanStore {
    netplan: Mutex<Netplan>,
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
        let config_content = fs::read_to_string(NETPLAN_CONFIG_PATH);
        match config_content {
            Err(_) => {
                // The config file does not exist, so we create it.
                let network = Network::new();
                Self::save_config(&network)?;
                Ok(network)
            }
            Ok(config_content) => {
                // This should never fail, since the users are not allowed to
                // modify the config file directly.
                let mut netplan_config: serde_yml::Value =
                    serde_yml::from_str(&config_content).unwrap();

                if let Some(network) = netplan_config.get_mut("network") {
                    if let Some(ethernets) = network.get_mut("ethernets") {
                        if let Some(ethernets_map) = ethernets.as_mapping_mut() {
                            for (ethernet, value) in ethernets_map.iter_mut() {
                                if let Some(ethernet_map) = value.as_mapping_mut() {
                                    ethernet_map.insert(
                                        serde_yml::Value::String("name".to_string()),
                                        ethernet.clone(),
                                    );
                                }
                            }
                        }
                    }
                }

                let network: Network = serde_yml::from_value(netplan_config["network"].clone())
                    .expect("Error: there was a problem while loading the parsed yaml string.");
                Ok(network)
            }
        }
    }

    pub fn backup_config() -> io::Result<()> {
        let backup_path = format!("{}.bak", NETPLAN_CONFIG_PATH);
        fs::copy(NETPLAN_CONFIG_PATH, backup_path)?;
        Ok(())
    }

    pub fn save_config(network: &Network) -> io::Result<()> {
        Self::backup_config()?;
        // let data = serde_yml::to_value(network)
        // .expect("Error: there was a problem while serializing the Network into YAML.");
        // let mut network_data = serde_yml::Mapping::new();
        // network_data.insert(serde_yml::Value::String("network".to_string()), data);

        let yaml_string = serde_yml::to_string(&network)
            .expect("Error: couldn't serialize network into YAML string.");
        fs::write(NETPLAN_CONFIG_PATH, yaml_string)?;
        Ok(())
    }

    pub fn restore_config() {
        let backup_path = format!("{}.bak", NETPLAN_CONFIG_PATH);
        fs::copy(backup_path, NETPLAN_CONFIG_PATH).unwrap();
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
