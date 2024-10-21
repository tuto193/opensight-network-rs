use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
};
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use super::{device::Device, nameservers::Nameservers, route::Route};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Ethernet {
    #[serde(skip_serializing)]
    pub name: String,
    pub dhcp4: bool,
    pub dhcp6: bool,
    pub mtu: usize,
    pub accept_ra: bool,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub routes: HashMap<String, Route>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    pub addresses: HashSet<IpAddr>,
    pub nameservers: Nameservers,
}

impl Ethernet {
    pub fn new(name: String) -> Self {
        Self {
            name,
            dhcp4: false,
            dhcp6: false,
            mtu: 0,
            accept_ra: false,
            routes: HashMap::new(),
            addresses: HashSet::new(),
            nameservers: Nameservers::new(),
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

    fn get_mtu(&self) -> usize {
        self.mtu
    }

    fn set_mtu(&mut self, mtu: usize) {
        self.mtu = mtu;
    }

    fn get_addresses(&self) -> HashSet<IpAddr> {
        self.addresses.clone()
    }

    fn set_addresses(&mut self, addresses: HashSet<IpAddr>) {
        self.addresses = addresses;
    }

    fn add_address(&mut self, address: IpAddr) {
        self.addresses.insert(address);
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

    fn delete_nameservers_search(&mut self, search: &str) -> bool {
        self.nameservers.remove_search(search)
    }

    fn delete_nameservers_address(&mut self, address: &IpAddr) -> bool {
        self.nameservers.remove_address(address)
    }

    fn add_route(&mut self, to: IpAddr, via: Option<IpAddr>, from: Option<IpAddr>) {
        let to_add = Route::new(to, via, from);
        self.routes.insert(Uuid::new_v4().to_string(), to_add);
    }

    fn add_gateway_route(&mut self, via: Option<IpAddr>, from: Option<IpAddr>) {
        self.add_route("::/0".parse().unwrap(), via, from)
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
}
