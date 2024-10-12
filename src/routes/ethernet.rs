use crate::{models::ethernet::Ethernet, netplan::Netplan};
use actix_web::{delete, get, patch, post, put, HttpResponse, Responder};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(ethernet::show_all_ethernerts, ethernet::create_ethernet))]
struct EthernesApi;

/// Retrieves and displays all Ethernet entries from the Netplan configuration.
///
/// # Returns
///
/// A formatted string representation of all Ethernet entries.
///
/// # Errors
///
/// This function will panic if the Netplan configuration cannot be loaded properly.
#[utoipa::path(context_path = "/")]
#[get("/")]
pub async fn show_all_ethernets() -> impl Responder {
    let ethernets = Netplan::load_config()
        .expect("Error: Netplan could not load config properly.")
        .ethernets;
    HttpResponse::Ok().json(ethernets)
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
pub async fn create_ethernet(ethernet_name: String) -> String {
    let result = Ethernet::new(ethernet_name.clone());
    let mut network = Netplan::load_config().unwrap();
    network.ethernets.insert(ethernet_name, result.clone());
    format!("{result:#?}")
}
