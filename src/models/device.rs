use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, SocketAddr},
};

use serde::{Deserialize, Serialize};

use super::{input_models::InputDevice, nameservers::Nameservers, route::Route};
use crate::custom_types::BoundedU32;

pub type MTU = BoundedU32<1280, 64000>;
pub type MTUV6 = BoundedU32<1280, 64000>;
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum DynDevAttrType {
    Addresses,
    Routes,
    DnsAddresses,
}

pub trait Device {
    fn from_input_device(name: &str, input_device: &InputDevice) -> Self;
    fn update_from_device(&mut self, device: &impl Device);
    // DHCP stuff
    fn set_dhcp4(&mut self, set: bool);
    fn get_dhcp4(&self) -> bool;
    fn get_dhcp6(&self) -> bool;
    fn set_dhcp6(&mut self, set: bool);
    // ACCEPT_RA
    fn set_accept_ra(&mut self, set: Option<bool>);
    fn get_accept_ra(&self) -> Option<bool>;
    // MTU
    fn get_mtu(&self) -> Option<MTU>;
    fn set_mtu(&mut self, mtu: Option<MTU>);
    fn set_ipv6_mtu(&mut self, mtu: Option<MTUV6>);
    fn get_ipv6_mtu(&self) -> Option<MTUV6>;
    // ADDRESSES
    fn get_addresses(&self) -> HashSet<SocketAddr>;
    fn add_address(&mut self, address: &SocketAddr);
    fn delete_address(&mut self, address: &SocketAddr) -> bool;
    // fn add_address(&mut self, address: IpAddr);
    // NAMESERVERS
    fn get_nameservers(&self) -> Nameservers;
    fn add_nameservers(&mut self, nameservers: Nameservers);
    fn add_nameservers_search(&mut self, search: &str);
    fn add_nameservers_address(&mut self, address: &IpAddr);
    fn delete_nameservers_search(&mut self, search: &str);
    fn delete_nameservers_address(&mut self, address: &IpAddr);
    // fn add_nameservers_address(&mut self, address: IpAddr);
    // ROUTES
    fn get_routes(&self) -> HashMap<String, Route>;
    fn add_route(&mut self, route: &Route);
    fn delete_route(&mut self, route_id: &str) -> bool;
    fn delete_all_routes(&mut self);
    // fn add_built_route(&mut self, route: Route);
    // fn add_gateway_route(&mut self, via: Option<IpAddr>, from: Option<IpAddr>);
    // Attributes that need to be seen in API but not in config
    fn get_system_state_diff(&self) -> HashMap<String, serde_yml::Value>;
    fn set_system_state_diff(&mut self, state: HashMap<String, serde_yml::Value>);
    fn get_dynamic_attributes(&self) -> HashMap<DynDevAttrType, String>;
    fn set_dynamic_attributes_from_yaml(&mut self, yaml_output: HashMap<String, serde_yml::Value>);
}
