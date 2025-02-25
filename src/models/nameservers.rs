use crate::misc::{
    serialize_hash_set_from_ip_addr_as_yaml_sequence,
    serialize_hash_set_from_string_as_yaml_sequence,
};
use std::{collections::HashSet, net::IpAddr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Nameservers {
    #[serde(
        serialize_with = "serialize_hash_set_from_string_as_yaml_sequence",
        skip_serializing_if = "HashSet::is_empty"
    )]
    pub search: HashSet<String>,
    #[serde(
        serialize_with = "serialize_hash_set_from_ip_addr_as_yaml_sequence",
        skip_serializing_if = "HashSet::is_empty"
    )]
    pub addresses: HashSet<IpAddr>,
}

impl Nameservers {
    pub fn new() -> Self {
        Self {
            search: HashSet::new(),
            addresses: HashSet::new(),
        }
    }

    pub fn add_search(&mut self, search: &str) {
        self.search.insert(search.to_string());
    }

    pub fn add_address(&mut self, address: &IpAddr) {
        self.addresses.insert(*address);
    }

    pub fn remove_search(&mut self, search: &str) -> bool {
        self.search.remove(search)
    }

    pub fn remove_address(&mut self, address: &IpAddr) -> bool {
        self.addresses.remove(address)
    }

    pub fn contains_search(&self, search: &str) -> bool {
        self.search.contains(search)
    }

    pub fn contains_address(&self, address: &IpAddr) -> bool {
        self.addresses.contains(address)
    }
}
