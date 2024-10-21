use std::net::IpAddr;

use crate::{
    models::{device::Device, ethernet::Ethernet},
    netplan::Netplan,
};
use actix_web::{delete, get, patch, post, put, HttpResponse, Responder};
use utoipa::{IntoParams, OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(paths(show_all_ethernets, create_ethernet))]
pub struct EthernetsApi;

/// Retrieves and displays all Ethernet entries from the Netplan configuration.
///
/// # Returns
///
/// A formatted string representation of all Ethernet entries.
///
/// # Errors
///
/// This function will panic if the Netplan configuration cannot be loaded properly.
#[utoipa::path(
    responses(
        (status = 200, description = "Retrieve all managed ethernets.")
    )
)]
#[get("")]
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
#[utoipa::path(
    responses(
        (status = 200, description = "Create a new Ethernet entry.")
    )
)]
#[post("/<ethernet_name>")]
pub async fn create_ethernet(ethernet_name: String) -> impl Responder {
    let result = Ethernet::new(ethernet_name.clone());
    let mut network = Netplan::load_config().unwrap();
    network.add_ethernet(result.clone());
    match Netplan::save_config(&network) {
        Ok(_) => HttpResponse::Ok().json(result),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "Get an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[get("/{ethernet_name}")]
pub async fn get_ethernet(ethernet_name: String) -> impl Responder {
    let network = Netplan::load_config().unwrap();
    let ethernet = network.get_ethernets().get(&ethernet_name);
    if let Some(ethernet) = ethernet {
        HttpResponse::Ok().json(ethernet)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "Delete an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[post("/{ethernet_name}")]
pub async fn delete_ethernet(ethernet_name: String) -> impl Responder {
    let mut network = Netplan::load_config().unwrap();
    let ethernet = network.ethernets.remove(&ethernet_name);
    if let Some(ethernet) = ethernet {
        HttpResponse::Ok().json(ethernet)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "Add an IP address to an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[post("/{ethernet_name}/addresses")]
pub async fn add_ip_address(ethernet_name: String, ip_address: String) -> impl Responder {
    let to_add: IpAddr = ip_address.parse().unwrap();
    let mut network = Netplan::load_config().unwrap();
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    // let ethernet = ethernet_proxy.clone();
    if let Some(mut ethernet) = ethernet {
        ethernet.add_address(to_add);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match Netplan::save_config(&network) {
            Ok(_) => HttpResponse::Ok().json(ethernet),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "Get IP addresses from an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[get("/{ethernet_name}/addresses")]
pub async fn get_ip_addresses(ethernet_name: String) -> impl Responder {
    let network = Netplan::load_config().unwrap();
    let ethernet = network.get_ethernets().get(&ethernet_name);
    if let Some(ethernet) = ethernet {
        HttpResponse::Ok().json(ethernet.get_addresses())
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "Delete an IP address from an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[delete("/{ethernet_name}/addresses/{ip_address}")]
pub async fn delete_ip_address(ethernet_name: String, ip_address: String) -> impl Responder {
    let to_delete: IpAddr = ip_address.parse().unwrap();
    let mut network = Netplan::load_config().unwrap();
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.delete_address(&to_delete);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match Netplan::save_config(&network) {
            Ok(_) => HttpResponse::Ok().json(ethernet),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "Show Nameservers from an existing ethernet"),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[get("/{ethernet_name}/nameservers")]
pub async fn get_ethernet_nameservers(ethernet_name: String) -> impl Responder {
    let network = Netplan::load_config().unwrap();
    let ethernet = network.get_ethernets().get(&ethernet_name);
    if let Some(ethernet) = ethernet {
        HttpResponse::Ok().json(ethernet.get_nameservers())
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "Add a nameserver to an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[post("/{ethernet_name}/nameservers")]
pub async fn add_nameservers_search(ethernet_name: String, search: String) -> impl Responder {
    let mut network = Netplan::load_config().unwrap();
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.add_nameservers_search(search);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match Netplan::save_config(&network) {
            Ok(_) => HttpResponse::Ok().json(ethernet),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[patch("/{ethernet_name}/dhcp4")]
pub async fn update_ethernet_dhcp4(ethernet_name: String, dhcp4: bool) -> impl Responder {
    let mut network = Netplan::load_config().unwrap();
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.set_dhcp4(dhcp4);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match Netplan::save_config(&network) {
            Ok(_) => HttpResponse::Ok().json(ethernet),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}
