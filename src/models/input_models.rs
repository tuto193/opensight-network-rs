use super::device::{MTU, MTUV6};

pub struct InputDevice {
    accept_ra: Option<bool>,
    dhcp4: Option<bool>,
    dhcp6: Option<bool>,
    mtu: Option<MTU>,
    ipv6_mtu: Option<MTUV6>,
}
