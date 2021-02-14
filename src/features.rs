use std::io::{Cursor, Write};
use std::net::{IpAddr, SocketAddr};

use sigma_ser::vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};

pub(super) type Features = Vec<Box<dyn SerializableFeature>>;

pub(super) trait SerializableFeature {
    fn write(&self, writer: &mut Cursor<Vec<u8>>) { // to strict writer type
        writer.put_u8(self.feature_id());
        let feature_bytes = self.to_bytes();
        writer.put_u16(feature_bytes.len() as u16); // need check
        writer.write(&feature_bytes);
    }

    fn to_bytes(&self) -> Vec<u8>; // 65535

    fn feature_id(&self) -> u8; // no const for box dyn
}

pub(super) struct Mode {
    pub(super) state_type: u8,
    pub(super) is_verifying: bool,
    pub(super) is_nipopow: bool,
    pub(super) blocks_kept: i32,
}

pub(super) struct LocalAddress(pub(super) SocketAddr);

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
        16
    }
}

impl SerializableFeature for LocalAddress {
    fn to_bytes(&self) -> Vec<u8> {
        let LocalAddress(ref socket_addr) = self;
        let mut writer = Cursor::new(Vec::new());
        match socket_addr.ip() {
            IpAddr::V4(v4) => writer.write(&v4.octets()),
            IpAddr::V6(v6) => writer.write(&v6.octets()),
        };
        writer.put_u32(socket_addr.port() as u32);

        writer.into_inner()
    }

    #[inline]
    fn feature_id(&self) -> u8 {
        2
    }
}
