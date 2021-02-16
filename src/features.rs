use std::convert::TryFrom;
use std::io::{Cursor, Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use sigma_ser::peekable_reader::PeekableReader;
use sigma_ser::vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};
use std::fmt::{Debug, Formatter};

const MODE_ID: u8 = 16;
const LOCAL_ADDR_ID: u8 = 2;

pub(super) type Features = Vec<Box<dyn SerializableFeature>>;

pub(super) trait SerializableFeature {
    fn write(&self, writer: &mut Cursor<Vec<u8>>) {
        // to strict writer type
        writer.put_u8(self.feature_id());
        let feature_bytes = self.to_bytes();
        writer.put_u16(feature_bytes.len() as u16); // need check
        writer.write(&feature_bytes);
    }

    fn to_bytes(&self) -> Vec<u8>; // 65535

    fn feature_id(&self) -> u8; // no const for box dyn
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Mode {
    pub(super) state_type: u8,
    pub(super) is_verifying: bool,
    pub(super) is_nipopow: bool,
    pub(super) blocks_kept: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct LocalAddress(pub(super) SocketAddr);

pub(super) fn parse_feature(id: u8, data: Vec<u8>) -> Result<Box<dyn SerializableFeature>, ()> {
    // parse error
    match id {
        MODE_ID => Mode::try_from(data).map(|f| Box::new(f) as Box<dyn SerializableFeature>),
        LOCAL_ADDR_ID => LocalAddress::try_from(data).map(|f| Box::new(f) as Box<dyn SerializableFeature>),
        _ => unreachable!(), // actually, reachable
    }
}

impl SerializableFeature for Mode {
    fn to_bytes(&self) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::new());
        writer.put_u8(self.state_type);
        writer.put_u8(self.is_verifying as u8);
        writer.put_u8(self.is_nipopow as u8);
        writer.put_i32(self.blocks_kept);

        writer.into_inner()
    }

    #[inline]
    fn feature_id(&self) -> u8 {
        MODE_ID
    }
}

impl SerializableFeature for LocalAddress {
    fn to_bytes(&self) -> Vec<u8> {
        let LocalAddress(ref socket_addr) = self;
        let mut writer = Cursor::new(Vec::new());
        match socket_addr.ip() {
            IpAddr::V4(v4) => writer.write(&v4.octets()),
            IpAddr::V6(v6) => writer.write(&v6.octets()), // spec wants 6 bytes!
        };
        writer.put_u32(socket_addr.port() as u32);

        writer.into_inner()
    }

    #[inline]
    fn feature_id(&self) -> u8 {
        LOCAL_ADDR_ID
    }
}

impl TryFrom<Vec<u8>> for LocalAddress {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let is_ip4 = value.len() == 6; // !!
        let is_ip6 = value.len() == 10;
        println!("LEN {}", value.len());

        let mut reader = PeekableReader::new(Cursor::new(value));
        let mut socket_addr = "0.0.0.0:8080".parse::<SocketAddr>().expect("todo");
        if is_ip4 {
            let mut ip4_bytes = [0; 4];
            reader.read_exact(&mut ip4_bytes).map_err(|_| ())?;
            println!("IP BYTES {:?}", ip4_bytes);
            socket_addr.set_ip(IpAddr::V4(Ipv4Addr::from(ip4_bytes)));
        } else if is_ip6 {
            let mut ip6_bytes = [0; 16]; // spec wants 6 bytes!
            reader.read_exact(&mut ip6_bytes).map_err(|_| ())?;
            socket_addr.set_ip(IpAddr::V6(Ipv6Addr::from(ip6_bytes)));
        }
        let port = reader.get_u32().map_err(|_| ())?;
        socket_addr.set_port(port as u16);
        Ok(LocalAddress(socket_addr))
    }
}

impl TryFrom<Vec<u8>> for Mode {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut reader = PeekableReader::new(Cursor::new(value));

        let state_type = reader.get_u8().map_err(|_| ())?;
        let is_verifying = 1 == reader.get_u8().map_err(|_| ())?;
        let is_nipopow = 1 == reader.get_u8().map_err(|_| ())?;
        let blocks_kept = reader.get_i32().map_err(|_| ())?;

        Ok(Mode {
            state_type,
            is_verifying,
            is_nipopow,
            blocks_kept,
        })
    }
}
//
// impl Debug for dyn SerializableFeature {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Feature id {}", self.feature_id())
//     }
// }
