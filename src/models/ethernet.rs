use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, SocketAddr},
};

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::device::DynDevAttrType;

use super::{device::Device, input_models::InputDevice, nameservers::Nameservers, route::Route};

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "kebab-case")]
pub struct Ethernet {
    #[serde(skip_serializing)]
    name: String,
    dhcp4: bool,
    dhcp6: bool,
    #[validate(range(min = 1280, max = 64000))]
    mtu: Option<u32>,
    #[validate(range(min = 1280, max = 64000))]
    ipv6_mtu: Option<u32>,
    accept_ra: Option<bool>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    routes: HashMap<String, Route>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    addresses: HashSet<SocketAddr>,
    nameservers: Nameservers,
    // Attributes that only appear in the config (not in model)
    dhcp4_overrides: Option<HashMap<String, bool>>,
    dhcp6_overrides: Option<HashMap<String, bool>>,
    // Attributes that don't belong in the config
    #[serde(skip_serializing)]
    dynamic_attributes: HashMap<DynDevAttrType, Vec<String>>,
    #[serde(skip_serializing)]
    system_state_differences: HashMap<String, serde_yml::Value>,
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
            dhcp4_overrides: None,
            dhcp6_overrides: None,
            dynamic_attributes: HashMap::new(),
            system_state_differences: HashMap::new(),
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

    fn update_from_device(&mut self, device: &impl Device) {
        self.set_dhcp4(device.get_dhcp4());
        self.set_dhcp6(device.get_dhcp6());
        self.set_accept_ra(device.get_accept_ra());
        self.set_mtu(device.get_mtu());
        self.set_ipv6_mtu(device.get_ipv6_mtu());
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

    fn get_mtu(&self) -> Option<u32> {
        self.mtu
    }

    fn set_mtu(&mut self, mtu: Option<u32>) {
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

    fn add_nameservers_search(&mut self, search: &str) {
        self.nameservers.add_search(search);
    }

    fn add_nameservers_address(&mut self, address: &IpAddr) {
        self.nameservers.add_address(address);
    }

    fn delete_nameservers_search(&mut self, search: &str) {
        self.nameservers.remove_search(search);
    }

    fn delete_nameservers_address(&mut self, address: &IpAddr) {
        self.nameservers.remove_address(address);
    }

    fn delete_route(&mut self, route_id: &str) {
        self.routes.remove(route_id).is_some();
    }

    fn delete_address(&mut self, address: &SocketAddr) {
        self.addresses.remove(address);
    }

    fn delete_all_routes(&mut self) {
        self.routes = HashMap::new();
    }

    fn add_address(&mut self, address: &SocketAddr) {
        self.addresses.insert(*address);
    }

    fn add_route(&mut self, route: &Route) {
        self.routes.insert(route.id(), *route);
    }

    fn set_ipv6_mtu(&mut self, mtu: Option<u32>) {
        self.ipv6_mtu = mtu;
    }

    fn get_ipv6_mtu(&self) -> Option<u32> {
        self.ipv6_mtu
    }

    fn get_system_state_diff(&self) -> HashMap<String, serde_yml::Value> {
        self.system_state_differences.clone()
    }

    fn set_system_state_diff(&mut self, state: HashMap<String, serde_yml::Value>) {
        self.system_state_differences = state;
    }

    fn get_dynamic_attributes(&self) -> HashMap<DynDevAttrType, Vec<String>> {
        todo!()
    }

    fn set_dynamic_attributes_from_yaml(
        &mut self,
        dynamic_attributes: HashMap<DynDevAttrType, Vec<String>>,
    ) {
        self.dynamic_attributes = dynamic_attributes;
    }
}
