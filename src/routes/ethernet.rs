use std::net::IpAddr;

use crate::{models::device::Device, models::ethernet::Ethernet, netplan::NetplanStore};
use actix_web::{
    delete, get, patch, post,
    web::{Data, Json},
    HttpResponse, Responder,
};
use utoipa::{path as api_path, OpenApi};
use utoipa_actix_web::service_config::ServiceConfig;

#[derive(OpenApi)]
#[openapi(paths(
    show_all_ethernets,
    create_ethernet,
    get_ethernet,
    delete_ethernet,
    get_ethernet_ip_addresses,
    add_ethernet_ip_address,
    delete_ethernet_ip_address,
    update_ethernet_dhcp4,
    update_ethernet_dhcp6,
    update_ethernet_mtu,
    update_ethernet_accept_ra,
    get_ethernet_nameservers,
    add_ethernet_nameservers_search,
    delete_ethernet_nameservers_search,
    add_ethernet_nameservers_address,
    delete_ethernet_nameservers_address,
    get_ethernet_routes,
    add_ethernet_route,
))]
/// API documentation for Ethernet management.
///
/// This struct provides the OpenAPI documentation for various endpoints related to Ethernet management.
/// The documented endpoints include operations for showing all ethernets, creating a new ethernet,
/// retrieving an existing ethernet, deleting an ethernet, managing IP addresses, DHCP settings, MTU,
/// accept_ra, nameservers, and routes.
pub struct EthernetsApi;

/// Configures the service with the provided NetplanStore.
///
/// This function sets up the service configuration by adding the necessary
/// data and services to the provided `ServiceConfig`. It registers various
/// endpoints related to Ethernet management, such as showing all ethernets,
/// creating a new ethernet, getting an existing ethernet, deleting an ethernet,
/// and managing IP addresses, DHCP settings, MTU, accept_ra, nameservers, and routes.
///
/// # Arguments
///
/// * `store` - A `Data<NetplanStore>` instance that holds the Netplan configuration store.
///
/// # Returns
///
/// A closure that takes a mutable reference to `ServiceConfig` and configures it with the necessary services.
pub fn configure(store: Data<NetplanStore>) -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .app_data(store)
            .service(add_ethernet_ip_address)
            .service(add_ethernet_nameservers_address)
            .service(add_ethernet_nameservers_search)
            .service(create_ethernet)
            .service(delete_ethernet)
            .service(delete_ethernet_ip_address)
            .service(delete_ethernet_nameservers_address)
            .service(delete_ethernet_nameservers_search)
            .service(get_ethernet)
            .service(get_ethernet_ip_addresses)
            .service(get_ethernet_nameservers)
            .service(get_ethernet_routes)
            .service(show_all_ethernets)
            .service(update_ethernet_accept_ra)
            .service(update_ethernet_dhcp4)
            .service(update_ethernet_dhcp6)
            .service(update_ethernet_mtu);
    }
}

#[api_path(operation_id = "show-all-ethernets")]
#[get("")]
/// Retrieves all managed Ethernet entries.
///
/// This function loads the network configuration using Netplan, extracts the Ethernet entries,
/// and returns them as a JSON response. If there is an error loading the configuration, it returns
/// an internal server error with the error message.
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the Ethernet entries if successful.
/// - `HttpResponse::InternalServerError` with an error message if there is an issue loading the configuration.
pub async fn show_all_ethernets(netplan_store: Data<NetplanStore>) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let ethernets = network.ethernets;
    HttpResponse::Ok().json(ethernets)
}

#[api_path(operation_id = "create-ethernet")]
#[post("/<ethernet_name>")]
/// Creates a new Ethernet entry.
///
/// This function creates a new Ethernet entry with the specified name, adds it to the network configuration,
/// saves the updated configuration, and applies the changes. If there is an error during any of these steps,
/// an appropriate HTTP response is returned.
///
/// # Arguments
/// - `netplan_store`: A `Data<NetplanStore>` instance that holds the Netplan configuration store.
/// - `ethernet_name`: The name of the new Ethernet entry to be created.
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the created Ethernet entry if successful.
/// - `HttpResponse::InternalServerError` if there is an issue loading, saving, or applying the configuration.
pub async fn create_ethernet(
    netplan_store: Data<NetplanStore>,
    ethernet_name: String,
) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let result = Ethernet::new(ethernet_name.clone());
    let mut network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    network.add_ethernet(result.clone());
    match netplan.save_config(&network) {
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        Ok(_) => match netplan.apply() {
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            Ok(_) => HttpResponse::Ok().json(result),
        },
    }
}

#[api_path(operation_id = "get-ethernetsethernet")]
#[get("/{ethernet_name}")]
/// Retrieves a specific Ethernet entry by name.
///
/// This function loads the network configuration using Netplan, searches for the specified Ethernet entry,
/// and returns it as a JSON response. If the Ethernet entry is not found, it returns a 404 Not Found response.
/// If there is an error loading the configuration, it returns an internal server error with the error message.
///
/// # Arguments
/// - `netplan_store`: A `Data<NetplanStore>` instance that holds the Netplan configuration store.
/// - `ethernet_name`: The name of the Ethernet entry to be retrieved.
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the Ethernet entry if found.
/// - `HttpResponse::NotFound` if the specified Ethernet entry is not found.
/// - `HttpResponse::InternalServerError` with an error message if there is an issue loading the configuration.
pub async fn get_ethernet(
    netplan_store: Data<NetplanStore>,
    ethernet_name: String,
) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let ethernet = network.get_ethernets().get(&ethernet_name);
    if let Some(ethernet) = ethernet {
        HttpResponse::Ok().json(ethernet)
    } else {
        HttpResponse::NotFound().body(format!("Ethernet {ethernet_name} was not found."))
    }
}

#[api_path(operation_id = "delete-ethernet")]
#[post("/{ethernet_name}")]
/// Deletes a specific Ethernet entry by name.
///
/// This function loads the network configuration using Netplan, searches for the specified Ethernet entry,
/// and removes it from the configuration. If the Ethernet entry is found, it is deleted and returned as a JSON response.
/// If the Ethernet entry is not found, it returns a 404 Not Found response. If there is an error loading the configuration,
/// it returns an internal server error with the error message.
///
/// # Arguments
/// - `netplan_store`: A `Data<NetplanStore>` instance that holds the Netplan configuration store.
/// - `ethernet_name`: The name of the Ethernet entry to be deleted.
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the deleted Ethernet entry if found.
/// - `HttpResponse::NotFound` if the specified Ethernet entry is not found.
/// - `HttpResponse::InternalServerError` with an error message if there is an issue loading the configuration.
pub async fn delete_ethernet(
    netplan_store: Data<NetplanStore>,
    ethernet_name: String,
) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let mut network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let ethernet = network.ethernets.remove(&ethernet_name);
    if let Some(ethernet) = ethernet {
        HttpResponse::Ok().json(ethernet)
    } else {
        HttpResponse::NotFound().body(format!("Ethernet {ethernet_name} was not found."))
    }
}

#[api_path(operation_id = "add-ethernet-ip-address")]
#[post("/{ethernet_name}/addresses")]
/// Adds an IP address to a specific Ethernet entry.
///
/// This function parses the provided IP address, loads the network configuration,
/// and adds the IP address to the specified Ethernet entry. If the Ethernet entry
/// is found, the IP address is added, and the updated configuration is saved and applied.
/// If the Ethernet entry is not found, a 404 response is returned.
///
/// # Arguments
/// - `netplan_store`: A `Data<NetplanStore>` instance that holds the Netplan configuration store.
/// - `ethernet_name`: The name of the Ethernet entry to which the IP address will be added.
/// - `ip_address`: The IP address to be added to the Ethernet entry.
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the updated Ethernet entry if successful.
/// - `HttpResponse::BadRequest` if the provided IP address is invalid.
/// - `HttpResponse::InternalServerError` if there is an issue loading or saving the configuration.
/// - `HttpResponse::NotFound` if the specified Ethernet entry is not found.
pub async fn add_ethernet_ip_address(
    netplan_store: Data<NetplanStore>,
    ethernet_name: String,
    ip_address: String,
) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let to_add = match ip_address.parse::<IpAddr>() {
        Err(err) => return HttpResponse::BadRequest().body(err.to_string()),
        Ok(ip) => ip,
    };
    let mut network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.add_address(to_add);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match netplan.save_config(&network) {
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            Ok(_) => match netplan.apply() {
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                Ok(_) => HttpResponse::Ok().json(ethernet),
            },
        }
    } else {
        HttpResponse::NotFound().body(format!("Ethernet {ethernet_name} was not found."))
    }
}

#[api_path(operation_id = "get-ethernet-ip-addresses")]
#[get("/{ethernet_name}/addresses")]
/// Retrieves the IP addresses associated with a specific Ethernet entry.
///
/// This function loads the network configuration using Netplan, searches for the specified Ethernet entry,
/// and returns its IP addresses as a JSON response. If the Ethernet entry is not found, it returns a 404 Not Found response.
/// If there is an error loading the configuration, it returns an internal server error with the error message.
///
/// # Arguments
/// - `netplan_store`: A `Data<NetplanStore>` instance that holds the Netplan configuration store.
/// - `ethernet_name`: The name of the Ethernet entry whose IP addresses are to be retrieved.
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the IP addresses if the Ethernet entry is found.
/// - `HttpResponse::NotFound` if the specified Ethernet entry is not found.
/// - `HttpResponse::InternalServerError` with an error message if there is an issue loading the configuration.
pub async fn get_ethernet_ip_addresses(
    netplan_store: Data<NetplanStore>,
    ethernet_name: String,
) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(n) => n,
    };
    let ethernet = network.get_ethernets().get(&ethernet_name);
    if let Some(ethernet) = ethernet {
        HttpResponse::Ok().json(ethernet.get_addresses())
    } else {
        HttpResponse::NotFound().body(format!("Ethernet {ethernet_name} was not found."))
    }
}

#[api_path(operation_id = "delete-ethernet-ip-address")]
#[delete("/{ethernet_name}/addresses/{ip_address}")]
/// Deletes an IP address from a specific Ethernet entry.
///
/// This function parses the provided IP address, loads the network configuration,
/// and removes the IP address from the specified Ethernet entry. If the Ethernet entry
/// is found, the IP address is removed, and the updated configuration is saved and applied.
/// If the Ethernet entry is not found, a 404 response is returned.
///
/// # Arguments
/// - `netplan_store`: A `Data<NetplanStore>` instance that holds the Netplan configuration store.
/// - `ethernet_name`: The name of the Ethernet entry from which the IP address will be removed.
/// - `ip_address`: The IP address to be removed from the Ethernet entry.
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the updated Ethernet entry if successful.
/// - `HttpResponse::BadRequest` if the provided IP address is invalid.
/// - `HttpResponse::InternalServerError` if there is an issue loading or saving the configuration.
/// - `HttpResponse::NotFound` if the specified Ethernet entry is not found.
pub async fn delete_ethernet_ip_address(
    netplan_store: Data<NetplanStore>,
    ethernet_name: String,
    ip_address: String,
) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let to_delete = match ip_address.parse::<IpAddr>() {
        Err(err) => return HttpResponse::BadRequest().body(err.to_string()),
        Ok(ip) => ip,
    };
    let mut network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.delete_address(&to_delete);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match netplan.save_config(&network) {
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            Ok(_) => match netplan.apply() {
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                Ok(_) => HttpResponse::Ok().json(ethernet),
            },
        }
    } else {
        HttpResponse::NotFound().body(format!("Ethernet {ethernet_name} was not found."))
    }
}

#[api_path(operation_id = "get-ethernet-nameservers")]
#[get("/{ethernet_name}/nameservers")]
/// Retrieves the nameservers associated with a specific Ethernet entry.
///
/// This function loads the network configuration using Netplan, searches for the specified Ethernet entry,
/// and returns its nameservers as a JSON response. If the Ethernet entry is not found, it returns a 404 Not Found response.
/// If there is an error loading the configuration, it returns an internal server error with the error message.
///
/// # Arguments
/// - `netplan_store`: A `Data<NetplanStore>` instance that holds the Netplan configuration store.
/// - `ethernet_name`: The name of the Ethernet entry whose nameservers are to be retrieved.
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the nameservers if the Ethernet entry is found.
/// - `HttpResponse::NotFound` if the specified Ethernet entry is not found.
/// - `HttpResponse::InternalServerError` with an error message if there is an issue loading the configuration.
pub async fn get_ethernet_nameservers(
    netplan_store: Data<NetplanStore>,
    ethernet_name: String,
) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(n) => n,
    };
    let ethernet = network.get_ethernets().get(&ethernet_name);
    if let Some(ethernet) = ethernet {
        HttpResponse::Ok().json(ethernet.get_nameservers())
    } else {
        HttpResponse::NotFound().body(format!("Ethernet {ethernet_name} was not found."))
    }
}

#[api_path(operation_id = "add-ethernet-nameservers-search")]
#[post("/{ethernet_name}/nameservers")]
/// Adds a search domain to the nameservers of a specific Ethernet entry.
///
/// This function loads the network configuration using Netplan, searches for the specified Ethernet entry,
/// and adds the provided search domain to its nameservers. If the Ethernet entry is found, the search domain
/// is added, and the updated configuration is saved and applied. If the Ethernet entry is not found, a 404 response is returned.
///
/// # Arguments
/// - `netplan_store`: A `Data<NetplanStore>` instance that holds the Netplan configuration store.
/// - `ethernet_name`: The name of the Ethernet entry to which the search domain will be added.
/// - `search`: The search domain to be added to the Ethernet entry's nameservers.
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the updated Ethernet entry if successful.
/// - `HttpResponse::InternalServerError` if there is an issue loading or saving the configuration.
/// - `HttpResponse::NotFound` if the specified Ethernet entry is not found.
pub async fn add_ethernet_nameservers_search(
    netplan_store: Data<NetplanStore>,
    ethernet_name: String,
    search: String,
) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let mut network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.add_nameservers_search(search);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match netplan.save_config(&network) {
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            Ok(_) => match netplan.apply() {
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                Ok(_) => HttpResponse::Ok().json(ethernet),
            },
        }
    } else {
        HttpResponse::NotFound().body(format!("Ethernet {ethernet_name} was not found."))
    }
}

#[api_path(operation_id = "update-ethernet-dhcp4")]
#[patch("/{ethernet_name}/dhcp4")]
/// Updates the DHCPv4 setting for a specific Ethernet entry.
///
/// This function loads the network configuration using Netplan, searches for the specified Ethernet entry,
/// and updates its DHCPv4 setting. If the Ethernet entry is found, the DHCPv4 setting is updated, and the
/// updated configuration is saved and applied. If the Ethernet entry is not found, a 404 response is returned.
///
/// # Arguments
/// - `netplan_store`: A `Data<NetplanStore>` instance that holds the Netplan configuration store.
/// - `ethernet_name`: The name of the Ethernet entry whose DHCPv4 setting is to be updated.
/// - `dhcp4`: A JSON body containing the new DHCPv4 setting (true or false).
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the updated Ethernet entry if successful.
/// - `HttpResponse::InternalServerError` if there is an issue loading or saving the configuration.
/// - `HttpResponse::NotFound` if the specified Ethernet entry is not found.
pub async fn update_ethernet_dhcp4(
    netplan_store: Data<NetplanStore>,
    ethernet_name: String,
    dhcp4: Json<bool>,
) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let dhcp4 = dhcp4.into_inner();
    let mut network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.set_dhcp4(dhcp4);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match netplan.save_config(&network) {
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            Ok(_) => match netplan.apply() {
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                Ok(_) => HttpResponse::Ok().json(ethernet),
            },
        }
    } else {
        HttpResponse::NotFound().body(format!("Ethernet {ethernet_name} was not found."))
    }
}

#[api_path(operation_id = "update-ethernet-dhcp6")]
#[patch("/{ethernet_name}/dhcp6")]
/// Updates the DHCPv6 setting for a specific Ethernet entry.
///
/// This function loads the network configuration using Netplan, searches for the specified Ethernet entry,
/// and updates its DHCPv6 setting. If the Ethernet entry is found, the DHCPv6 setting is updated, and the
/// updated configuration is saved and applied. If the Ethernet entry is not found, a 404 response is returned.
///
/// # Arguments
/// - `netplan_store`: A `Data<NetplanStore>` instance that holds the Netplan configuration store.
/// - `ethernet_name`: The name of the Ethernet entry whose DHCPv6 setting is to be updated.
/// - `dhcp6`: A JSON body containing the new DHCPv6 setting (true or false).
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the updated Ethernet entry if successful.
/// - `HttpResponse::InternalServerError` if there is an issue loading or saving the configuration.
/// - `HttpResponse::NotFound` if the specified Ethernet entry is not found.
pub async fn update_ethernet_dhcp6(
    netplan_store: Data<NetplanStore>,
    ethernet_name: String,
    dhcp6: Json<bool>,
) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let dhcp6 = dhcp6.into_inner();
    let mut network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.set_dhcp6(dhcp6);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match netplan.save_config(&network) {
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            Ok(_) => match netplan.apply() {
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                Ok(_) => HttpResponse::Ok().json(ethernet),
            },
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[api_path(operation_id = "update-ethernet-accept-ra")]
#[patch("/{ethernet_name}/accept_ra")]
/// Updates the accept_ra setting for a specific Ethernet entry.
///
/// This function loads the network configuration using Netplan, searches for the specified Ethernet entry,
/// and updates its accept_ra setting. If the Ethernet entry is found, the accept_ra setting is updated, and the
/// updated configuration is saved and applied. If the Ethernet entry is not found, a 404 response is returned.
///
/// # Arguments
/// - `netplan_store`: A `Data<NetplanStore>` instance that holds the Netplan configuration store.
/// - `ethernet_name`: The name of the Ethernet entry whose accept_ra setting is to be updated.
/// - `accept_ra`: A JSON body containing the new accept_ra setting (true or false).
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the updated Ethernet entry if successful.
/// - `HttpResponse::InternalServerError` if there is an issue loading or saving the configuration.
/// - `HttpResponse::NotFound` if the specified Ethernet entry is not found.
pub async fn update_ethernet_accept_ra(
    netplan_store: Data<NetplanStore>,
    ethernet_name: String,
    accept_ra: Json<bool>,
) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let accept_ra = accept_ra.into_inner();
    let mut network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.set_accept_ra(accept_ra);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match netplan.save_config(&network) {
            Ok(_) => HttpResponse::Ok().json(ethernet),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[api_path(operation_id = "update-ethernet-mtu")]
#[patch("/{ethernet_name}/mtu")]
/// Updates the MTU setting for a specific Ethernet entry.
///
/// This function loads the network configuration using Netplan, searches for the specified Ethernet entry,
/// and updates its MTU setting. If the Ethernet entry is found, the MTU setting is updated, and the
/// updated configuration is saved and applied. If the Ethernet entry is not found, a 404 response is returned.
///
/// # Arguments
/// - `netplan_store`: A `Data<NetplanStore>` instance that holds the Netplan configuration store.
/// - `ethernet_name`: The name of the Ethernet entry whose MTU setting is to be updated.
/// - `mtu`: A JSON body containing the new MTU setting (usize).
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the updated Ethernet entry if successful.
/// - `HttpResponse::InternalServerError` if there is an issue loading or saving the configuration.
/// - `HttpResponse::NotFound` if the specified Ethernet entry is not found.
pub async fn update_ethernet_mtu(
    netplan_store: Data<NetplanStore>,
    ethernet_name: String,
    mtu: Json<usize>,
) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let mtu = mtu.into_inner();
    let mut network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.set_mtu(mtu);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match netplan.save_config(&network) {
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            Ok(_) => match netplan.apply() {
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                Ok(_) => HttpResponse::Ok().json(ethernet),
            },
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[api_path(operation_id = "delete-ethernet-nameservers-search")]
#[delete("/{ethernet_name}/nameservers/search")]
/// Deletes a search domain from the nameservers of a specific Ethernet entry.
///
/// This function loads the network configuration using Netplan, searches for the specified Ethernet entry,
/// and removes the provided search domain from its nameservers. If the Ethernet entry is found, the search domain
/// is removed, and the updated configuration is saved and applied. If the Ethernet entry is not found, a 404 response is returned.
///
/// # Arguments
/// - `netplan_store`: A `Data<NetplanStore>` instance that holds the Netplan configuration store.
/// - `ethernet_name`: The name of the Ethernet entry from which the search domain will be removed.
/// - `search`: The search domain to be removed from the Ethernet entry's nameservers.
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the updated Ethernet entry if successful.
/// - `HttpResponse::InternalServerError` if there is an issue loading or saving the configuration.
/// - `HttpResponse::NotFound` if the specified Ethernet entry is not found.
pub async fn delete_ethernet_nameservers_search(
    netplan_store: Data<NetplanStore>,
    ethernet_name: String,
    search: String,
) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let mut network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.delete_nameservers_search(search);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match netplan.save_config(&network) {
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            Ok(_) => match netplan.apply() {
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                Ok(_) => HttpResponse::Ok().json(ethernet),
            },
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[api_path(operation_id = "add-ethernet-nameservers-address")]
#[post("/{ethernet_name}/nameservers/address")]
/// Adds a nameserver address to a specific Ethernet entry.
///
/// This function parses the provided nameserver address, loads the network configuration,
/// and adds the address to the specified Ethernet entry. If the Ethernet entry is found,
/// the nameserver address is added, and the updated configuration is saved and applied.
/// If the Ethernet entry is not found, a 404 response is returned.
///
/// # Arguments
/// - `netplan_store`: A `Data<NetplanStore>` instance that holds the Netplan configuration store.
/// - `ethernet_name`: The name of the Ethernet entry to which the nameserver address will be added.
/// - `address`: The nameserver address to be added to the Ethernet entry.
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the updated Ethernet entry if successful.
/// - `HttpResponse::BadRequest` if the provided nameserver address is invalid.
/// - `HttpResponse::InternalServerError` if there is an issue loading or saving the configuration.
/// - `HttpResponse::NotFound` if the specified Ethernet entry is not found.
pub async fn add_ethernet_nameservers_address(
    netplan_store: Data<NetplanStore>,
    ethernet_name: String,
    address: String,
) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let address: IpAddr = match address.parse() {
        Err(_) => return HttpResponse::BadRequest().finish(),
        Ok(address) => address,
    };
    let mut network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.add_nameservers_address(address);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match netplan.save_config(&network) {
            Ok(_) => HttpResponse::Ok().json(ethernet),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[api_path(operation_id = "delete-ethernet-nameservers-address")]
#[delete("/{ethernet_name}/nameservers/address")]
/// Deletes a nameserver address from a specific Ethernet entry.
///
/// This function parses the provided nameserver address, loads the network configuration,
/// and removes the address from the specified Ethernet entry. If the Ethernet entry is found,
/// the nameserver address is removed, and the updated configuration is saved and applied.
/// If the Ethernet entry is not found, a 404 response is returned.
///
/// # Arguments
/// - `netplan_store`: A `Data<NetplanStore>` instance that holds the Netplan configuration store.
/// - `ethernet_name`: The name of the Ethernet entry from which the nameserver address will be removed.
/// - `address`: The nameserver address to be removed from the Ethernet entry.
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the updated Ethernet entry if successful.
/// - `HttpResponse::BadRequest` if the provided nameserver address is invalid.
/// - `HttpResponse::InternalServerError` if there is an issue loading or saving the configuration.
/// - `HttpResponse::NotFound` if the specified Ethernet entry is not found.
pub async fn delete_ethernet_nameservers_address(
    netplan_store: Data<NetplanStore>,
    ethernet_name: String,
    address: String,
) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let address: IpAddr = match address.parse() {
        Err(_) => return HttpResponse::BadRequest().finish(),
        Ok(address) => address,
    };
    let mut network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.delete_nameservers_address(&address);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match netplan.save_config(&network) {
            Ok(_) => HttpResponse::Ok().json(ethernet),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[api_path(operation_id = "get-ethernet-routes")]
#[get("/{ethernet_name}/routes")]
/// Retrieves the routes associated with a specific Ethernet entry.
///
/// This function loads the network configuration using Netplan, searches for the specified Ethernet entry,
/// and returns its routes as a JSON response. If the Ethernet entry is not found, it returns a 404 Not Found response.
/// If there is an error loading the configuration, it returns an internal server error with the error message.
///
/// # Arguments
/// - `netplan_store`: A `Data<NetplanStore>` instance that holds the Netplan configuration store.
/// - `ethernet_name`: The name of the Ethernet entry whose routes are to be retrieved.
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the routes if the Ethernet entry is found.
/// - `HttpResponse::NotFound` if the specified Ethernet entry is not found.
/// - `HttpResponse::InternalServerError` with an error message if there is an issue loading the configuration.
pub async fn get_ethernet_routes(
    netplan_store: Data<NetplanStore>,
    ethernet_name: String,
) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(n) => n,
    };
    let ethernet = network.get_ethernets().get(&ethernet_name);
    if let Some(ethernet) = ethernet {
        HttpResponse::Ok().json(ethernet.get_routes())
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[api_path(operation_id = "add-ethernet-route")]
#[post("/{ethernet_name}/routes")]
/// Adds a route to an existing Ethernet entry.
///
/// This function parses the provided `to`, `via`, and `from` IP addresses, loads the network configuration,
/// and adds the route to the specified Ethernet entry. If the Ethernet entry is found, the route is added,
/// and the updated configuration is saved and applied. If the Ethernet entry is not found, a 404 response is returned.
///
/// # Arguments
/// - `netplan_store`: A `Data<NetplanStore>` instance that holds the Netplan configuration store.
/// - `ethernet_name`: The name of the Ethernet entry to which the route will be added.
/// - `to`: The destination IP address for the route.
/// - `via`: The gateway IP address for the route (optional).
/// - `from`: The source IP address for the route (optional).
///
/// # Returns
/// - `HttpResponse::Ok` with a JSON body containing the updated Ethernet entry if successful.
/// - `HttpResponse::BadRequest` if the provided IP addresses are invalid.
/// - `HttpResponse::InternalServerError` if there is an issue loading or saving the configuration.
/// - `HttpResponse::NotFound` if the specified Ethernet entry is not found.
pub async fn add_ethernet_route(
    netplan_store: Data<NetplanStore>,
    ethernet_name: String,
    to: String,
    via: String,
    from: String,
) -> impl Responder {
    let netplan = netplan_store.netplan.lock().unwrap();
    let to = match to.parse::<IpAddr>() {
        Ok(to) => to,
        Err(err) => {
            return HttpResponse::BadRequest().body(err.to_string());
        }
    };
    let via: Option<IpAddr> = match via.parse() {
        Ok(ip) => Some(ip),
        Err(_) => None,
    };
    let from: Option<IpAddr> = match from.parse() {
        Ok(ip) => Some(ip),
        Err(_) => None,
    };
    let mut network = match netplan.load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.add_route(to, via, from);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match netplan.save_config(&network) {
            Ok(_) => HttpResponse::Ok().json(ethernet),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    } else {
        HttpResponse::NotFound().body(format!("Ethernet {ethernet_name} was not found."))
    }
}
