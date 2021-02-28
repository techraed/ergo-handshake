use std::convert::{TryInto, TryFrom};
use std::io::{Write, Read};

use sigma_ser::vlq_encode::{WriteSigmaVlqExt, ReadSigmaVlqExt};

use crate::models::MagicBytes;
use crate::utils::{default_vlq_reader, default_vlq_writer, TryIntoVlq, TryFromVlq};

use super::{FeatureSerializeError, FeatureParseError};

#[derive(Debug, PartialEq, Eq)]
pub struct SessionId {
    pub magic: MagicBytes,
    pub session_id: i64,
}

impl TryFromVlq for SessionId {
    type Error = FeatureParseError;

    fn try_from_vlq(data: Vec<u8>) -> Result<Self, Self::Error> {
        let mut vlq_reader = default_vlq_reader(data);

        let magic = {
            let mut m = MagicBytes::default();
            vlq_reader.read_exact(&mut m.0)?;
            m
        };
        let session_id = vlq_reader.get_i64()?;

        Ok(SessionId { magic, session_id })
    }
}

impl TryIntoVlq for SessionId {
    type Error = FeatureSerializeError;

    fn try_into_vlq(&self) -> Result<Vec<u8>, Self::Error> {
        let mut vlq_writer = default_vlq_writer(Vec::new());
        let SessionId { magic, session_id } = self;
        let MagicBytes(magic) = magic;

        vlq_writer.write(magic.as_ref())?;
        vlq_writer.put_i64(*session_id)?;

        Ok(vlq_writer.into_inner())
    }
}
