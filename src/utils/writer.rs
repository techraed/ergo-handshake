use std::io;
use std::convert::TryInto;
use std::ops::{Deref, DerefMut};

use sigma_ser::vlq_encode::{VlqEncodingError, WriteSigmaVlqExt};

use crate::features::{Features, PeerFeature};
use crate::models::{PeerAddr, ShortString, Version};
use crate::models::ModelSerializeError; // todo-tmp

pub(crate) type DefaultVlqWriter<T> = io::Cursor<T>;

pub(crate) struct HSSpecWriter<W: WriteSigmaVlqExt>(W);

pub(crate) fn default_vlq_writer<T: AsRef<[u8]>>(data: T) -> DefaultVlqWriter<T> {
    io::Cursor::new(data)
}

impl<W: WriteSigmaVlqExt> HSSpecWriter<W> {
    // Used due to public address (de)serialization bug in the reference ergo-node:
    // port length is encoded as 4 bytes rather than 2: https://github.com/hyperledger-labs/Scorex/blob/30f3bea5ddb660f479964b7879912cebc4ee467e/src/main/scala/scorex/core/network/PeerSpec.scala#L49
    const PORT_EXCESS_BYTES: u8 = 2;

    // todo-minor discuss putting lengths approaches: 1) doing it by write fns or 2) by 1 generally used `write_model`, which puts usize len as u16.
    // argument for the second approach is in `write_feature` and in simple test
    // #[test]
    // fn simple() {
    //     use crate::utils::default_vlq_reader;
    //
    //     let mut w = default_vlq_writer(Vec::new());
    //     w.put_u32(123123141);
    //     w.put_u8(10);
    //     let inner = w.into_inner();
    //     let mut r = default_vlq_reader(inner);
    //     let a = r.get_u64().unwrap();
    //     let b = r.get_u16().unwrap();
    //     assert_eq!(123123141, a);
    //     assert_eq!(10, b);
    // }
    pub(crate) fn new(writer: W) -> Self {
        Self(writer)
    }

    pub(crate) fn into_inner(self) -> W {
        self.0
    }

    pub(crate) fn write_short_string(&mut self, short_string: &ShortString) -> Result<(), VlqEncodingError> {
        let data = short_string.as_bytes();
        self.put_u8(data.len() as u8)?;
        self.write(&data).map(|_| ()).map_err(VlqEncodingError::from)
    }

    pub(crate) fn write_version(&mut self, version: &Version) -> Result<(), VlqEncodingError> {
        let Version(data) = version;
        self.write_all(data).map_err(VlqEncodingError::from)
    }

    pub(crate) fn write_peer_addr(&mut self, peer_addr: &PeerAddr) -> Result<(), VlqEncodingError> {
        let peer_addr = peer_addr.clone(); // todo-tmp
        let data: Vec<u8> = peer_addr.try_into().expect("todo"); // todo-crucial!!;
        self.put_u8(data.len() as u8 + Self::PORT_EXCESS_BYTES)?;
        self.write(&data).map(|_| ()).map_err(VlqEncodingError::from)
    }

    pub(crate) fn write_features(&mut self, features: &Features) -> Result<(), VlqEncodingError> {
        self.put_u8(features.len() as u8)?;
        for feature in features.iter() {
            self.write_feature(feature)?;
        }
        Ok(())
    }

    fn write_feature(&mut self, feature: &PeerFeature) -> Result<(), VlqEncodingError> {
        self.put_u8(feature.get_id())?;
        let data: Vec<u8> = feature.try_into().expect("todo"); // todo-crucial!
        self.put_u16(data.len() as u16)?;
        self.write(&data).map(|_| ()).map_err(VlqEncodingError::from)
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
