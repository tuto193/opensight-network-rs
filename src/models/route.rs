use crate::misc::{deserialize_ip, deserialize_ip_option, serialize_ip, serialize_ip_option};
use std::net::{AddrParseError, IpAddr};

use serde::{Deserialize, Serialize};

use super::input_models::InputRoute;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Route {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_ip_option",
        deserialize_with = "deserialize_ip_option"
    )]
    pub from: Option<IpAddr>,
    #[serde(serialize_with = "serialize_ip", deserialize_with = "deserialize_ip")]
    pub to: IpAddr,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_ip_option",
        deserialize_with = "deserialize_ip_option"
    )]
    pub via: Option<IpAddr>,
}

impl Route {
    pub fn new(to: IpAddr, via: Option<IpAddr>, from: Option<IpAddr>) -> Self {
        Route { from, to, via }
    }

    pub fn from_input_route(input_route: &InputRoute) -> Result<Self, AddrParseError> {
        let result = Route {
            from: {
                if let Some(from) = input_route.from.clone() {
                    let new_value = if from == "default" {
                        "::/0".parse::<IpAddr>()?
                    } else {
                        from.parse::<IpAddr>()?
                    };
                    Some(new_value)
                } else {
                    None
                }
            },
            to: {
                if input_route.to == "default" {
                    "::/0".parse()?
                } else {
                    input_route.to.parse()?
                }
            },
            via: {
                if let Some(via) = input_route.via.clone() {
                    let new_value = if via == "default" {
                        "::/0".parse::<IpAddr>()?
                    } else {
                        via.parse::<IpAddr>()?
                    };
                    Some(new_value)
                } else {
                    None
                }
            },
        };
        Ok(result)
    }

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
