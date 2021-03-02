use std::convert::TryFrom;

use ergo_handshake::handshaking;
use ergo_handshake::models::{ShortString, Version};
use ergo_handshake::messages::Handshake;

fn main() {
    // Run locally ergo node
    // It is usually upped locally at 0.0.0.0:9030
    let remote_node_addr = "0.0.0.0:9030";
    let (conn, received_hs) = handshaking(remote_node_addr, my_default_hs()).expect("can't perform handshake with ergo node");
    // use further `conn` with a remote node and `received_hs` from it
}

fn my_default_hs() -> Handshake {
    let short_string = |s: &str| ShortString::try_from(s.to_string().into_bytes()).expect("invalid short string");

    Handshake {
        agent_name: short_string("ergoref"),
        version: Version([3, 3, 6]),
        peer_name: short_string("ergo-mainnet"),
        pub_address: None,
        features: None
    }
}