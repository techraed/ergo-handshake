use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn make_timestamp() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("internal error: current time is before unix epoch");
    now.as_millis() as u64
}

pub(crate) mod reader {
    use std::io;
    use std::ops::{Deref, DerefMut};

    use sigma_ser::peekable_reader::PeekableReader;
    use sigma_ser::vlq_encode::{ReadSigmaVlqExt, VlqEncodingError};

    use crate::models::{HSPeerAddr, ShortString, Version};

    pub(crate) type DefaultVlqReader<T: AsRef<[u8]>> = PeekableReader<io::Cursor<T>>;

    pub(crate) struct HSReader<R: ReadSigmaVlqExt>(R);

    // todo: get_vlq_reader(type, data) - shall be discussed
    pub(crate) fn default_vlq_reader<T: AsRef<[u8]>>(data: T) -> DefaultVlqReader<T> {
        PeekableReader::new(io::Cursor::new(data))
    }

    impl<R: ReadSigmaVlqExt> HSReader<R> {
        pub(crate) fn new(reader: R) -> Self {
            Self(reader)
        }

        pub(crate) fn read_short_string(&mut self) -> Result<ShortString, VlqEncodingError> {
            let buf = self.read_var_len()?;
            ShortString::try_from(buf).map_err(|e| VlqEncodingError::Io(e.to_string()))
        }

        pub(crate) fn read_version(&mut self) -> Result<Version, VlqEncodingError> {
            let mut v = Version::default();
            self.read_exact(&mut v.0).map_err(VlqEncodingError::from)?;
            Ok(v)
        }

        pub(crate) fn read_peer_addr(&mut self) -> Result<HSPeerAddr, VlqEncodingError> {
            let buf = self.read_var_len()?;
            HSPeerAddr::try_from(buf).map_err(|e| VlqEncodingError::Io(e.to_string()))
        }

        fn read_var_len(&mut self) -> Result<Vec<u8>, VlqEncodingError> {
            let len = self.get_u8()?;
            let mut buf = vec![0; len as usize];
            self.read_exact(&mut buf).map_err(VlqEncodingError::from)?;
            Ok(buf)
        }
    }

    impl<R: ReadSigmaVlqExt> Deref for HSReader<R> {
        type Target = R;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<R: ReadSigmaVlqExt> DerefMut for HSReader<R> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
}

mod righter {
    // todo
}
