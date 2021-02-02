use std::time::Duration;
use std::net::SocketAddr;

struct Handshake {
    agent_name: String,
    version: Version,
    peer_name: String,
    is_public: bool,
    pub_address: SocketAddr,
    features: Option<Vec<PeerFeature>>
}

struct Version;

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
