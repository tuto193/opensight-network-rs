use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
};

use serde::{Deserialize, Serialize};

use super::{nameservers::Nameservers, route::Route};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum IpType {
    V4,
    V6,
}

pub trait Device {
    // DHCP stuff
    fn set_dhcp4(&mut self, set: bool);
    fn get_dhcp4(&self) -> bool;
    fn get_dhcp6(&self) -> bool;
    fn set_dhcp6(&mut self, set: bool);
    // ACCEPT_RA
    fn set_accept_ra(&mut self, set: bool);
    fn get_accept_ra(&self) -> bool;
    // MTU
    fn get_mtu(&self) -> usize;
    fn set_mtu(&mut self, mtu: usize);
    // ADDRESSES
    fn get_addresses(&self) -> HashSet<IpAddr>;
    fn set_addresses(&mut self, addresses: HashSet<IpAddr>);
    fn add_address(&mut self, address: IpAddr);
    // NAMESERVERS
    fn get_nameservers(&self) -> Nameservers;
    fn add_nameservers(&mut self, nameservers: Nameservers);
    // ROUTES
    fn get_routes(&self) -> HashMap<IpType, Route>;
    fn set_routes(&mut self, ip_type: IpType, route: Route);
}
