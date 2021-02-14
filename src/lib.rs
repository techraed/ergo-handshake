use std::io::{Cursor, Read, Write};
use std::net::{IpAddr, SocketAddr};
use std::time::SystemTime;

use sigma_ser::peekable_reader::PeekableReader;
use sigma_ser::vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};

use errors::*;
use features::{Features, LocalAddress, Mode};
use utils::make_timestamp;

mod errors;
mod features;
mod utils;

struct Handshake {
    agent_name: String,
    version: Version,
    peer_name: String,
    is_pub_node: bool,
    pub_address: Option<SocketAddr>,
    features: Option<Features>,
}

struct Version(u8, u8, u8); // maybe u32?

impl Handshake {
    pub(crate) fn serialize(&self) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::new());

        writer.put_u64(make_timestamp()); // hard to test
        writer.put_u8(self.agent_name.len() as u8); // type check
        writer.write(self.agent_name.as_bytes());
        writer.write(&self.version.to_bytes());
        writer.put_u8(self.peer_name.len() as u8);
        writer.write(self.peer_name.as_bytes());
        writer.put_u8(self.is_pub_node as u8);
        if self.is_pub_node {
            let pub_addr = self.pub_address.expect("internal error: pub node without pub addr");
            match pub_addr.ip() {
                IpAddr::V4(v4) => {
                    writer.put_u8(8); // including port 4 bytes!
                    writer.write(&v4.octets())
                }
                IpAddr::V6(v6) => {
                    writer.put_u8(10); // including port 4 bytes!
                    writer.write(&v6.octets())
                }
            };
            writer.put_u32(pub_addr.port() as u32);
        }
        if let Some(ref features) = self.features {
            writer.put_u8(features.len() as u8); // type check
            for feature in features {
                feature.write(&mut writer)
            }
        }

        writer.into_inner()
    }

    pub(crate) fn parse(data: &[u8]) -> Result<(), HandshakeParseError> {
        // any length check?
        let mut reader = PeekableReader::new(Cursor::new(data));
        let _timestamp = reader.get_u64().map_err(|_| HandshakeParseError)?;
        let agent_name = {
            let name_len = reader.get_u8().map_err(|_| HandshakeParseError)?;
            let mut agent_name_bytes = vec![0; name_len as usize];
            reader.read_exact(&mut agent_name_bytes).map_err(|| HandshakeParseError)?;
            String::from_utf8(agent_name_bytes).expect("internal error: received non utf-8 bytes")
        };
        let version = {
            let major = reader.get_u8().map_err(|e| HandshakeParseError)?;
            let minor = reader.get_u8().map_err(|e| HandshakeParseError)?;
            let patch = reader.get_u8().map_err(|e| HandshakeParseError)?;
            Version(major, minor, patch)
        };
        let peer_name = {
            let name_len = reader.get_u8().map_err(|_| HandshakeParseError)?;
            let mut peer_name_bytes = vec![0; name_len as usize];
            reader.read_exact(&mut peer_name_bytes).map_err(|| HandshakeParseError)?;
            String::from_utf8(peer_name_bytes).expect("internal error: received non utf-8 bytes")
        };
        let is_pub_node = { 1 == reader.get_u8().map_err(|e| HandshakeParseError)? };
        let mut pub_address = None;
        if is_pub_node {
            let address_len = reader.get_u8().map_err(|e| HandshakeParseError)?;
            let mut ip_bytes = vec![0; address_len as usize - 4];
            reader.read_exact(&mut ip_bytes).map_err(|_| HandshakeParseError)?;
            // create address from bytes
        }
        // parse Features
        Ok(())
    }
}

impl Version {
    pub(crate) fn to_bytes(&self) -> [u8; 3] {
        let Version(major, minor, patch) = *self;
        [major, minor, patch]
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};

    use hex;
    use sigma_ser::{
        peekable_reader::PeekableReader,
        vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt},
    };

    use super::*;
    use crate::features::SerializableFeature;

    fn hex_to_bytes(s: &str) -> Vec<u8> {
        hex::decode(s).expect("internal error: invalid hex str")
    }

    fn create_hs_featured(mut hs: Handshake, f: Features) -> Handshake {
        hs.features = Some(f);
        hs
    }

    fn create_hs(agent_name: String, peer_name: String, version: Version, is_pub: bool, pub_addr: Option<SocketAddr>) -> Handshake {
        Handshake {
            agent_name,
            version,
            peer_name,
            is_pub_node: false,
            pub_address: None,
            features: None,
        }
    }

    fn create_mode(state_type: u8, is_verifying: bool, is_nipopow: bool, blocks_kept: i32) -> Mode {
        Mode {
            state_type,
            is_verifying,
            is_nipopow,
            blocks_kept,
        }
    }

    fn create_local_addr(s: &str) -> LocalAddress {
        let socket_addr = s.parse().expect("internal error: invalid socket addr string");
        LocalAddress(socket_addr)
    }

    #[test]
    fn test_serialize_hs_ergo_vector() {
        let hs_bytes = hex_to_bytes("bcd2919cee2e076572676f726566030306126572676f2d6d61696e6e65742d332e332e36000210040001000102067f000001ae46");
        let hs = {
            let hs_simple = create_hs("ergoref".to_owned(), "ergo-mainnet-3.3.6".to_owned(), Version(3, 3, 6), false, None);
            let mode = create_mode(0, true, false, -1);
            let local = create_local_addr("127.0.0.1:9006");
            let features: Vec<Box<dyn SerializableFeature>> = vec![Box::new(mode), Box::new(local)];
            create_hs_featured(hs_simple, features)
        };
        assert_eq!(&hs.serialize()[5..], &hs_bytes[5..]); // ts bytes
    }

    #[test]
    fn testing_vector() {
        let mut writer = Cursor::new(Vec::new());
        let hs = "bcd2919cee2e076572676f726566030306126572676f2d6d61696e6e65742d332e332e36000210040001000102067f000001ae46";
        let hs_bytes = hex::decode(hs).unwrap();
        println!("{:?}", hs_bytes);

        let secs = 1610134874428u64;
        writer.put_u64(secs);
        writer.put_u8("ergoref".chars().count() as u8);
        writer.write("ergoref".as_bytes());
        writer.put_u8(3);
        writer.put_u8(3);
        writer.put_u8(6);
        writer.put_u8("ergo-mainnet-3.3.6".chars().count() as u8);
        writer.write("ergo-mainnet-3.3.6".as_bytes());
        println!("{:?}", writer);
        writer.put_u8(0); // is pub node flag
                          // потом идут 2 (количество фич), 16 (id Mode фичи), 4 (его длина), 0 (тип стэйта - utxo), 1 (надо верифайить транзы), 0 (нипопов),
        writer.put_i32(-1); // блокс ту кип
        writer.put_u8(127);
        writer.put_u8(0);
        writer.put_u8(0);
        writer.put_u8(1);
        writer.put_u32(9006u16 as u32); // port as u32
        println!("{:?}", writer);

        let mut reader = PeekableReader::new(Cursor::new(vec![174, 70]));
        let a = reader.get_u32().unwrap();
        println!("V: {}", a);
    }
}
