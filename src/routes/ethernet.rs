use lazy_static::lazy_static;
use rocket::Route;

use crate::{models::ethernet::Ethernet, netplan::Netplan};

/// Retrieves and displays all Ethernet entries from the Netplan configuration.
///
/// # Returns
///
/// A formatted string representation of all Ethernet entries.
///
/// # Errors
///
/// This function will panic if the Netplan configuration cannot be loaded properly.
#[utoipa::path(context_path = "/ethernets")]
#[get("/")]
pub fn show_all_ethernets() -> String {
    let ethernets = Netplan::load_config()
        .expect("Error: Netplan could not load config properly.")
        .ethernets;
    format!("{ethernets:#?}")
}

/// Creates a new Ethernet entry in the Netplan configuration.
///
/// # Arguments
///
/// * `ethernet_name` - A string slice that holds the name of the Ethernet to be created.
///
/// # Returns
///
/// A formatted string representation of the newly created Ethernet entry.
///
/// # Errors
///
/// This function will panic if the Netplan configuration cannot be loaded properly.
#[utoipa::path(context_path = "/ethernets")]
#[patch("/<ethernet_name>")]
pub fn create_ethernet(ethernet_name: String) -> String {
    let result = Ethernet::new(ethernet_name.clone());
    let mut network = Netplan::load_config().unwrap();
    network.ethernets.insert(ethernet_name, result.clone());
    format!("{result:#?}")
}

lazy_static! {
    pub static ref ETHERNET_ROUTES: Vec<Route> = routes![show_all_ethernets, create_ethernet];
}
