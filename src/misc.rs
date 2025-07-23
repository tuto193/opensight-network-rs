use std::{
    collections::HashSet,
    net::{self, AddrParseError, IpAddr},
    os::unix::net::SocketAddr,
};

use serde::{Deserializer, Serialize};

use crate::models::route::BaseTo;

struct BaseToVisitor;
struct IpAddrVisitor;

pub fn serialize_hash_set_from_ip_addr_as_yaml_sequence<S>(
    addresses: &HashSet<IpAddr>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let addresses_vec: Vec<&IpAddr> = addresses.iter().collect();
    addresses_vec.serialize(serializer)
}

pub fn serialize_hash_set_from_string_as_yaml_sequence<S>(
    addresses: &HashSet<String>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let addresses_vec: Vec<&String> = addresses.iter().collect();
    addresses_vec.serialize(serializer)
}

pub fn serialize_ip_option<S>(origin: &Option<IpAddr>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match origin {
        Some(ip) => serializer.serialize_str(&ip.to_string()),
        None => serializer.serialize_none(),
    }
}

pub fn serialize_base_to<S>(ip: &BaseTo, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if ip.is_unspecified() {
        serializer.serialize_str("default")
    } else {
        serializer.serialize_str(&ip.to_string())
    }
}

impl serde::de::Visitor<'_> for BaseToVisitor {
    type Value = BaseTo;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an IP address or the literal 'default'")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if value == "default" {
            Ok(BaseTo::Default)
        } else {
            let result: Result<SocketAddr, AddrParseError> = value.parse();
            match result {
                Ok(ip) => Ok(BaseTo::AddressWithBits(ip)),
                Err(_) => Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(value),
                    &self,
                )),
            }
        }
    }
}

impl serde::de::Visitor<'_> for IpAddrVisitor {
    type Value = Option<IpAddr>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an IP address")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let result: Result<IpAddr, AddrParseError> = value.parse();
        match result {
            Ok(ip) => Ok(Some(ip)),
            Err(_) => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(value),
                &self,
            )),
        }
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }
}
pub fn deserialize_ip_option<'de, D>(deserializer: D) -> Result<Option<IpAddr>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_option(IpAddrVisitor)
}

pub fn deserialize_base_to<'de, D>(deserializer: D) -> Result<BaseTo, D::Error>
where
    D: Deserializer<'de>,
{
    let result = deserializer.deserialize_str(BaseToVisitor)?.unwrap();
    Ok(result)
}
