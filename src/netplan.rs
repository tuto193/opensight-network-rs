use crate::models::device::Device;
use crate::models::ethernet::Ethernet;
use crate::models::network::Network;
use crate::models::route::Route;
use actix_web::{HttpResponse, Responder, Result};
use serde_yml;
use std::collections::HashMap;
use std::fs;
use std::io::{self, ErrorKind};
use std::process::{Command, Output};
use std::sync::Mutex;

const NETPLAN_CONFIG_PATH: &str = "/etc/netplan/01-network-conf.yaml";

#[derive(Default)]
pub struct Netplan;

#[derive(Default)]
pub struct NetplanStore {
    pub netplan: Mutex<Netplan>,
}

impl Netplan {
    fn run_command(cmd: Vec<&str>) -> io::Result<String> {
        let output = Command::new("netplan").args(&cmd).output()?;

        if !output.status.success() {
            eprintln!("Command failed: {:?}", output);
            return Err(io::Error::new(ErrorKind::Other, "Command execution failed"));
        }
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        println!("{}", result);
        Ok(result)
    }

    pub fn apply(&self) -> io::Result<()> {
        let cmd = vec!["apply"];
        Self::run_command(cmd)?;
        Ok(())
    }

    fn interfaces_with_misssing_dhcp_address(
        data: &HashMap<String, serde_yml::Mapping>,
    ) -> Vec<String> {
        let mut interfaces: Vec<String> = vec![];
        let search_strings = vec!["missing_dhcp4_address", "missing_dhcp6_address"];
        for (eth, eth_dict) in data.iter() {
            if let Some(missing_dhcp4) = eth_dict.get(&search_strings[0]) {
                if missing_dhcp4.as_bool().unwrap() {
                    interfaces.push(eth.clone());
                }
            } else if let Some(missing_dhcp6) = eth_dict.get(&search_strings[1]) {
                if missing_dhcp6.as_bool().unwrap() {
                    interfaces.push(eth.clone());
                }
            }
        }
        interfaces
    }

    fn interfaces_expecting_dhcp_address(network: &Network) -> Vec<String> {
        let mut result = vec![];
        for (eth_name, eth) in network.ethernets.iter() {
            if eth.dhcp4
                || (eth.dhcp6
                    && eth.accept_ra.is_some()
                    && eth
                        .accept_ra
                        .expect("Accept RA is set and it should be a bool."))
            {
                result.push(eth_name.clone());
            }
        }
        result
    }

    pub fn apply_with_diff(&self) -> Result<Network, HttpResponse> {
        if self.apply().is_err() {
            return Err(HttpResponse::InternalServerError()
                .body("There was a problem applying the current config."));
        }
        const SECONDS_TO_WAIT: i32 = 15;
        let mut there_are_differences = false;
        for _waited in 0..SECONDS_TO_WAIT {
            match self.get_diff() {
                Ok(diff) => {
                    if diff.is_empty() {
                        there_are_differences = false;
                        break;
                    }
                    there_are_differences = true;
                    let ifaces_without_dhcp_address =
                        Self::interfaces_with_misssing_dhcp_address(&diff);
                    if ifaces_without_dhcp_address.is_empty() {
                        // There is an error, but it's not the addresses
                        return Err(HttpResponse::InternalServerError()
                            .body("There are unchecked system_state differences"));
                    }
                    let ifaces_expecting_dhcp_address =
                        Self::interfaces_expecting_dhcp_address(&self.load_config().unwrap());
                    let affected_ifaces = ifaces_without_dhcp_address
                        .iter()
                        .filter(|iface| ifaces_expecting_dhcp_address.contains(*iface))
                        .collect::<Vec<&String>>();
                    if !affected_ifaces.is_empty() {
                        // Sleep for 1 second and try again
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                    // Only diff is from dhcp, but no affected interfaces present.
                    there_are_differences = false;
                    break;
                }
                Err(_) => {
                    return Err(HttpResponse::InternalServerError()
                        .body("The config was not applied correctly."));
                }
            }
        }
        if there_are_differences {
            // LOG a warning, so user is aware.
        }
        match self.load_config() {
            Ok(network) => Ok(network),
            Err(_) => Err(HttpResponse::InternalServerError()
                .body("There was an error while loading the config.")),
        }
    }
    pub fn tryout(&self) -> io::Result<()> {
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

    fn get_dynamic_addresses_from_netplan_status(
        data: serde_yml::Mapping,
    ) -> HashMap<String, Vec<String>> {
        let mut result: HashMap<String, Vec<String>> = HashMap::new();
        data.iter().for_each(|(eth, data)| {
            if data
                .get("type")
                .expect("All interfaces in the netplan output should have a `type`.")
                .as_str()
                .unwrap()
                == "ethernet"
            {
                if let Some(addresses_dict) = data.get("addresses") {
                    let mut found_addresses: Vec<String> = vec![];
                    let addresses_dict = addresses_dict.as_mapping().unwrap();
                    addresses_dict
                        .iter()
                        .for_each(|(address_type, properties)| {
                            let properties = properties.as_mapping().unwrap();
                            if let Some(flags) = properties.get("flags") {
                                let mut parsed_address = address_type.as_str().unwrap().to_string();
                                let flags: Vec<String> = flags
                                    .as_sequence()
                                    .unwrap()
                                    .iter()
                                    .map(|entry| entry.as_str().unwrap().to_string())
                                    .collect();
                                let flags: String = flags.join(", ");
                                let suffix = format!("({})", flags);
                                if let Some(prefix) = properties.get("prefix") {
                                    parsed_address.insert_str(0, prefix.as_str().unwrap());
                                }
                                parsed_address.push_str(&suffix);
                                found_addresses.push(parsed_address);
                            }
                        });
                    result.insert(eth.as_str().unwrap().to_string(), found_addresses);
                }
            }
        });
        result
    }

    pub fn load_config(&self) -> io::Result<Network> {
        let status_yaml: serde_yml::Mapping = serde_yml::from_str(&Self::run_command(vec![
            "status", "--format", "yaml", "--all",
        ])?)
        .unwrap();
        let interfaces_dynamic_addresses =
            Self::get_dynamic_addresses_from_netplan_status(status_yaml);
        let diff = self.get_diff()?;

        let config_content = fs::read_to_string(NETPLAN_CONFIG_PATH);
        match config_content {
            Err(_) => {
                // The config file does not exist, so we create it.
                // Check for existing ethernets in /sys/class/net
                let mut base_interface: Option<Ethernet> = None;
                if fs::read_dir("/sys/class/net")?.any(|entry| {
                    entry
                        .expect("All entries in /sys/class/net should be proper entries")
                        .file_name()
                        .into_string()
                        .unwrap()
                        == "eth0"
                }) {
                    let mut iface = Ethernet::new("eth0".to_string());
                    iface.set_dhcp4(true);
                    base_interface = Some(iface);
                }

                let mut result = Network::new();
                if let Some(base_interface) = base_interface {
                    result.add_ethernet(&base_interface);
                }
                self.save_config(&result)?;
                Ok(result)
            }
            Ok(config_content) => {
                // This should never fail, since the users are not allowed to
                // modify the config file directly.
                let mut netplan_config: serde_yml::Value =
                    serde_yml::from_str(&config_content).unwrap();

                if let Some(network) = netplan_config.get_mut("network") {
                    if let Some(ethernets) = network.get_mut("ethernets") {
                        if let Some(ethernets_map) = ethernets.as_mapping_mut() {
                            for (ethernet_name, actual_ethernet) in ethernets_map.iter_mut() {
                                if let Some(ethernet_map) = actual_ethernet.as_mapping_mut() {
                                    ethernet_map.insert("name".into(), ethernet_name.clone());
                                    // Make sure to parse the routes, since they don't come as a mapping but rather as sequence
                                    // Need to turn routes from a sequence to a mapping
                                    if let Some(routes) = ethernet_map.get_mut("routes") {
                                        if let Some(routes_seq) = routes.as_sequence_mut() {
                                            let mut new_routes = serde_yml::Mapping::new();
                                            for route in routes_seq.iter() {
                                                let parsed_route: Route =
                                                    serde_yml::from_value(route.clone())
                                                        .expect("Error: there was a problem while parsing Route yaml string.");
                                                new_routes.insert(
                                                    serde_yml::Value::String(parsed_route.id()),
                                                    route.clone(),
                                                );
                                            }
                                            ethernet_map.insert("routes".into(), new_routes.into());
                                        }
                                    }
                                    // Also add the system_state, if it exists
                                    if let Some(interface_diff) =
                                        diff.get(ethernet_name.as_str().unwrap())
                                    {
                                        if let Some(system_state) =
                                            interface_diff.get("system_state")
                                        {
                                            if let Some(system_state_mapping) =
                                                system_state.as_mapping()
                                            {
                                                if !system_state_mapping.is_empty() {
                                                    ethernet_map.insert(
                                                        "system_state".into(),
                                                        system_state.clone(),
                                                    );
                                                }
                                            }
                                        }
                                    }
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

    pub fn save_config(&self, network: &Network) -> io::Result<()> {
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

    pub fn restore_config(&self) {
        let backup_path = format!("{}.bak", NETPLAN_CONFIG_PATH);
        fs::copy(backup_path, NETPLAN_CONFIG_PATH).unwrap();
    }

    pub fn get_diff(&self) -> io::Result<HashMap<String, serde_yml::Mapping>> {
        let cmd = vec!["status", "--diff-only", "--format", "yaml"];
        let mut result: HashMap<String, serde_yml::Mapping> = HashMap::new();
        let output = Self::run_command(cmd)?;
        let yaml_output: serde_yml::Mapping = serde_yml::from_str(&output).unwrap();
        let managed_interfaces = yaml_output
            .get("interfaces")
            .expect("Output of diff should contain managed interfaces")
            .as_mapping()
            .unwrap();
        managed_interfaces
            .iter()
            .for_each(|(interface, interface_data)| {
                let iface_date = interface_data.as_mapping().unwrap();
                if let Some(system_state) = iface_date.get("system_state") {
                    result.insert(
                        interface.as_str().unwrap().to_string(),
                        system_state.as_mapping().unwrap().clone(),
                    );
                }
            });
        Ok(result)
    }

    pub fn save_and_apply(&self, network: &Network) -> Result<Network, HttpResponse> {
        self.save_config(network);
        self.apply_with_diff()
    }

    pub fn get_all_ethernets(&self) -> Vec<String> {
        let output = Self::run_command(vec!["status", "--diff-only", "--format", "yaml"])
            .expect("Netplan status command should not fail");
        let yaml_output: serde_yml::Mapping = serde_yml::from_str(&output).unwrap();
        let managed_interfaces = yaml_output
            .get("interfaces")
            .expect("Output of diff should contain managed interfaces")
            .as_mapping()
            .unwrap();
        managed_interfaces
            .iter()
            .for_each(|(interface, interface_data)| {
                let iface_date = interface_data.as_mapping().unwrap();
                if let Some(system_state) = iface_date.get("system_state") {
                    result.push(interface.as_str().unwrap().to_string());
                }
            });
        result
    }
}
