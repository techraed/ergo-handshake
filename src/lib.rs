use std::time::SystemTime;
use std::net::SocketAddr;
use std::io::Cursor;

use sigma_ser::vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};

use utils::make_timestamp;
use errors::*;

struct Handshake {
    agent_name: String,
    version: Version,
    peer_name: String,
    is_pub_node: bool,
    pub_address: Option<SocketAddr>,
    features: Option<Vec<PeerFeature>>
}

struct Version(u32, u32, u32);

enum PeerFeature {}

impl Handshake {
    pub(crate) fn serialize(self) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::new());

        writer.put_u64(make_timestamp());

    }

    pub(crate) fn parse(data: &[u8]) -> Result<Self, HandshakeParseError> {
        todo!()
    }
}

mod utils {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub(super) fn make_timestamp() -> u64 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("internal error: current time is before unix epoch");
        now.as_millis() as u64
    }
}

mod errors {
    pub(super) enum HandshakeParseError {}
    pub(super) enum HandshakeSerializeError{}
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};

    use hex;
    use sigma_ser::{peekable_reader::PeekableReader, vlq_encode::{WriteSigmaVlqExt, ReadSigmaVlqExt}};

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
        writer.put_u32(9006); // port as u32
        println!("{:?}", writer);

        let mut reader = PeekableReader::new(Cursor::new(vec![174, 70]));
        let a = reader.get_u32().unwrap();
        println!("V: {}", a);
    }
}