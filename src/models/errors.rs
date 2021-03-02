use std::io;
use std::string::FromUtf8Error;

use thiserror::Error;

use super::PeerAddr;
use super::ShortString;

#[derive(Error, Debug)]
pub enum ModelParseError {
    #[error("Can't create ShortString from buffer with length {0}. Should be {}", ShortString::MAX_SIZE)]
    InvalidShortStringLength(usize),
    #[error("Received invalid data: {0}")]
    InvalidUtf8Buffer(#[from] FromUtf8Error),
    #[error(
        "Can't create HSPeerAddr from buffer with length {0}. Should be {} or {}",
        PeerAddr::SIZE_IPv4_SOCKET,
        PeerAddr::SIZE_IPv6_SOCKET
    )]
    InvalidPeerAddrLength(usize),
}

#[derive(Error, Debug)]
pub enum ModelSerializeError {
    #[error("Model can't be written to resource: {0}")]
    CannotWriteData(#[from] io::Error),
}
