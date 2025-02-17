use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
};

use super::{nameservers::Nameservers, route::Route};

pub trait Device {
    // DHCP stuff
    fn set_dhcp4(&mut self, set: bool);
    fn get_dhcp4(&self) -> bool;
    fn get_dhcp6(&self) -> bool;
    fn set_dhcp6(&mut self, set: bool);
    // ACCEPT_RA
    fn set_accept_ra(&mut self, set: Option<bool>);
    fn get_accept_ra(&self) -> Option<bool>;
    // MTU
    fn get_mtu(&self) -> Option<u32>;
    fn set_mtu(&mut self, mtu: Option<u32>);
    fn set_ipv6_mtu(&mut self, mtu: Option<u32>);
    // ADDRESSES
    fn get_addresses(&self) -> HashSet<IpAddr>;
    // fn add_address(&mut self, address: IpAddr);
    fn add_addresses(&mut self, address: Vec<IpAddr>);
    fn get_dynamic_addresses(&self) -> HashSet<IpAddr>;
    fn delete_address(&mut self, address: &IpAddr) -> bool;
    // NAMESERVERS
    fn get_nameservers(&self) -> Nameservers;
    fn add_nameservers(&mut self, nameservers: Nameservers);
    fn add_nameservers_search(&mut self, search: String);
    // fn add_nameservers_address(&mut self, address: IpAddr);
    fn add_nameservers_address(&mut self, address: Vec<IpAddr>);
    fn delete_nameservers_search(&mut self, search: String) -> bool;
    fn delete_nameservers_address(&mut self, address: &IpAddr) -> bool;
    // ROUTES
    fn get_routes(&self) -> HashMap<String, Route>;
    fn add_route(&mut self, route: Route);
    // fn add_built_route(&mut self, route: Route);
    // fn add_gateway_route(&mut self, via: Option<IpAddr>, from: Option<IpAddr>);
    fn delete_route(&mut self, route_id: String) -> bool;
    fn delete_all_routes(&mut self);
}
