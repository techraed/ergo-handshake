use std::time::Duration;
use std::net::SocketAddr;

// TODO
/*
Recommended VLQ impls:
1. https://docs.rs/vint64/1.0.1/vint64/
2. https://docs.rs/vlq-rust/0.4.0/vlq_rust/
3. https://github.com/ergoplatform/sigma-rust/blob/master/sigma-ser/src/vlq_encode.rs

Also search for VLQ + ZigZag impl. It's necessary, because type are being encoded in
Handshake message using VLQ + ZigZag.
*/

struct Handshake {
    agent_name: String,
    version: Version,
    peer_name: String,
    is_public: bool,
    pub_address: SocketAddr,
    features: Option<Vec<PeerFeature>>
}

struct Version(u32, u32, u32);

enum PeerFeature {}

impl Handshake {
    pub(crate) fn serialize(self) -> Vec<u8> {
        todo!()
    }

    pub(crate) fn parse(data: &[u8]) -> Result<Self, HandshakeParseError> {
        todo!()
    }
}

mod errors {
    enum HandshakeParseError {}
    enum HandshakeSerializeError{}
}
