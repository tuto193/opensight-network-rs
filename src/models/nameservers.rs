use crate::misc::{serialize_HashSet_from_IpAddr_as_yaml_sequence, serialize_HashSet_from_String_as_yaml_sequence}
use std::{collections::HashSet, net::IpAddr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Nameservers {
    #[serde(serialize_with = "serialize_HashSet_from_String_as_yaml_sequence")]
    search: HashSet<String>,
    #[serde(serialize_with = "serialize_HashSet_from_IpAddr_as_yaml_sequence")]
    addresses: HashSet<IpAddr>,
}

impl Nameservers {
    pub fn new() -> Self {
        Nameservers {
            search: HashSet::new(),
            addresses: HashSet::new(),
        }
    }

    pub fn add_search(&mut self, search: String) {
        self.search.insert(search);
    }

    pub fn add_address(&mut self, address: IpAddr) {
        self.addresses.insert(address);
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
