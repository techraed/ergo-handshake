use std::convert::TryFrom;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::ops::{Deref, DerefMut};
use std::string::FromUtf8Error;

use crate::errors::ModelCreationError;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ShortString(String);

#[derive(Debug, PartialEq, Eq, Default)]
pub(super) struct Version(pub(super) [u8; Version::SIZE]);

#[derive(Debug, PartialEq, Eq)]
pub(super) struct HSPeerAddr(SocketAddr);

impl ShortString {
    pub(super) const SIZE: usize = 255;

    pub(super) fn try_from(data: Vec<u8>) -> Result<Self, ModelCreationError> {
        if data.len() > Self::SIZE {
            return Err(ModelCreationError::InvalidShortStringLength(data.len()));
        }
        let s = String::from_utf8(data).map_err(ModelCreationError::InvalidUtf8Buffer)?; // err type
        Ok(Self(s))
    }
}

impl Version {
    const SIZE: usize = 3;
}

impl HSPeerAddr {
    #[allow(non_upper_case_globals)]
    pub(super) const SIZE_IPv6_SOCKET: usize = Self::SIZE_IPv6 + Self::SIZE_PORT;
    #[allow(non_upper_case_globals)]
    pub(super) const SIZE_IPv4_SOCKET: usize = Self::SIZE_IPv4 + Self::SIZE_PORT;
    #[allow(non_upper_case_globals)]
    const SIZE_IPv4: usize = 4;
    #[allow(non_upper_case_globals)]
    const SIZE_IPv6: usize = 16;
    const SIZE_PORT: usize = 2;

    pub(crate) fn try_from(data: Vec<u8>) -> Result<Self, ModelCreationError> {
        let ip_addr = {
            match data.len() {
                Self::SIZE_IPv4_SOCKET => {
                    let ip_octets = <[u8; Self::SIZE_IPv4]>::try_from(&data[..Self::SIZE_IPv4]).expect("internal error: slice len != 4");
                    IpAddr::V4(Ipv4Addr::from(ip_octets))
                }
                Self::SIZE_IPv6_SOCKET => {
                    let ip_octets = <[u8; Self::SIZE_IPv6]>::try_from(&data[..Self::SIZE_IPv6]).expect("internal error: slice len != 16");
                    IpAddr::V6(Ipv6Addr::from(ip_octets))
                }
                _ => return Err(ModelCreationError::InvalidPeerAddrData(data.len())),
            }
        };
        let port = {
            let port_start = data.len() - Self::SIZE_PORT;
            let port_bytes = <[u8; Self::SIZE_PORT]>::try_from(&data[port_start..]).expect("internal error: slice len != 2");
            u16::from_be_bytes(port_bytes) // todo be? le? ne?
        };

        Ok(Self(SocketAddr::new(ip_addr, port)))
    }
}
