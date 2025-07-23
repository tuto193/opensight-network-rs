use crate::misc::{
    deserialize_base_to, deserialize_ip_option, serialize_base_to, serialize_ip_option,
};
use std::net::{AddrParseError, IpAddr, SocketAddr};

use serde::{Deserialize, Serialize};

use super::input_models::InputRoute;

pub enum BaseTo {
    Default,
    AddressWithBits(SocketAddr),
}

pub enum DynTo {
    Default,
    AddressWithBits(SocketAddr),
    Address(IpAddr),
}

#[derive(Serialize, Deserialize)]
pub enum RouteType {
    Unicast,
    Anycast,
    Blackhole,
    Broadcast,
    Multicast,
    Local,
    Unreachable,
    Nat,
    Prohibit,
    Throw,
    Xresolve,
}

#[derive(Serialize, Deserialize)]
pub enum RouteScope {
    Global,
    Link,
    Host,
}

#[derive(Serialize, Deserialize)]
pub enum RouteProtocol {
    Dhcp,
    Kernel,
    Ra,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Represents a network route configuration.
///
/// A route defines how network traffic is directed from a source to a destination,
/// potentially through an intermediate gateway.
pub struct Route {
    /// The source IP address for this route.
    /// If `None`, the route applies to traffic from any source.
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_ip_option",
        deserialize_with = "deserialize_ip_option"
    )]
    pub from: Option<IpAddr>,
    /// The destination for this route.
    /// Can be either a default route or a specific address with network bits.
    #[serde(
        serialize_with = "serialize_base_to",
        deserialize_with = "deserialize_base_to"
    )]
    pub to: BaseTo,
    /// The gateway IP address through which traffic should be routed.
    /// If `None`, traffic is routed directly to the destination.
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_ip_option",
        deserialize_with = "deserialize_ip_option"
    )]
    pub via: Option<IpAddr>,
}

impl Route {
    /// Creates a new Route instance with the specified destination and optional gateway and source.
    ///
    /// # Arguments
    ///
    /// * `to` - The destination for this route as a BaseTo enum value
    /// * `via` - Optional gateway IP address through which traffic should be routed
    /// * `from` - Optional source IP address for this route
    ///
    /// # Returns
    ///
    /// A new `Route` instance with the specified configuration.
    pub fn new(to: BaseTo, via: Option<IpAddr>, from: Option<IpAddr>) -> Self {
        Route { from, to, via }
    }

    /// Converts an InputRoute into a Route, parsing string addresses and handling special cases.
    ///
    /// This method handles the conversion from string-based route definitions to
    /// strongly-typed structures. It treats "default" strings specially:
    /// - For `from` and `via` fields: converts "default" to IPv6 unspecified address ("::/0")
    /// - For `to` field: converts "default" to `BaseTo::Default`, otherwise parses as `SocketAddr` for `BaseTo::AddressWithBits`
    ///
    /// # Arguments
    ///
    /// * `input_route` - A reference to an InputRoute containing string-based route definitions
    ///
    /// # Returns
    ///
    /// * `Ok(Route)` - Successfully parsed route
    /// * `Err(AddrParseError)` - If any of the IP address strings cannot be parsed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let input = InputRoute {
    ///     from: Some("192.168.1.1".to_string()),
    ///     to: "10.0.0.0/24".to_string(),
    ///     via: Some("192.168.1.254".to_string()),
    /// };
    /// let route = Route::from_input_route(&input)?;
    /// ```
    pub fn from_input_route(input_route: &InputRoute) -> Result<Self, AddrParseError> {
        let result = Route {
            from: {
                if let Some(from) = input_route.from.clone() {
                    Some(from.parse::<IpAddr>()?)
                } else {
                    None
                }
            },
            to: {
                if input_route.to == "default" {
                    BaseTo::Default
                } else {
                    BaseTo::AddressWithBits(input_route.to.parse()?)
                }
            },
            via: {
                if let Some(via) = input_route.via.clone() {
                    Some(via.parse::<IpAddr>()?)
                } else {
                    None
                }
            },
        };
        Ok(result)
    }

    /// Displays the route information to stdout in a human-readable format.
    ///
    /// This method prints the route details including source (from), destination (to),
    /// and gateway (via) addresses. Optional fields are displayed as "None" when not present.
    /// Note: There's an inconsistency in the implementation where `from` is labeled as "Origin"
    /// when None.
    pub fn display(&self) {
        println!("Route:");
        if let Some(origin) = &self.from {
            println!("  From: {}", origin);
        } else {
            println!("  Origin: None");
        }
        println!("  To: {}", self.to);
        if let Some(via) = &self.via {
            println!("  Via: {}", via);
        } else {
            println!("  Via: None");
        }
    }

    /// Generates a string identifier for this route by combining address components.
    ///
    /// The identifier is constructed by combining the from, to, and via addresses
    /// separated by hyphens. The method uses placeholder strings for missing optional
    /// fields and handles the BaseTo enum appropriately.
    ///
    /// # Returns
    ///
    /// A string in the format "{from}-{to}-{via}" where:
    /// - Missing `from` becomes "from"
    /// - `to` is converted using the BaseTo's Display implementation, with "default" for unspecified addresses
    /// - Missing `via` becomes "via"
    ///
    /// # Examples
    ///
    /// * Route with specific addresses: "192.168.1.1-10.0.0.0:8080-192.168.1.254"
    /// * Default route: "from-default-via"
    /// * Partial route: "192.168.1.1-10.0.0.0:8080-via"
    ///
    /// # Note
    ///
    /// The current implementation calls `is_unspecified()` on `self.to`, but `to` is of type
    /// `BaseTo`, not `IpAddr`. This may cause compilation issues.
    pub fn id(&self) -> String {
        format!(
            "{}-{}-{}",
            match self.from {
                Some(from) => from.to_string(),
                None => "from".to_string(),
            },
            if self.to.is_unspecified() {
                "default".to_string()
            } else {
                self.to.to_string()
            },
            match self.via {
                Some(via) => via.to_string(),
                None => "via".to_string(),
            }
        )
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DynamicRoute {
    pub from: Option<IpAddr>,
    pub to: DynTo,
    pub via: Option<IpAddr>,
    pub scope: RouteScope,
    pub protocol: RouteProtocol,
    pub type_: RouteType,
}
