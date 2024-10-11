use std::{collections::HashSet, net::IpAddr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Nameservers {
    pub search: HashSet<String>,
    pub addresses: HashSet<IpAddr>,
}

impl Nameservers {
    pub fn new() -> Self {
        Nameservers {
            search: HashSet::new(),
            addresses: HashSet::new(),
        }
    }

    pub fn add_search(&mut self, domain: String) {
        self.search.insert(domain);
    }

    pub fn add_address(&mut self, address: IpAddr) {
        self.addresses.insert(address);
    }

    pub fn remove_search(&mut self, domain: &str) -> bool {
        self.search.remove(domain)
    }

    pub fn remove_address(&mut self, address: &IpAddr) -> bool {
        self.addresses.remove(address)
    }

    pub fn contains_search(&self, domain: &str) -> bool {
        self.search.contains(domain)
    }

    pub fn contains_address(&self, address: &IpAddr) -> bool {
        self.addresses.contains(address)
    }
}
