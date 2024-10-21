use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
};

use serde::{Deserialize, Serialize};

use super::{nameservers::Nameservers, route::Route};

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
    fn delete_address(&mut self, address: &IpAddr) -> bool;
    // NAMESERVERS
    fn get_nameservers(&self) -> Nameservers;
    fn add_nameservers(&mut self, nameservers: Nameservers);
    fn add_nameservers_search(&mut self, search: String);
    fn add_nameservers_address(&mut self, address: IpAddr);
    fn delete_nameservers_search(&mut self, search: &str) -> bool;
    fn delete_nameservers_address(&mut self, address: &IpAddr) -> bool;
    // ROUTES
    fn get_routes(&self) -> HashMap<String, Route>;
    fn add_route(&mut self, to: IpAddr, via: Option<IpAddr>, from: Option<IpAddr>);
    fn add_gateway_route(&mut self, via: Option<IpAddr>, from: Option<IpAddr>);
    fn delete_route(&mut self, route_id: String) -> bool;
    fn delete_all_routes(&mut self);
}
