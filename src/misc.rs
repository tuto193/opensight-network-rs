use std::{collections::HashSet, net::IpAddr};

use serde::Serialize;

pub fn serialize_HashSet_from_IpAddr_as_yaml_sequence<S>(
    addresses: &HashSet<IpAddr>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let addresses_vec: Vec<&IpAddr> = addresses.iter().collect();
    addresses_vec.serialize(serializer)
}

pub fn serialize_HashSet_from_String_as_yaml_sequence<S>(
    addresses: &HashSet<String>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let addresses_vec: Vec<&String> = addresses.iter().collect();
    addresses_vec.serialize(serializer)
}
