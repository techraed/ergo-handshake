use std::io;
use std::ops::{Deref, DerefMut};

use sigma_ser::peekable_reader::PeekableReader;
use sigma_ser::vlq_encode::{ReadSigmaVlqExt, VlqEncodingError};

use crate::models::{parse_feature, PeerAddr, PeerFeature, ShortString, Version};

pub(crate) type DefaultVlqReader<T> = PeekableReader<io::Cursor<T>>;

pub(crate) struct HSSpecReader<R: ReadSigmaVlqExt>(R);

// todo-minor: get_vlq_reader(type, data) - shall be discussed
pub(crate) fn default_vlq_reader<T: AsRef<[u8]>>(data: T) -> DefaultVlqReader<T> {
    PeekableReader::new(io::Cursor::new(data))
}

impl<R: ReadSigmaVlqExt> HSSpecReader<R> {
    pub(crate) fn new(reader: R) -> Self {
        Self(reader)
    }

    pub(crate) fn read_short_string(&mut self) -> Result<ShortString, VlqEncodingError> {
        let buf = self.read_model_data()?;
        ShortString::try_from(buf).map_err(|e| VlqEncodingError::Io(e.to_string()))
    }

    pub(crate) fn read_version(&mut self) -> Result<Version, VlqEncodingError> {
        let mut v = Version::default();
        self.read_exact(&mut v.0).map_err(VlqEncodingError::from)?;
        Ok(v)
    }

    pub(crate) fn read_peer_addr(&mut self) -> Result<PeerAddr, VlqEncodingError> {
        let buf = self.read_model_data()?;
        PeerAddr::try_from(buf).map_err(|e| VlqEncodingError::Io(e.to_string()))
    }

    pub(crate) fn read_features(&mut self) -> Result<Option<Vec<PeerFeature>>, VlqEncodingError> {
        let features_num = self.get_u8().ok();
        if let Some(mut num) = features_num {
            let mut ret = Vec::with_capacity(num as usize);
            while num != 0 {
                // todo-minor move to parse feature?
                let feature_id = self.get_u8()?;
                let feature_data = self.read_model_data()?;
                let feature_res = parse_feature(feature_id, feature_data)?;
                ret.push(feature_res);
                num -= 1;
            }
            return Ok(Some(ret));
        }
        return Ok(None);
    }

    fn read_model_data(&mut self) -> Result<Vec<u8>, VlqEncodingError> {
        let len = self.get_u16()?; // todo-crucial potentially dangerous, should be discussed
        let mut buf = vec![0; len as usize];
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
