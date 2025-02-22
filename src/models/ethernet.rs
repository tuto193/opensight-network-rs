use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, SocketAddr},
};

use serde::{Deserialize, Serialize};

use super::{
    device::{Device, MTU, MTUV6},
    input_models::InputDevice,
    nameservers::Nameservers,
    route::Route,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Ethernet {
    #[serde(skip_serializing)]
    name: String,
    dhcp4: bool,
    dhcp6: bool,
    mtu: Option<MTU>,
    ipv6_mtu: Option<MTUV6>,
    accept_ra: Option<bool>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    routes: HashMap<String, Route>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    addresses: HashSet<SocketAddr>,
    nameservers: Nameservers,
    #[serde(skip_serializing)]
    dynamic_addresses: Vec<String>,
    #[serde(skip_serializing)]
    system_state: HashMap<String, serde_yml::Value>,
}

impl Ethernet {
    pub fn new(name: String) -> Self {
        Self {
            name,
            dhcp4: false,
            dhcp6: false,
            mtu: None,
            ipv6_mtu: None,
            accept_ra: None,
            routes: HashMap::new(),
            addresses: HashSet::new(),
            nameservers: Nameservers::new(),
            dynamic_addresses: Vec::new(),
            system_state: HashMap::new(),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl Device for Ethernet {
    fn from_input_device(name: &str, input_device: &InputDevice) -> Self {
        let mut result = Self::new(name.to_string());
        if let Some(dhcp) = input_device.dhcp4 {
            result.set_dhcp4(dhcp);
        }
        if let Some(dhcp6) = input_device.dhcp6 {
            result.set_dhcp6(dhcp6);
        }

        result.set_accept_ra(input_device.accept_ra);
        result.set_mtu(input_device.mtu);
        result.set_ipv6_mtu(input_device.ipv6_mtu);

        result
    }

    fn set_dhcp4(&mut self, set: bool) {
        self.dhcp4 = set;
    }

    fn get_dhcp4(&self) -> bool {
        self.dhcp4
    }

    fn get_dhcp6(&self) -> bool {
        self.dhcp6
    }

    fn set_dhcp6(&mut self, set: bool) {
        self.dhcp6 = set;
    }

    fn set_accept_ra(&mut self, set: Option<bool>) {
        self.accept_ra = set;
    }

    fn get_accept_ra(&self) -> Option<bool> {
        self.accept_ra
    }

    fn get_mtu(&self) -> Option<MTU> {
        self.mtu
    }

    fn set_mtu(&mut self, mtu: Option<MTU>) {
        self.mtu = mtu;
    }

    fn get_addresses(&self) -> HashSet<SocketAddr> {
        self.addresses.clone()
    }

    fn get_nameservers(&self) -> super::nameservers::Nameservers {
        self.nameservers.clone()
    }

    fn add_nameservers(&mut self, nameservers: super::nameservers::Nameservers) {
        self.nameservers = nameservers;
    }

    fn get_routes(&self) -> HashMap<String, Route> {
        self.routes.clone()
    }

    fn add_nameservers_search(&mut self, search: &String) {
        self.nameservers.add_search(search);
    }

    fn add_nameservers_address(&mut self, address: &IpAddr) {
        self.nameservers.add_address(address);
    }

    fn delete_nameservers_search(&mut self, search: &String) -> bool {
        self.nameservers.remove_search(search)
    }

    fn delete_nameservers_address(&mut self, address: &IpAddr) -> bool {
        self.nameservers.remove_address(address)
    }

    fn delete_route(&mut self, route_id: &String) -> bool {
        self.routes.remove(route_id).is_some()
    }

    fn delete_address(&mut self, address: &SocketAddr) -> bool {
        self.addresses.remove(address)
    }

    fn delete_all_routes(&mut self) {
        self.routes = HashMap::new();
    }

    fn add_address(&mut self, address: &SocketAddr) {
        self.addresses.insert(address.clone());
    }

    fn get_dynamic_addresses(&self) -> Vec<String> {
        self.dynamic_addresses.clone()
    }

    fn add_route(&mut self, route: &Route) {
        self.routes.insert(route.id(), route.clone());
    }

    fn set_ipv6_mtu(&mut self, mtu: Option<MTUV6>) {
        self.ipv6_mtu = mtu;
    }

    fn get_ipv6_mtu(&self) -> Option<MTUV6> {
        self.ipv6_mtu
    }

    fn get_system_state(&self) -> HashMap<String, serde_yml::Value> {
        self.system_state.clone()
    }

    fn set_system_state(&mut self, state: HashMap<String, serde_yml::Value>) {
        self.system_state = state;
    }

    fn set_dynamic_addresses(&mut self, addresses: &Vec<String>) {
        self.dynamic_addresses = addresses.clone();
    }
}
