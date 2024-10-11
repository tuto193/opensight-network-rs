use lazy_static::lazy_static;
use rocket::Route;
use rocket_okapi::{openapi, openapi_get_routes};

use crate::{models::ethernet::Ethernet, netplan::Netplan};

#[openapi]
#[get("/")]
pub fn show_all_ethernets() -> String {
    let ethernets = Netplan::load_config()
        .expect("Error: Netplan could not load config properly.")
        .ethernets;
    format!("{ethernets:#?}")
}

#[openapi]
#[patch("/<ethernet_name>")]
pub fn create_ethernet(ethernet_name: String) -> String {
    let result = Ethernet::new(ethernet_name.clone());
    let mut network = Netplan::load_config().unwrap();
    network.ethernets.insert(ethernet_name, result.clone());
    format!("{result:#?}")
}

lazy_static! {
    pub static ref ETHERNET_ROUTES: Vec<Route> =
        openapi_get_routes![show_all_ethernets, create_ethernet];
}
