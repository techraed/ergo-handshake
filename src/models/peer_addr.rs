use std::convert::TryFrom;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use sigma_ser::vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};

use crate::encoding::vlq::{default_vlq_reader, default_vlq_writer, TryFromVlq, TryIntoVlq};

use super::errors::{ModelParseError, ModelSerializeError};

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
    // Port u16 value is vlq-encoded in 2 or 3 bytes
    const PORT_EXCESS_VLQ_SIZE: usize = 1;
}

impl TryFromVlq for PeerAddr {
    type Error = ModelParseError;

    fn try_from_vlq(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (ip_addr, port_bytes) = {
            match data.len() {
                size_ip4_socket if size_ip4_socket == Self::SIZE_IPv4_SOCKET ||
                    size_ip4_socket == Self::SIZE_IPv4_SOCKET + Self::PORT_EXCESS_VLQ_SIZE => {
                    let (ip_bytes, port_bytes) = data.split_at(Self::SIZE_IPv4);
                    let ip_octets = <[u8; Self::SIZE_IPv4]>::try_from(ip_bytes).expect("internal error: slice len != 4");
                    (IpAddr::V4(Ipv4Addr::from(ip_octets)), port_bytes)
                }
                size_ip4_socket if size_ip4_socket == Self::SIZE_IPv6_SOCKET ||
                    size_ip4_socket == Self::SIZE_IPv6_SOCKET + Self::PORT_EXCESS_VLQ_SIZE => {
                    let (ip_bytes, port_bytes) = data.split_at(Self::SIZE_IPv6);
                    let ip_octets = <[u8; Self::SIZE_IPv6]>::try_from(ip_bytes).expect("internal error: slice len != 16");
                    (IpAddr::V6(Ipv6Addr::from(ip_octets)), port_bytes)
                }
                _ => return Err(ModelParseError::InvalidPeerAddrLength(data.len())),
            }
        };
        let port = {
            let mut vlq_reader = default_vlq_reader(port_bytes);
            vlq_reader.get_u16().expect("internal error: port bytes slice len isn't equal to 2 or 3")
        };

        Ok(Self(SocketAddr::new(ip_addr, port)))
    }
}

impl TryIntoVlq for PeerAddr {
    type Error = ModelSerializeError;

    fn try_into_vlq(&self) -> Result<Vec<u8>, Self::Error> {
        let PeerAddr(inner) = self;
        let mut vlq_writer = {
            let buf_size = if inner.is_ipv4() { Self::SIZE_IPv4_SOCKET } else { Self::SIZE_IPv6_SOCKET };
            default_vlq_writer(Vec::with_capacity(buf_size))
        };
        // todo-minor clean up copy-paste
        match inner {
            SocketAddr::V4(sock4) => {
                vlq_writer.write_all(&sock4.ip().octets())?;
            }
            SocketAddr::V6(sock6) => {
                vlq_writer.write_all(&sock6.ip().octets())?;
            }
        };
        vlq_writer.put_u16(inner.port())?;

        Ok(vlq_writer.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng};

    use super::*;

    enum AddrType {
        Ip4,
        Ip6
    }

    fn generate_random_peer_addr_bytes(addr_type: AddrType) -> Vec<u8> {
        let mut vlq_writer = default_vlq_writer(Vec::new());
        match addr_type {
            AddrType::Ip4 => {
                let ip = gen_ip4_octets();
                vlq_writer.write_all(&ip).expect("internal error: io failed");
            }
            AddrType::Ip6 => {
                let ip = gen_ip6_octets();
                vlq_writer.write_all(&ip).expect("internal error: io failed");
            }
        };
        let port = thread_rng().gen::<u16>();
        vlq_writer.put_u16(port).expect("internal error: io failed");
        vlq_writer.into_inner()
    }

    fn generate_random_addr(addr_type: AddrType) -> PeerAddr {
        let ip = match addr_type {
            AddrType::Ip4 => {
                let ip4 = thread_rng().gen::<u32>();
                IpAddr::V4(Ipv4Addr::from(ip4))
            }
            AddrType::Ip6 => {
                let ip6 = thread_rng().gen::<u128>();
                IpAddr::V6(Ipv6Addr::from(ip6))
            }
        };
        let port = thread_rng().gen::<u16>();
        PeerAddr(SocketAddr::new(ip, port))
    }

    fn gen_ip4_octets() -> [u8; 4] {
        let mut ret = [0u8; 4];
        for index in 0..4 {
            ret[index] = thread_rng().gen::<u8>();
        }
        ret
    }

    fn gen_ip6_octets() -> [u8; 16] {
        let mut ret = [0u8; 16];
        for index in 0..16 {
            ret[index] = thread_rng().gen::<u8>();
        }
        ret
    }

    #[test]
    fn test_serialize_valid_ip4_peer_addr() {
        for _ in 0..10 {
            let addr = generate_random_addr(AddrType::Ip4);
            assert!(addr.try_into_vlq().is_ok());
        }
    }

    #[test]
    fn test_serialize_valid_ip6_peer_addr() {
        for _ in 0..10 {
            let addr = generate_random_addr(AddrType::Ip6);
            assert!(addr.try_into_vlq().is_ok());
        }
    }

    #[test]
    fn test_parse_valid_ip4_peer_addr() {
        for _ in 0..10 {
            let data = generate_random_peer_addr_bytes(AddrType::Ip4);
            let peer_addr = PeerAddr::try_from_vlq(data);
            assert!(peer_addr.is_ok());
            assert!(peer_addr.expect("internal error: can't vlq decode peer addr").0.is_ipv4())
        }
    }

    #[test]
    fn test_parse_valid_ip6_peer_addr() {
        for _ in 0..10 {
            let data = generate_random_peer_addr_bytes(AddrType::Ip6);
            let peer_addr = PeerAddr::try_from_vlq(data);
            assert!(peer_addr.is_ok());
            assert!(peer_addr.expect("internal error: can't vlq decode peer addr").0.is_ipv6())
        }
    }

    #[test]
    fn test_parse_invalid_addr_len() {
        for len in 0..100 {
            // skip proper lengths
            if len == PeerAddr::SIZE_IPv4_SOCKET ||
                len == PeerAddr::SIZE_IPv6_SOCKET ||
                len == PeerAddr::SIZE_IPv4_SOCKET + PeerAddr::PORT_EXCESS_VLQ_SIZE ||
                len == PeerAddr::SIZE_IPv6_SOCKET + PeerAddr::PORT_EXCESS_VLQ_SIZE
            {
                continue;
            }
            let bytes = vec![0; len];
            assert!(PeerAddr::try_from_vlq(bytes).is_err());
        }
    }
}
