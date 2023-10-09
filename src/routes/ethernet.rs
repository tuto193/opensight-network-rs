use std::net::{AddrParseError, IpAddr};

use crate::{
    models::{device::Device, ethernet::Ethernet, route::Route},
    netplan::NetplanStore,
};
use actix_web::{
    delete, get, patch, post, put,
    web::{Data, Json, Path, ServiceConfig},
    HttpResponse, Responder,
};
use utoipa::{IntoParams, OpenApi, ToSchema};

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
pub struct EthernetsApi;

pub fn configure(store: Data<NetplanStore>) -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .app_data(store)
            .service(show_all_ethernets)
            .service(get_ethernet)
            .service(get_ethernet_ip_addresses)
            .service(create_ethernet)
            .service(add_ethernet_ip_address)
            .service(delete_ethernet_ip_address)
            .service(update_ethernet_dhcp6)
            .service(update_ethernet_mtu)
            .service(update_ethernet_accept_ra)
            .service(get_ethernet_nameservers)
            .service(delete_ethernet)
            .service(update_ethernet_dhcp4)
            .service(add_ethernet_nameservers_search)
            .service(delete_ethernet_nameservers_search)
            .service(add_ethernet_nameservers_address)
            .service(delete_ethernet_nameservers_address)
            .service(get_ethernet_routes);
    }
}

#[utoipa::path(
    operation_id = "show-all-ethernets",
    responses(
        (status = 200, description = "Retrieve all managed ethernets.")
    )
)]
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
pub async fn show_all_ethernets(store: Data<NetplanStore>) -> impl Responder {
    let network = match Netplan::load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let ethernets = network.ethernets;
    HttpResponse::Ok().json(ethernets)
}

#[utoipa::path(
    operation_id = "create-ethernet",
    responses(
        (status = 200, description = "Create a new Ethernet entry.")
    )
)]
#[post("/<ethernet_name>")]
pub async fn create_ethernet(ethernet_name: String) -> impl Responder {
    let result = Ethernet::new(ethernet_name.clone());
    let mut network = match Netplan::load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    network.add_ethernet(result.clone());
    match Netplan::save_config(&network) {
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        Ok(_) => match Netplan::apply() {
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            Ok(_) => HttpResponse::Ok().json(result),
        },
    }
}

#[utoipa::path(
    operation_id = "get-ethernetsethernet",
    responses(
        (status = 200, description = "Get an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[get("/{ethernet_name}")]
pub async fn get_ethernet(ethernet_name: String) -> impl Responder {
    let network = match Netplan::load_config() {
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

#[utoipa::path(
    operation_id = "delete-ethernet",
    responses(
        (status = 200, description = "Delete an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[post("/{ethernet_name}")]
pub async fn delete_ethernet(ethernet_name: String) -> impl Responder {
    let mut network = match Netplan::load_config() {
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

#[utoipa::path(
    operation_id = "add-ethernet-ip-address",
    responses(
        (status = 200, description = "Add an IP address to an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[post("/{ethernet_name}/addresses")]
pub async fn add_ethernet_ip_address(ethernet_name: String, ip_address: String) -> impl Responder {
    let to_add = match ip_address.parse::<IpAddr>() {
        Err(err) => return HttpResponse::BadRequest().body(err.to_string()),
        Ok(ip) => ip,
    };
    let mut network = match Netplan::load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    // let ethernet = ethernet_proxy.clone();
    if let Some(mut ethernet) = ethernet {
        ethernet.add_address(to_add);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match Netplan::save_config(&network) {
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            Ok(_) => match Netplan::apply() {
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                Ok(_) => HttpResponse::Ok().json(ethernet),
            },
        }
    } else {
        HttpResponse::NotFound().body(format!("Ethernet {ethernet_name} was not found."))
    }
}

#[utoipa::path(
    operation_id = "get-ethernet-ip-addresses",
    responses(
        (status = 200, description = "Get IP addresses from an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[get("/{ethernet_name}/addresses")]
pub async fn get_ethernet_ip_addresses(ethernet_name: String) -> impl Responder {
    let network = match Netplan::load_config() {
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

#[utoipa::path(
    operation_id = "delete-ethernet-ip-address",
    responses(
        (status = 200, description = "Delete an IP address from an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[delete("/{ethernet_name}/addresses/{ip_address}")]
pub async fn delete_ethernet_ip_address(
    ethernet_name: String,
    ip_address: String,
) -> impl Responder {
    let to_delete = match ip_address.parse::<IpAddr>() {
        Err(err) => return HttpResponse::BadRequest().body(err.to_string()),
        Ok(ip) => ip,
    };
    let mut network = match Netplan::load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.delete_address(&to_delete);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match Netplan::save_config(&network) {
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            Ok(_) => match Netplan::apply() {
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                Ok(_) => HttpResponse::Ok().json(ethernet),
            },
        }
    } else {
        HttpResponse::NotFound().body(format!("Ethernet {ethernet_name} was not found."))
    }
}

#[utoipa::path(
    operation_id = "get-ethernet-nameservers",
    responses(
        (status = 200, description = "Show Nameservers from an existing ethernet"),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[get("/{ethernet_name}/nameservers")]
pub async fn get_ethernet_nameservers(ethernet_name: String) -> impl Responder {
    let network = match Netplan::load_config() {
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

#[utoipa::path(
    operation_id = "add-ethernet-nameservers-search",
    responses(
        (status = 200, description = "Add a nameserver to an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[post("/{ethernet_name}/nameservers")]
pub async fn add_ethernet_nameservers_search(
    ethernet_name: String,
    search: String,
) -> impl Responder {
    let mut network = match Netplan::load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.add_nameservers_search(search);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match Netplan::save_config(&network) {
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            Ok(_) => match Netplan::apply() {
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                Ok(_) => HttpResponse::Ok().json(ethernet),
            },
        }
    } else {
        HttpResponse::NotFound().body(format!("Ethernet {ethernet_name} was not found."))
    }
}

#[utoipa::path(
    operation_id = "update-ethernet-dhcp4",
    responses(
        (status = 200, description = "Update dhcp4 setting on an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[patch("/{ethernet_name}/dhcp4")]
pub async fn update_ethernet_dhcp4(ethernet_name: String, dhcp4: Json<bool>) -> impl Responder {
    let dhcp4 = dhcp4.into_inner();
    let mut network = match Netplan::load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.set_dhcp4(dhcp4);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match Netplan::save_config(&network) {
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            Ok(_) => match Netplan::apply() {
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                Ok(_) => HttpResponse::Ok().json(ethernet),
            },
        }
    } else {
        HttpResponse::NotFound().body(format!("Ethernet {ethernet_name} was not found."))
    }
}

#[utoipa::path(
    operation_id = "update-ethernet-dhcp6",
    responses(
        (status = 200, description = "Update dhcp6 setting on an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[patch("/{ethernet_name}/dhcp6")]
pub async fn update_ethernet_dhcp6(ethernet_name: String, dhcp6: Json<bool>) -> impl Responder {
    let dhcp6 = dhcp6.into_inner();
    let mut network = match Netplan::load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.set_dhcp6(dhcp6);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match Netplan::save_config(&network) {
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            Ok(_) => match Netplan::apply() {
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                Ok(_) => HttpResponse::Ok().json(ethernet),
            },
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[utoipa::path(
    operation_id = "update-ethernet-accept-ra",
    responses(
        (status = 200, description = "Update accept_ra setting on an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[patch("/{ethernet_name}/accept_ra")]
pub async fn update_ethernet_accept_ra(
    ethernet_name: String,
    accept_ra: Json<bool>,
) -> impl Responder {
    let accept_ra = accept_ra.into_inner();
    let mut network = match Netplan::load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.set_accept_ra(accept_ra);
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
    operation_id = "update-ethernet-mtu",
    responses(
        (status = 200, description = "Update mtu setting on an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[patch("/{ethernet_name}/mtu")]
pub async fn update_ethernet_mtu(ethernet_name: String, mtu: Json<usize>) -> impl Responder {
    let mtu = mtu.into_inner();
    let mut network = match Netplan::load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.set_mtu(mtu);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match Netplan::save_config(&network) {
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            Ok(_) => match Netplan::apply() {
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                Ok(_) => HttpResponse::Ok().json(ethernet),
            },
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[utoipa::path(
    operation_id = "delete-ethernet-nameservers-search",
    responses(
        (status = 200, description = "Delete nameservers search domain on an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[delete("/{ethernet_name}/nameservers/search")]
pub async fn delete_ethernet_nameservers_search(
    ethernet_name: String,
    search: String,
) -> impl Responder {
    let mut network = match Netplan::load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.delete_nameservers_search(search);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match Netplan::save_config(&network) {
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            Ok(_) => match Netplan::apply() {
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                Ok(_) => HttpResponse::Ok().json(ethernet),
            },
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[utoipa::path(
    operation_id = "add-ethernet-nameservers-address",
    responses(
        (status = 200, description = "Add nameserver address on an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[post("/{ethernet_name}/nameservers/address")]
pub async fn add_ethernet_nameservers_address(
    ethernet_name: String,
    address: String,
) -> impl Responder {
    let address: IpAddr = match address.parse() {
        Err(_) => return HttpResponse::BadRequest().finish(),
        Ok(address) => address,
    };
    let mut network = match Netplan::load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.add_nameservers_address(address);
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
    operation_id = "delete-ethernet-nameservers-address",
    responses(
        (status = 200, description = "Delete nameserver address on an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[delete("/{ethernet_name}/nameservers/address")]
pub async fn delete_ethernet_nameservers_address(
    ethernet_name: String,
    address: String,
) -> impl Responder {
    let address: IpAddr = match address.parse() {
        Err(_) => return HttpResponse::BadRequest().finish(),
        Ok(address) => address,
    };
    let mut network = match Netplan::load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.delete_nameservers_address(&address);
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
    operation_id = "get-ethernet-routes",
    responses(
        (status = 200, description = "Get routes from an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[get("/{ethernet_name}/routes")]
pub async fn get_ethernet_routes(ethernet_name: String) -> impl Responder {
    let network = match Netplan::load_config() {
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

#[utoipa::path(
    operation_id = "add-ethernet-route",
    responses(
        (status = 200, description = "Add a route on an existing Ethernet entry."),
        (status = 404, description = "Ethernet entry not found.")
    )
)]
#[post("/{ethernet_name}/routes")]
pub async fn add_ethernet_route(
    ethernet_name: String,
    to: String,
    via: String,
    from: String,
) -> impl Responder {
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
    let mut network = match Netplan::load_config() {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(network) => network,
    };
    let mut ethernets = network.ethernets.clone();
    let ethernet = ethernets.remove(&ethernet_name);
    if let Some(mut ethernet) = ethernet {
        ethernet.add_route(to, via, from);
        ethernets.insert(ethernet_name.clone(), ethernet.clone());
        network.set_ethernets(ethernets);
        match Netplan::save_config(&network) {
            Ok(_) => HttpResponse::Ok().json(ethernet),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    } else {
        HttpResponse::NotFound().body(format!("Ethernet {ethernet_name} was not found."))
    }
}
