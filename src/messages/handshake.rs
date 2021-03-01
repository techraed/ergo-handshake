use std::io::{ErrorKind, Read, Write};

use sigma_ser::vlq_encode::{ReadSigmaVlqExt, VlqEncodingError, WriteSigmaVlqExt};

use crate::models::{PeerAddr, ShortString, Version};
use crate::features::{Features, PeerFeature};
use crate::utils::make_timestamp;
use crate::encoding::vlq::{default_vlq_reader, default_vlq_writer};

use spec::{HSSpecWriter, HSSpecReader};

#[derive(Debug, PartialEq, Eq)]
pub struct Handshake {
    pub agent_name: ShortString,
    pub version: Version,
    pub peer_name: ShortString,
    pub pub_address: Option<PeerAddr>,
    pub features: Option<Features>,
}

// todo-minor HandshakeParser/SerializerError
impl Handshake {
    // todo-crucial max size?
    pub fn parse(data: &[u8]) -> Result<Self, VlqEncodingError> {
        let mut hs_reader = HSSpecReader::new(default_vlq_reader(data));

        let _timestamp = hs_reader.get_u64()?;
        let agent_name = hs_reader.read_short_string()?;
        let version = hs_reader.read_version()?;
        let peer_name = hs_reader.read_short_string()?;
        let pub_address = {
            let is_pub_node = hs_reader.get_u8()? == 1;
            if is_pub_node {
                Some(hs_reader.read_peer_addr()?)
            } else {
                None
            }
        };
        let features = hs_reader
            // moving out unrecognized features
            // todo-minor move to utils::reader?
            .read_features()?
            .map(|mut f| {
                f.retain(|pf| pf != &PeerFeature::Unrecognized);
                if f.len() > 0 {
                    Some(f)
                } else {
                    None
                }
            })
            .flatten();

        Ok(Handshake {
            agent_name,
            version,
            peer_name,
            pub_address,
            features,
        })
    }

    pub fn serialize(&self) -> Result<Vec<u8>, VlqEncodingError> {
        let mut hs_writer = HSSpecWriter::new(default_vlq_writer(Vec::new()));

        hs_writer.put_u64(make_timestamp())?;
        hs_writer.write_short_string(&self.agent_name)?;
        hs_writer.write_version(&self.version)?;
        hs_writer.write_short_string(&self.peer_name)?;
        if let Some(peer_addr) = self.pub_address.as_ref() {
            hs_writer.put_u8(1)?;
            hs_writer.write_peer_addr(peer_addr)?;
        } else {
            hs_writer.put_u8(0)?;
        }
        if let Some(features) = self.features.as_ref() {
            hs_writer.write_features(features)?;
        }

        Ok(hs_writer.into_inner().into_inner())
    }
}

mod spec {
    use std::convert::TryFrom;
    use std::io;
    use std::ops::{Deref, DerefMut};

    use sigma_ser::peekable_reader::PeekableReader;
    use sigma_ser::vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt, VlqEncodingError};

    use crate::features::{Features, PeerFeature};
    use crate::models::{PeerAddr, ShortString, Version};
    use crate::encoding::vlq::{TryFromVlq, TryIntoVlq};

    pub(super) struct HSSpecReader<R: ReadSigmaVlqExt>(R);
    pub(super) struct HSSpecWriter<W: WriteSigmaVlqExt>(W);

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
            let data = peer_addr.try_into_vlq().expect("todo"); // todo-crucial!!;
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
            let data = feature.try_into_vlq().expect("todo"); // todo-crucial!
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

}
