use std::string::FromUtf8Error;

use thiserror::Error;

use crate::models::{HSPeerAddr, ShortString};

pub(super) struct HandshakeParseError;
pub(super) struct HandshakeSerializeError;

#[derive(Error, Debug)]
pub(super) enum ModelCreationError {
    #[error("Can't create ShortString from buffer with length {0}. Should be {}", ShortString::SIZE)]
    InvalidShortStringLength(usize),
    #[error("Received invalid data: {0}")]
    InvalidUtf8Buffer(#[source] FromUtf8Error),
    #[error(
        "Can't create HSPeerAddr from buffer with length {0}. Should be {} or {}",
        HSPeerAddr::SIZE_IPv4_SOCKET,
        HSPeerAddr::SIZE_IPv6_SOCKET
    )]
    InvalidPeerAddrData(usize),
}
