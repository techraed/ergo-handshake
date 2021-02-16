use sigma_ser::vlq_encode::{ReadSigmaVlqExt, VlqEncodingError};

use crate::models::{PeerAddr, PeerFeature, ShortString, Version};
use crate::utils::{default_vlq_reader, HSReader};

#[derive(Debug, PartialEq, Eq)]
pub struct Handshake {
    pub agent_name: ShortString,
    pub version: Version,
    pub peer_name: ShortString,
    pub pub_address: Option<PeerAddr>,
    pub features: Option<Vec<PeerFeature>>,
}

impl Handshake {
    pub fn parse(data: &[u8]) -> Result<Self, VlqEncodingError> {
        let mut reader = HSReader::new(default_vlq_reader(data));

        let _timestamp = reader.get_u64()?;
        let agent_name = reader.read_short_string()?;
        let version = reader.read_version()?;
        let peer_name = reader.read_short_string()?;
        let pub_address = {
            let is_pub_node = reader.get_u8()? == 1;
            if is_pub_node {
                Some(reader.read_peer_addr()?)
            } else {
                None
            }
        };
        let features = reader
            // moving out unrecognized features
            // todo move to reader?
            .read_features()?
            .map(|mut f| {
                // todo if let PeerFeature::Unrecognized = pf { false } else { true }
                f.retain(|pf| pf != &PeerFeature::Unrecognized);
                if f.len() > 0 {
                    Some(f)
                } else {
                    None
                }
            })
            .flatten();

        Ok(Handshake {
            agent_name,
            version,
            peer_name,
            pub_address,
            features,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        //     let mut writer = Cursor::new(Vec::new());
        //
        //     writer.put_u64(make_timestamp()); // hard to test
        //     writer.put_u8(self.agent_name.len() as u8); // type check
        //     writer.write(self.agent_name.as_bytes());
        //     writer.write(&self.version.as_bytes());
        //     writer.put_u8(self.peer_name.len() as u8);
        //     writer.write(self.peer_name.as_bytes());
        //     writer.put_u8(self.is_pub_node as u8);
        //     if self.is_pub_node {
        //         let pub_addr = self.pub_address.expect("internal error: pub node without pub addr");
        //         match pub_addr.ip() {
        //             IpAddr::V4(v4) => {
        //                 writer.put_u8(8); // including port 4 bytes!
        //                 writer.write(&v4.octets())
        //             }
        //             IpAddr::V6(v6) => {
        //                 writer.put_u8(10); // including port 4 bytes!
        //                 writer.write(&v6.octets())
        //             }
        //         };
        //         writer.put_u32(pub_addr.port() as u32);
        //     }
        //     if let Some(ref features) = self.features {
        //         writer.put_u8(features.len() as u8); // type check
        //         for feature in features {
        //             feature.write(&mut writer)
        //         }
        //     }
        //
        //     writer.into_inner()
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use hex;

    use super::*;

    fn hex_to_bytes(s: &str) -> Vec<u8> {
        hex::decode(s).expect("internal error: invalid hex str")
    }

    // fn create_hs_featured(mut hs: Handshake, f: Features) -> Handshake {
    //     hs.features = Some(f);
    //     hs
    // }
    //
    // fn create_hs(agent_name: String, peer_name: String, version: Version, is_pub: bool, pub_addr: Option<SocketAddr>) -> Handshake {
    //     Handshake {
    //         agent_name,
    //         version,
    //         peer_name,
    //         is_pub_node: false,
    //         pub_address: None,
    //         features: None,
    //     }
    // }
    //
    // fn create_mode(state_type: u8, is_verifying: bool, is_nipopow: bool, blocks_kept: i32) -> Mode {
    //     Mode {
    //         state_type,
    //         is_verifying,
    //         is_nipopow,
    //         blocks_kept,
    //     }
    // }
    //
    // fn create_local_addr(s: &str) -> LocalAddress {
    //     let socket_addr = s.parse().expect("internal error: invalid socket addr string");
    //     LocalAddress(socket_addr)
    // }

    #[test]
    fn test_hs_ergo_vector() {
        let hs_bytes = hex_to_bytes("bcd2919cee2e076572676f726566030306126572676f2d6d61696e6e65742d332e332e36000210040001000102067f000001ae46");
        let a = Handshake::parse(&hs_bytes).ok().expect("ergo reference test vector failed");
        println!("{:?}", a);
    }

    // #[test]
    // fn testing_vector() {
    //     let mut writer = Cursor::new(Vec::new());
    //     let hs = "bcd2919cee2e076572676f726566030306126572676f2d6d61696e6e65742d332e332e36000210040001000102067f000001ae46";
    //     let hs_bytes = hex::decode(hs).unwrap();
    //     println!("{:?}", hs_bytes);
    //
    //     let secs = 1610134874428u64;
    //     writer.put_u64(secs);
    //     writer.put_u8("ergoref".chars().count() as u8);
    //     writer.write("ergoref".as_bytes());
    //     writer.put_u8(3);
    //     writer.put_u8(3);
    //     writer.put_u8(6);
    //     writer.put_u8("ergo-mainnet-3.3.6".chars().count() as u8);
    //     writer.write("ergo-mainnet-3.3.6".as_bytes());
    //     println!("{:?}", writer);
    //     writer.put_u8(0); // is pub node flag
    //                       // потом идут 2 (количество фич), 16 (id Mode фичи), 4 (его длина), 0 (тип стэйта - utxo), 1 (надо верифайить транзы), 0 (нипопов),
    //     writer.put_i32(-1); // блокс ту кип
    //     writer.put_u8(127);
    //     writer.put_u8(0);
    //     writer.put_u8(0);
    //     writer.put_u8(1);
    //     writer.put_u32(9006u16 as u32); // port as u32
    //     println!("{:?}", writer);
    //
    //     let mut reader = PeekableReader::new(Cursor::new(vec![174, 70]));
    //     let a = reader.get_u32().unwrap();
    //     println!("V: {}", a);
    // }
}
