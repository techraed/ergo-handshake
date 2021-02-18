use sigma_ser::vlq_encode::{ReadSigmaVlqExt, VlqEncodingError, WriteSigmaVlqExt};

use crate::models::{Features, PeerAddr, PeerFeature, ShortString, Version};
use crate::utils::{default_vlq_reader, default_vlq_writer, make_timestamp, HSSpecReader, HSSpecWriter};

#[derive(Debug, PartialEq, Eq)]
pub struct Handshake {
    pub agent_name: ShortString,
    pub version: Version,
    pub peer_name: ShortString,
    pub pub_address: Option<PeerAddr>,
    pub features: Option<Features>,
}

// todo-minor HandshakeParser/SerializerError
impl Handshake {
    pub fn parse(data: &[u8]) -> Result<Self, VlqEncodingError> {
        let mut hs_reader = HSSpecReader::new(default_vlq_reader(data));

        let _timestamp = hs_reader.get_u64()?;
        let agent_name = hs_reader.read_short_string()?;
        let version = hs_reader.read_version()?;
        let peer_name = hs_reader.read_short_string()?;
        let pub_address = {
            let is_pub_node = hs_reader.get_u8()? == 1;
            if is_pub_node {
                Some(hs_reader.read_peer_addr()?)
            } else {
                None
            }
        };
        let features = hs_reader
            // moving out unrecognized features
            // todo-minor move to utils::reader?
            .read_features()?
            .map(|mut f| {
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

    pub fn serialize(&self) -> Result<Vec<u8>, VlqEncodingError> {
        let mut hs_writer = HSSpecWriter::new(default_vlq_writer(Vec::new()));

        hs_writer.put_u64(make_timestamp())?;
        hs_writer.write_short_string(&self.agent_name)?;
        hs_writer.write_version(&self.version)?;
        hs_writer.write_short_string(&self.peer_name)?;
        if let Some(peer_addr) = self.pub_address.as_ref() {
            hs_writer.put_u8(1)?;
            hs_writer.write_peer_addr(peer_addr)?;
        } else {
            hs_writer.put_u8(0)?;
        }
        if let Some(features) = self.features.as_ref() {
            hs_writer.write_features(features)?;
        }

        Ok(hs_writer.into_inner().into_inner())
    }
}

#[cfg(test)]
mod tests {
    use hex;

    use super::*;

    fn hex_to_bytes(s: &str) -> Vec<u8> {
        hex::decode(s).expect("internal error: invalid hex str")
    }

    #[test]
    fn vectors() {
        let bytes = [
            hex_to_bytes("c3bcaca3fb2e076572676f726566040005126572676f2d6d61696e6e65742d342e302e300002100400010001030e01000204e5c6abfafabc87cbc801"), // my
            hex_to_bytes("93bdaca3fb2e076572676f726566030306146d61696e6e65742d736565642d6e6f64652d73660108a5e31aafc64602100400010001030d01000204f7c1e5d8dadac6b742"), // remote=/165.227.26.175:9030
            hex_to_bytes("dee2aca3fb2e076572676f726566040005126572676f2d6d61696e6e65742d342e302e310108d5efc1d0c64602100400010001030e01000204eecc9582ffaaafeeaa01"), // 213.239.193.208:9030
            hex_to_bytes("ed8aada3fb2e076572676f726566030306196d61696e6e65742d736565642d6e6f64652d746f726f6e746f01089f59740fc64602100400010001030e01000204d1a098d9dff69fb3e601"), // 159.89.116.15:9030
            hex_to_bytes("e3b1ada3fb2e076572676f726566030306126572676f2d6d61696e6e65742d342e302e300002100400010001030d01000204b4cbc4c4f19ce7cc56") //83.212.114.255:9030
        ];
        for b in bytes.iter() {
            println!("{:?}", b);
            println!("{:?}", Handshake::parse(b));
        }


    }

    #[test]
    fn test_hs_ergo_vector() {
        let hs_bytes = hex_to_bytes("bcd2919cee2e076572676f726566030306126572676f2d6d61696e6e65742d332e332e36000210040001000102067f000001ae46");
        let a = Handshake::parse(&hs_bytes).ok().expect("ergo reference test vector failed");
        println!("{:?}", a);
        println!("{:?}", hs_bytes);
        println!("{:?}", a.serialize().unwrap());
        assert_eq!(&hs_bytes[5..], &a.serialize().unwrap()[5..]);
    }
}
