use std::io::{Error as IoError, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use thiserror::Error;

use crate::messages::{Handshake, HsSpecWriterError, HsSpecReaderError};

const HS_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Error, Debug)]
pub enum HandshakingError {
    #[error("Failed IO operation: {0}")]
    FailedIoOp(#[from] IoError),
    #[error("Failed handshake message serialization: {0}")]
    MessageSerializeError(#[from] HsSpecWriterError),
    #[error("Failed handshake message parse: {0}")]
    MessageParseError(#[from] HsSpecReaderError),
}

pub fn handshaking<A: ToSocketAddrs>(addr: A, hs_msg: Handshake) -> Result<(TcpStream, Handshake), HandshakingError> {
    let mut conn = TcpStream::connect(addr)?;
    conn.set_read_timeout(Some(HS_TIMEOUT))
        .expect("internal error: zero duration passed as read timeout");

    let hs_bytes = hs_msg.serialize()?;
    send_hs(&mut conn, &hs_bytes)?;
    read_hs(&mut conn).map(|hs| (conn, hs))
}

fn send_hs(conn: &mut TcpStream, data: &[u8]) -> Result<(), HandshakingError> {
    conn.write_all(data)?;
    conn.flush().map_err(HandshakingError::FailedIoOp)
}

fn read_hs(conn: &mut TcpStream) -> Result<Handshake, HandshakingError> {
    let mut buf = vec![0; 100];
    match conn.read(&mut buf) {
        Ok(_) => {
            conn.set_read_timeout(None).expect("internal error: zero duration passed as read timeout");
            Handshake::parse(&buf).map_err(HandshakingError::MessageParseError)
        }
        Err(e) => Err(HandshakingError::FailedIoOp(e)),
    }
}
