use std::convert::TryFrom;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use sigma_ser::vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};

use crate::utils::{default_vlq_reader, default_vlq_writer};

use super::errors::ModelError;

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
            let mut vlq_reader = default_vlq_reader(&data[port_start..]); // todo-minor move to utils?
            vlq_reader.get_u16().expect("internal error: port bytes slice len != 2") // todo-crucial 4 bytes??
        };

        Ok(Self(SocketAddr::new(ip_addr, port)))
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>, ModelError> {
        let PeerAddr(inner) = self;
        let mut vlq_writer = {
            let buf_size = if inner.is_ipv4() { Self::SIZE_IPv4_SOCKET } else { Self::SIZE_IPv6_SOCKET };
            default_vlq_writer(Vec::with_capacity(buf_size))
        };
        // todo-minor clean up copy-paste
        match inner {
            SocketAddr::V4(sock4) => {
                vlq_writer.write_all(sock4.ip().octets().as_ref()).map_err(ModelError::CannotSerializeData)?;
            }
            SocketAddr::V6(sock6) => {
                vlq_writer.write_all(sock6.ip().octets().as_ref()).map_err(ModelError::CannotSerializeData)?;
            }
        };
        vlq_writer.put_u16(inner.port()).map_err(ModelError::CannotSerializeData)?; // todo-crucial 4 bytes??

        Ok(vlq_writer.into_inner())
    }
}
