use std::convert::TryFrom;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use crate::utils::default_vlq_reader;

use super::errors::ModelError;
use sigma_ser::vlq_encode::ReadSigmaVlqExt;

#[derive(Debug, PartialEq, Eq)]
pub struct PeerAddr(pub SocketAddr);

impl PeerAddr {
    #[allow(non_upper_case_globals)]
    pub(crate) const SIZE_IPv6_SOCKET: usize = Self::SIZE_IPv6 + Self::SIZE_PORT;
    #[allow(non_upper_case_globals)]
    pub(crate) const SIZE_IPv4_SOCKET: usize = Self::SIZE_IPv4 + Self::SIZE_PORT;
    #[allow(non_upper_case_globals)]
    const SIZE_IPv4: usize = 4;
    #[allow(non_upper_case_globals)]
    const SIZE_IPv6: usize = 16;
    pub(crate) const SIZE_PORT: usize = 2;

    pub fn try_from(data: Vec<u8>) -> Result<Self, ModelError> {
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
                _ => return Err(ModelError::InvalidPeerAddrLength(data.len())),
            }
        };
        let port = {
            let port_start = data.len() - Self::SIZE_PORT;
            let mut vlq_reader = default_vlq_reader(&data[port_start..]); // todo utils?
            vlq_reader.get_u16().expect("internal error: port bytes slice len != 2")
        };

        Ok(Self(SocketAddr::new(ip_addr, port)))
    }
}
