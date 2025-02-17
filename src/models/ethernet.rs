use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
};

use serde::{Deserialize, Serialize};

use super::{device::Device, nameservers::Nameservers, route::Route};
use crate::custom_types::BoundedU32;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Ethernet {
    #[serde(skip_serializing)]
    pub name: String,
    pub dhcp4: bool,
    pub dhcp6: bool,
    pub mtu: Option<BoundedU32<68, 64000>>,
    pub ipv6_mtu: Option<BoundedU32<1280, 64000>>,
    pub accept_ra: Option<bool>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub routes: HashMap<String, Route>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    pub addresses: HashSet<IpAddr>,
    pub nameservers: Nameservers,
    #[serde(skip_serializing)]
    pub dynamic_addresses: Vec<String>,
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
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
}

impl Device for Ethernet {
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

    fn set_accept_ra(&mut self, set: bool) {
        self.accept_ra = set;
    }

    fn get_accept_ra(&self) -> bool {
        self.accept_ra
    }

    fn get_mtu(&self) -> u32 {
        self.mtu
    }

    fn set_mtu(&mut self, mtu: u32) {
        self.mtu = mtu;
    }

    fn get_addresses(&self) -> HashSet<IpAddr> {
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

    fn add_nameservers_search(&mut self, search: String) {
        self.nameservers.add_search(search);
    }

    fn add_nameservers_address(&mut self, address: IpAddr) {
        self.nameservers.add_address(address);
    }

    fn delete_nameservers_search(&mut self, search: String) -> bool {
        self.nameservers.remove_search(search)
    }

    fn delete_nameservers_address(&mut self, address: &IpAddr) -> bool {
        self.nameservers.remove_address(address)
    }

    fn delete_route(&mut self, route_id: String) -> bool {
        self.routes.remove(&route_id).is_some()
    }

    fn delete_address(&mut self, address: &IpAddr) -> bool {
        self.addresses.remove(address)
    }

    fn delete_all_routes(&mut self) {
        self.routes = HashMap::new();
    }

    fn add_addresses(&mut self, address: Vec<IpAddr>) {
        todo!()
    }

    fn get_dynamic_addresses(&self) -> HashSet<IpAddr> {
        todo!()
    }

    fn add_route(&mut self, route: Route) {
        todo!()
    }
}
