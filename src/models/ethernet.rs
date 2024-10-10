use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
};

use serde::{Deserialize, Serialize};

use super::{
    device::{Device, IpType, Mtu},
    nameservers::Nameservers,
    route::Route,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Ethernet {
    dhcp4: bool,
    dhcp6: bool,
    mtu: Mtu,
    accept_ra: bool,
    routes: HashMap<IpType, Route>,
    addresses: HashSet<IpAddr>,
    nameservers: Nameservers,
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

    fn get_mtu(&self) -> Mtu {
        self.mtu
    }

    fn set_mtu(&mut self, mtu: Mtu) {
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

    fn get_routes(&self) -> HashMap<IpType, Route> {
        self.routes.clone()
    }

    fn set_routes(&mut self, ip_type: IpType, route: Route) {
        self.routes.insert(ip_type, route);
    }
}
