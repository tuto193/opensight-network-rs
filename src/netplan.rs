use crate::models::device::{Device, DynDevAttrType};
use crate::models::ethernet::Ethernet;
use crate::models::network::Network;
use crate::models::route::{DynamicRoute, Route};
use actix_web::{HttpResponse, Result};
use log::info;
use serde_yml;
use std::collections::HashMap;
use std::fs;
use std::io::{self, ErrorKind};
use std::process::Command;
use std::sync::Mutex;

const NETPLAN_CONFIG_PATH: &str = "/etc/netplan/01-network-conf.yaml";

/// A Netplan configuration manager that provides an interface to interact with
/// the system's netplan utility for network configuration management.
///
/// This struct serves as the main entry point for all netplan operations including:
/// - Loading and saving network configurations
/// - Applying network changes with validation
/// - Managing DHCP configurations and dynamic attributes
/// - Handling configuration backups and restoration
/// - Retrieving network interface status and differences
///
/// The `Netplan` struct is designed to work with the system's netplan utility
/// and manages the configuration file at `/etc/netplan/01-network-conf.yaml`.
///
/// # Examples
///
/// ```
/// use your_crate::Netplan;
///
/// let netplan = Netplan::default();
///
/// // Load current network configuration
/// let network = netplan.load_config().expect("Failed to load config");
///
/// // Apply configuration changes
/// netplan.apply().expect("Failed to apply configuration");
/// ```
#[derive(Default)]
pub struct Netplan;

#[derive(Default)]
pub struct NetplanStore {
    pub netplan: Mutex<Netplan>,
}

impl Netplan {
    /// Executes a netplan command with the given arguments.
    ///
    /// This is a helper function that runs the `netplan` command with the specified
    /// arguments and returns the stdout output as a string.
    ///
    /// # Arguments
    ///
    /// * `args` - A slice of string references containing the command arguments to pass to netplan
    ///
    /// # Returns
    ///
    /// Returns `Ok(String)` containing the stdout output if the command succeeds,
    /// or an `Err(io::Error)` if the command fails or returns a non-zero exit code.
    ///
    /// # Examples
    ///
    /// ```
    /// let output = Netplan::run_command(&["status", "--format", "yaml"])?;
    /// ```
    fn run_command(args: &[&str]) -> io::Result<String> {
        let output = Command::new("netplan").args(args).output()?;

        if !output.status.success() {
            eprintln!("Command failed: {:?}", output);
            return Err(io::Error::new(ErrorKind::Other, "Command execution failed"));
        }
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        println!("{}", result);
        Ok(result)
    }

    /// Applies the current netplan configuration to the system.
    ///
    /// This method executes `netplan apply` to activate the network configuration
    /// changes that have been written to the netplan configuration file.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the configuration is applied successfully,
    /// or an `Err(io::Error)` if the netplan apply command fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let netplan = Netplan::default();
    /// netplan.apply()?;
    /// ```
    pub fn apply(&self) -> io::Result<()> {
        Self::run_command(&["apply"])?;
        Ok(())
    }

    /// Identifies network interfaces that are missing DHCP addresses.
    ///
    /// This function examines the provided interface data to find interfaces
    /// that have `missing_dhcp4_address` or `missing_dhcp6_address` flags set to true.
    ///
    /// # Arguments
    ///
    /// * `data` - A HashMap containing interface names as keys and their configuration
    ///            mappings as values, typically from netplan status output
    ///
    /// # Returns
    ///
    /// Returns a `Vec<String>` containing the names of interfaces that are missing
    /// either DHCPv4 or DHCPv6 addresses.
    fn interfaces_with_misssing_dhcp_address(
        data: &HashMap<String, serde_yml::Mapping>,
    ) -> Vec<String> {
        let mut interfaces: Vec<String> = vec![];
        let search_strings = &["missing_dhcp4_address", "missing_dhcp6_address"];
        for (eth, eth_dict) in data.iter() {
            if let Some(missing_dhcp4) = eth_dict.get(search_strings[0]) {
                if missing_dhcp4.as_bool().unwrap() {
                    interfaces.push(eth.clone());
                }
            } else if let Some(missing_dhcp6) = eth_dict.get(search_strings[1]) {
                if missing_dhcp6.as_bool().unwrap() {
                    interfaces.push(eth.clone());
                }
            }
        }
        interfaces
    }

    /// Identifies network interfaces that are configured to expect DHCP addresses.
    ///
    /// This function examines the network configuration to find interfaces that are
    /// configured with DHCP4 enabled or DHCP6 with Router Advertisement acceptance enabled.
    ///
    /// # Arguments
    ///
    /// * `network` - A reference to the Network configuration containing ethernet interfaces
    ///
    /// # Returns
    ///
    /// Returns a `Vec<String>` containing the names of interfaces that are expecting
    /// to receive DHCP addresses based on their configuration.
    fn interfaces_expecting_dhcp_address(network: &Network) -> Vec<String> {
        let mut result = vec![];
        for (eth_name, eth) in network.get_ethernets().iter() {
            if eth.get_dhcp4()
                || (eth.get_dhcp6()
                    && eth.get_accept_ra().is_some()
                    && eth
                        .get_accept_ra()
                        .expect("Accept RA is set and it should be a bool."))
            {
                result.push(eth_name.clone());
            }
        }
        result
    }

    /// Applies the network configuration and waits for differences to resolve.
    ///
    /// This method applies the current configuration and then monitors for system state
    /// differences for up to 15 seconds. It specifically handles DHCP address assignment
    /// delays and distinguishes between expected DHCP delays and actual configuration errors.
    ///
    /// The method will wait for interfaces that are expecting DHCP addresses to receive them,
    /// but will fail immediately if there are other types of configuration differences.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Network)` with the current network configuration if successful,
    /// or an `Err(HttpResponse)` with an appropriate error message if:
    /// - The initial apply operation fails
    /// - There are persistent non-DHCP configuration differences
    /// - The configuration cannot be loaded after applying
    ///
    /// # Behavior
    ///
    /// - Waits up to 15 seconds for DHCP address assignment
    /// - Polls system state differences every second
    /// - Ignores DHCP address delays for interfaces not expecting DHCP
    /// - Fails fast on non-DHCP configuration errors
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

    /// Tests the current netplan configuration with a timeout.
    ///
    /// This method executes `netplan try` with a 5-second timeout to test the current
    /// configuration before permanently applying it. This is useful for validating
    /// configuration changes without risking network connectivity loss.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the configuration test succeeds,
    /// or an `Err(io::Error)` if the netplan try command fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let netplan = Netplan::default();
    /// netplan.tryout()?;
    /// ```
    pub fn tryout(&self) -> io::Result<()> {
        let cmd = &[
            "try",
            "--timeout",
            "5",
            "--config-file",
            NETPLAN_CONFIG_PATH,
        ];
        Self::run_command(cmd)?;
        Ok(())
    }

    /// Initializes the netplan configuration file with default network settings.
    ///
    /// This method creates a new network configuration by scanning the system's network
    /// interfaces in `/sys/class/net` and configuring any Ethernet interfaces found.
    /// It automatically enables DHCP4 for the `eth0` interface if present, while other
    /// Ethernet interfaces are added without DHCP configuration.
    ///
    /// The method performs the following operations:
    /// 1. Creates a new empty Network configuration
    /// 2. Scans `/sys/class/net` for available network interfaces
    /// 3. Identifies interfaces starting with "eth" (Ethernet interfaces)
    /// 4. Configures `eth0` with DHCP4 enabled (if present)
    /// 5. Adds other Ethernet interfaces without DHCP configuration
    /// 6. Saves the configuration to the netplan configuration file
    ///
    /// # Returns
    ///
    /// Returns `Ok(Network)` containing the newly created network configuration
    /// with all discovered Ethernet interfaces properly configured.
    ///
    /// Returns `Err(io::Error)` if:
    /// - Unable to read the `/sys/class/net` directory
    /// - File system operations fail during interface discovery
    /// - Saving the configuration to the netplan file fails
    ///
    /// # Examples
    ///
    /// ```
    /// let mut netplan = Netplan::default();
    /// let network = netplan.initalize_config_file()?;
    /// println!("Initialized {} ethernet interfaces", network.get_ethernets().len());
    /// ```
    ///
    /// # Behavior
    ///
    /// - Only processes interfaces with names starting with "eth"
    /// - Automatically enables DHCP4 for `eth0` interface
    /// - Other Ethernet interfaces are added with default settings
    /// - Creates and saves the configuration file immediately
    /// - Overwrites any existing configuration
    pub fn initalize_config_file(&mut self) -> Result<Network, io::Error> {
        let mut network = Network::new();
        fs::read_dir("/sys/class/net")?.for_each(|entry| {
            let eth_name = entry
                .expect("All entries in /sys/class/net should be proper entries")
                .file_name()
                .into_string()
                .unwrap();
            if eth_name.starts_with("eth") {
                let mut iface = Ethernet::new(eth_name.clone());
                iface.set_dhcp4(eth_name == "eth0");
                network.add_ethernet(&iface);
            }
        });
        Netplan::save_config(&self, &network)?;
        Ok(network)
    }

    /// Extracts dynamic network attributes from netplan status output.
    ///
    /// This function parses the netplan status YAML data to extract dynamic network
    /// attributes such as IP addresses, DNS addresses, and routes for ethernet interfaces.
    /// It processes the runtime network state information that netplan collects from the system.
    ///
    /// # Arguments
    ///
    /// * `data` - A YAML mapping containing the netplan status output with interface information
    ///
    /// # Returns
    ///
    /// Returns a nested HashMap where:
    /// - Outer key: Interface name (String)
    /// - Inner key: Dynamic attribute type (DynDevAttrType)
    /// - Value: Vector of formatted attribute values (Vec<String>)
    ///
    /// The function extracts:
    /// - **Addresses**: IP addresses with their flags and prefixes
    /// - **DNS Addresses**: DNS server addresses
    /// - **Routes**: Dynamic routing information
    ///
    /// # Behavior
    ///
    /// - Only processes interfaces with type "ethernet"
    /// - Formats addresses with flags in parentheses
    /// - Includes prefix information when available
    /// - Filters out invalid route entries and logs parsing errors
    /// - Returns empty HashMap entries for interfaces without dynamic attributes
    fn get_dynamic_attributes_from_netplan_status(
        data: serde_yml::Mapping,
    ) -> HashMap<String, HashMap<DynDevAttrType, Vec<String>>> {
        let mut result = HashMap::new();
        data.iter().for_each(|(eth, data)| {
            if data
                .get("type")
                .expect("All interfaces in the netplan output should have a `type`.")
                .as_str()
                .unwrap()
                == "ethernet"
            {
                let eth_name = eth.as_str().unwrap().to_string();
                // First initialize the HashMap for the interface
                result.insert(
                    eth_name.clone(),
                    HashMap::new(), // HashMap::from([
                                    //     (DynDevAttrType::Addresses, vec![]),
                                    //     (DynDevAttrType::DnsAddresses, vec![]),
                                    //     (DynDevAttrType::Routes, vec![]),
                                    // ]),
                );
                // Add the dynamic addresses first
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
                    result.entry(eth_name.clone()).and_modify(|dyn_attrs| {
                        dyn_attrs.insert(DynDevAttrType::Addresses, found_addresses);
                    });
                }
                // Then the dns_addresses
                if let Some(dns_addresses_sequence) = data.get("dns_addresses") {
                    let dns_addresses_vec: Vec<String> = dns_addresses_sequence
                        .as_sequence()
                        .unwrap()
                        .iter()
                        .map(|addr_val| addr_val.as_str().unwrap().to_string())
                        .collect();
                    result.entry(eth_name.clone()).and_modify(|dyn_attrs| {
                        dyn_attrs.insert(DynDevAttrType::DnsAddresses, dns_addresses_vec);
                    });
                }
                // And finally the dynamic_routes
                if let Some(routes_sequence_of_maps) = data.get("routes") {
                    let dynamic_routes_vec: Vec<String> = routes_sequence_of_maps
                        .as_sequence()
                        .unwrap()
                        .iter()
                        .map(|route_map| {
                            let parsed_dynamic_route: Result<DynamicRoute, serde_yml::Error> =
                                serde_yml::from_value(route_map.clone());
                            match parsed_dynamic_route {
                                Ok(route) => Some(route),
                                Err(err) => {
                                    info!("Failed to parse dynamic route: {err}");
                                    None
                                }
                            }
                        })
                        .filter(|r| r.is_some())
                        .map(|route_option| {
                            let dyn_route = route_option.unwrap();
                            format!("{dyn_route:?}")
                        })
                        .collect();
                    result.entry(eth_name).and_modify(|dyn_attrs| {
                        dyn_attrs.insert(DynDevAttrType::Routes, dynamic_routes_vec);
                    });
                }
            }
        });
        result
    }

    /// Loads the current network configuration from netplan and system state.
    ///
    /// This method combines information from multiple sources to build a complete
    /// Network configuration:
    /// - Netplan status output for dynamic attributes
    /// - System state differences
    /// - The netplan configuration file
    ///
    /// If the configuration file doesn't exist, it creates a default configuration
    /// with eth0 interface configured for DHCP4 if the interface exists in the system.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Network)` containing the complete network configuration with:
    /// - Static configuration from the netplan file
    /// - Dynamic attributes from system state
    /// - System state differences
    /// - Route information properly parsed and organized
    ///
    /// Returns `Err(io::Error)` if:
    /// - Netplan commands fail
    /// - File system operations fail
    /// - YAML parsing fails
    ///
    /// # Behavior
    ///
    /// **When configuration file exists:**
    /// - Parses the existing YAML configuration
    /// - Adds interface names to ethernet configurations
    /// - Converts route sequences to mappings for easier manipulation
    /// - Merges system state differences
    /// - Applies dynamic attributes from netplan status
    ///
    /// **When configuration file doesn't exist:**
    /// - Checks for eth0 interface in `/sys/class/net`
    /// - Creates default DHCP4 configuration for eth0 if present
    /// - Saves the new configuration to disk
    /// - Returns the newly created configuration
    pub fn load_config(&self) -> io::Result<Network> {
        let status_yaml: serde_yml::Mapping = serde_yml::from_str(&Self::run_command(&[
            "status", "--format", "yaml", "--all",
        ])?)
        .unwrap();
        let interfaces_dynamic_attributes =
            Self::get_dynamic_attributes_from_netplan_status(status_yaml);
        let diff = self.get_diff()?;

        let config_content = fs::read_to_string(NETPLAN_CONFIG_PATH);
        match config_content {
            Err(_) => {
                // The config file does not exist, so we create it.
                // Check for existing ethernets in /sys/class/net
                let mut result = Network::new();
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
                    if let Some(eth0_diff) = diff.get("eth0") {
                        iface.set_system_state_diff(serde_yml::from_value(
                            eth0_diff.get("system_state")
                                .expect("Ethernet should have system_state")
                                .clone(),
                        ).expect("Mapping from system state should be made (at least) an empty mapping."));
                    }
                    if let Some(eth0_dyn_attributes) = interfaces_dynamic_attributes.get("eth0") {
                        iface.set_dynamic_attributes_from_yaml(eth0_dyn_attributes.clone());
                    }
                    base_interface = Some(iface);
                }

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

    /// Creates a backup copy of the current netplan configuration file.
    ///
    /// This static method creates a backup of the netplan configuration file by
    /// copying it to a `.bak` extension. This is typically called before making
    /// configuration changes to allow for restoration if needed.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the backup is created successfully,
    /// or an `Err(io::Error)` if the file copy operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// Netplan::backup_config()?;
    /// ```
    pub fn backup_config() -> io::Result<()> {
        let backup_path = format!("{}.bak", NETPLAN_CONFIG_PATH);
        fs::copy(NETPLAN_CONFIG_PATH, backup_path)?;
        Ok(())
    }

    /// Saves the network configuration to the netplan configuration file.
    ///
    /// This method serializes the provided Network configuration to YAML format
    /// and writes it to the netplan configuration file. It automatically creates
    /// a backup of the existing configuration before saving the new one.
    ///
    /// # Arguments
    ///
    /// * `network` - A reference to the Network configuration to save
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the configuration is saved successfully,
    /// or an `Err(io::Error)` if:
    /// - The backup operation fails
    /// - YAML serialization fails
    /// - File write operation fails
    ///
    /// # Examples
    ///
    /// ```
    /// let netplan = Netplan::default();
    /// let network = Network::new();
    /// netplan.save_config(&network)?;
    /// ```
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

    /// Restores the netplan configuration from the backup file.
    ///
    /// This method restores the netplan configuration by copying the backup file
    /// (created by `backup_config()`) back to the main configuration file location.
    /// This is typically used when a configuration change needs to be reverted.
    ///
    /// # Panics
    ///
    /// This method will panic if the backup file doesn't exist or if the copy
    /// operation fails. It uses `unwrap()` on the file copy operation.
    ///
    /// # Examples
    ///
    /// ```
    /// let netplan = Netplan::default();
    /// netplan.restore_config(); // Restores from backup
    /// ```
    pub fn restore_config(&self) {
        let backup_path = format!("{}.bak", NETPLAN_CONFIG_PATH);
        fs::copy(backup_path, NETPLAN_CONFIG_PATH).unwrap();
    }

    /// Retrieves the differences between netplan configuration and system state.
    ///
    /// This method executes `netplan status --diff-only --format yaml` to get
    /// information about differences between the configured state and the actual
    /// system state. It parses the output to extract system state differences
    /// for each managed interface.
    ///
    /// # Returns
    ///
    /// Returns `Ok(HashMap<String, serde_yml::Mapping>)` where:
    /// - Key: Interface name
    /// - Value: YAML mapping containing the system state differences for that interface
    ///
    /// Returns `Err(io::Error)` if the netplan command fails.
    ///
    /// # Behavior
    ///
    /// - Only includes interfaces that have system_state differences
    /// - Filters out interfaces without system state information
    /// - Parses YAML output to extract structured difference data
    ///
    /// # Examples
    ///
    /// ```
    /// let netplan = Netplan::default();
    /// let diff = netplan.get_diff()?;
    /// for (interface, differences) in diff {
    ///     println!("Interface {} has differences: {:?}", interface, differences);
    /// }
    /// ```
    pub fn get_diff(&self) -> io::Result<HashMap<String, serde_yml::Mapping>> {
        let cmd = &["status", "--diff-only", "--format", "yaml"];
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

    /// Saves the network configuration and applies it with difference monitoring.
    ///
    /// This method combines the save and apply operations, providing a complete
    /// workflow for updating the network configuration. It first saves the provided
    /// configuration to the netplan file, then applies it using `apply_with_diff()`
    /// which monitors for system state convergence.
    ///
    /// # Arguments
    ///
    /// * `network` - A reference to the Network configuration to save and apply
    ///
    /// # Returns
    ///
    /// Returns `Ok(Network)` containing the final network configuration after
    /// successful save and apply operations.
    ///
    /// Returns `Err(HttpResponse)` with an appropriate error response if:
    /// - The save operation fails
    /// - The apply operation fails
    /// - There are persistent configuration differences
    ///
    /// # Examples
    ///
    /// ```
    /// let netplan = Netplan::default();
    /// let mut network = netplan.load_config()?;
    /// // Modify network configuration...
    /// let updated_network = netplan.save_and_apply(&network)?;
    /// ```
    pub fn save_and_apply(&self, network: &Network) -> Result<Network, HttpResponse> {
        match self.save_config(network) {
            Ok(_) => (),
            Err(err) => return Err(HttpResponse::InternalServerError().body(err.to_string())),
        }
        self.apply_with_diff()
    }

    /// Retrieves all Ethernet interface names from the netplan status output.
    ///
    /// This method executes `netplan status --diff-only --format yaml` and parses
    /// the YAML output to extract all Ethernet interface names (those starting with "eth").
    /// It searches through multiple sections of the netplan status output:
    /// - `interfaces`: Currently managed interfaces
    /// - `missing_interfaces_netplan`: Interfaces missing from netplan configuration
    /// - `missing_interfaces_system`: Interfaces missing from the system
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<String>)` containing the names of all found Ethernet interfaces,
    /// or an `Err(io::Error)` if the netplan command fails or if the expected YAML
    /// structure is not found in the output.
    ///
    /// # Panics
    ///
    /// This method will panic if:
    /// - The netplan status output is not valid YAML
    /// - Required sections (`interfaces`, `missing_interfaces_netplan`, `missing_interfaces_system`)
    ///   are missing from the YAML output
    /// - Interface names cannot be converted to strings
    pub fn get_all_ethernets(&self) -> io::Result<Vec<String>> {
        let output = Self::run_command(&["status", "--diff-only", "--format", "yaml"])?;
        let mut result: Vec<String> = Vec::new();
        let yaml_output: serde_yml::Mapping = serde_yml::from_str(&output)
            .expect("Netplan status command should be YAML, but failed to parse");
        // Normal interfaces
        yaml_output
            .get("interfaces")
            .expect("Output of diff should contain managed interfaces")
            .as_mapping()
            .unwrap()
            .iter()
            .for_each(|(interface, _interface_data)| {
                if interface.as_str().unwrap().starts_with("eth") {
                    result.push(interface.as_str().unwrap().to_string());
                }
            });
        // missing_interfaces_netplan
        yaml_output
            .get("missing_interfaces_netplan")
            .expect("Output of diff should contain missing interfaces")
            .as_mapping()
            .unwrap()
            .iter()
            .for_each(|(interface, _interface_data)| {
                if interface.as_str().unwrap().starts_with("eth") {
                    result.push(interface.as_str().unwrap().to_string());
                }
            });
        // missing_interfaces_system
        yaml_output
            .get("missing_interfaces_system")
            .expect("Output of diff should contain missing system interfaces")
            .as_mapping()
            .unwrap()
            .iter()
            .for_each(|(interface, _interface_data)| {
                if interface.as_str().unwrap().starts_with("eth") {
                    result.push(interface.as_str().unwrap().to_string());
                }
            });
        // missing_interfaces_system
        yaml_output
            .get("missing_interfaces_system")
            .expect("Output of diff should contain missing system interfaces")
            .as_mapping()
            .unwrap()
            .iter()
            .for_each(|(interface, _interface_data)| {
                if interface.as_str().unwrap().starts_with("eth") {
                    result.push(interface.as_str().unwrap().to_string());
                }
            });
        Ok(result)
    }
}
