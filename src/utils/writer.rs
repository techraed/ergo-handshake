use std::io;

use sigma_ser::vlq_encode::{VlqEncodingError, WriteSigmaVlqExt};
use std::ops::{Deref, DerefMut};

use crate::models::{serialize_feature, PeerAddr, PeerFeature, ShortString, Version};

pub(crate) type DefaultWriter<T> = io::Cursor<T>;

pub(crate) struct HSSpecWriter<W: WriteSigmaVlqExt>(W);

pub(crate) fn default_vlq_writer<T: AsRef<[u8]>>(data: T) -> DefaultWriter<T> {
    io::Cursor::new(data)
}

impl<W: WriteSigmaVlqExt> HSSpecWriter<W> {
    pub(crate) fn new(writer: W) -> Self {
        Self(writer)
    }

    pub(crate) fn into_inner(self) -> W {
        self.0
    }

    pub(crate) fn write_short_string(&mut self, short_string: &ShortString) -> Result<(), VlqEncodingError> {
        let data = short_string.as_bytes();
        self.write_model_data(data)
    }

    pub(crate) fn write_version(&mut self, version: &Version) -> Result<(), VlqEncodingError> {
        let Version(data) = version;
        self.write_all(data).map_err(|e| VlqEncodingError::Io(e.to_string()))
    }

    pub(crate) fn write_peer_addr(&mut self, peer_addr: &PeerAddr) -> Result<(), VlqEncodingError> {
        let data = peer_addr.as_bytes().map_err(|e| VlqEncodingError::Io(e.to_string()))?;
        self.write_model_data(&data)
    }

    pub(crate) fn write_features(&mut self, features: &[PeerFeature]) -> Result<(), VlqEncodingError> {
        self.put_usize_as_u16(features.len())?; // todo-crucial potentially dangerous, should be discussed
        for feature in features {
            self.put_u8(feature.get_id())?;
            let data = serialize_feature(feature)?;
            self.write_model_data(&data)?;
        }
        Ok(())
    }

    fn write_model_data(&mut self, data: &[u8]) -> Result<(), VlqEncodingError> {
        let len = data.len();
        self.put_usize_as_u16(len)?; // todo-crucial potentially dangerous, should be discussed
        let _ = self.write(data)?;
        Ok(())
    }
}

impl<W: WriteSigmaVlqExt> Deref for HSSpecWriter<W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<W: WriteSigmaVlqExt> DerefMut for HSSpecWriter<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
