use std::io;
use std::convert::TryFrom;
use std::ops::{Deref, DerefMut};

use sigma_ser::peekable_reader::PeekableReader;
use sigma_ser::vlq_encode::{ReadSigmaVlqExt, VlqEncodingError};

use crate::features::{Features, PeerFeature};
use crate::models::{PeerAddr, ShortString, Version};

// todo-minor try better: it should somehow define, that vlq is used
pub trait TryFromVlq: Sized {
    type Error;

    fn try_from_vlq(data: Vec<u8>) -> Result<Self, Self::Error>;
}

pub(crate) type DefaultVlqReader<T> = PeekableReader<io::Cursor<T>>;

pub(crate) struct HSSpecReader<R: ReadSigmaVlqExt>(R);

// todo-minor: get_vlq_reader(type, data) - shall be discussed
pub(crate) fn default_vlq_reader<T: AsRef<[u8]>>(data: T) -> DefaultVlqReader<T> {
    PeekableReader::new(io::Cursor::new(data))
}

impl<R: ReadSigmaVlqExt> HSSpecReader<R> {
    // Used due to public address (de)serialization bug in the reference ergo-node:
    // port length is encoded as 4 bytes rather than 2: https://github.com/hyperledger-labs/Scorex/blob/30f3bea5ddb660f479964b7879912cebc4ee467e/src/main/scala/scorex/core/network/PeerSpec.scala#L49
    const PORT_EXCESS_BYTES: u8 = 2;

    // todo-minor discuss reading lengths approaches: 1) doing it by read fns (more safe) or 2) by 1 generally used `read_next_model` fn.
    // #[test]
    // fn simple() {
    //     let mut w = default_vlq_writer(Vec::new());
    //     w.put_usize_as_u16(10);
    //     w.put_u8(10);
    //     let inner = w.into_inner();
    //     let mut r = default_vlq_reader(inner);
    //     let a = r.get_u64().unwrap();
    //     let b = r.get_u16().unwrap();
    //     assert_eq!(10, a);
    //     assert_eq!(10, b);
    // }
    pub(crate) fn new(reader: R) -> Self {
        Self(reader)
    }

    pub(crate) fn read_short_string(&mut self) -> Result<ShortString, VlqEncodingError> {
        let len = self.get_u8()?;
        let buf = self.read_model_data(len as usize)?;
        ShortString::try_from(buf).map_err(|e| VlqEncodingError::Io(e.to_string()))
    }

    pub(crate) fn read_version(&mut self) -> Result<Version, VlqEncodingError> {
        let mut v = Version::default();
        self.read_exact(&mut v.0).map_err(VlqEncodingError::from)?;
        Ok(v)
    }

    pub(crate) fn read_peer_addr(&mut self) -> Result<PeerAddr, VlqEncodingError> {
        let len = self.get_u8()?;
        if let Some(len) = len.checked_sub(Self::PORT_EXCESS_BYTES) {
            let buf = self.read_model_data(len as usize)?;
            return PeerAddr::try_from_vlq(buf).map_err(|e| VlqEncodingError::Io(e.to_string()));
        }
        Err(VlqEncodingError::Io("todo msg".to_string()))
    }

    pub(crate) fn read_features(&mut self) -> Result<Option<Features>, VlqEncodingError> {
        let features_num = self.get_u8().ok();
        if let Some(mut num) = features_num {
            let mut features = Vec::with_capacity(num as usize);
            while num != 0 {
                let feature_id = self.get_u8()?;
                let feature_data = {
                    let len = self.get_u16()?;
                    self.read_model_data(len as usize)?
                };
                let feature_res = PeerFeature::try_from((feature_id, feature_data)).expect("todo!!!"); // todo crucial
                features.push(feature_res);
                num -= 1;
            }
            return Features::try_new(features).map(|f| Some(f)).map_err(|e| VlqEncodingError::Io(e.to_string()));
        }
        Ok(None)
    }

    fn read_model_data(&mut self, len: usize) -> Result<Vec<u8>, VlqEncodingError> {
        let mut buf = vec![0; len];
        self.read_exact(&mut buf).map_err(VlqEncodingError::from)?;
        Ok(buf)
    }
}

impl<R: ReadSigmaVlqExt> Deref for HSSpecReader<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<R: ReadSigmaVlqExt> DerefMut for HSSpecReader<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
