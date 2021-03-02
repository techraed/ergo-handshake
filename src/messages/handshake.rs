use std::convert::TryFrom;
use std::io;
use std::ops::{Deref, DerefMut};

use sigma_ser::vlq_encode::{ReadSigmaVlqExt, VlqEncodingError, WriteSigmaVlqExt};
use thiserror::Error;

use crate::encoding::vlq::{default_vlq_reader, default_vlq_writer, TryFromVlq, TryIntoVlq};
use crate::features::{Features, FeaturesError, PeerFeature};
use crate::models::{ModelParseError, ModelSerializeError, PeerAddr, ShortString, Version};
use crate::utils::make_timestamp;

use spec_reader::HSSpecReader;
pub use spec_reader::HsSpecReaderError;
use spec_writer::HSSpecWriter;
pub use spec_writer::HsSpecWriterError;

#[derive(Debug, PartialEq, Eq)]
pub struct Handshake {
    pub agent_name: ShortString,
    pub version: Version,
    pub peer_name: ShortString,
    pub pub_address: Option<PeerAddr>,
    pub features: Option<Features>,
}

impl Handshake {
    // todo-crucial max size?
    pub fn parse(data: &[u8]) -> Result<Self, HsSpecReaderError> {
        let mut hs_reader = HSSpecReader::new(default_vlq_reader(data));

        let _timestamp = hs_reader.get_u64()?;
        let agent_name = hs_reader.read_short_string()?;
        let version = hs_reader.read_version()?;
        let peer_name = hs_reader.read_short_string()?;
        // todo-minor hide in reader (+ read_opt and read_bool)
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
            // todo-minor move to spec reader?
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

    pub fn serialize(&self) -> Result<Vec<u8>, HsSpecWriterError> {
        let mut hs_writer = HSSpecWriter::new(default_vlq_writer(Vec::new()));

        hs_writer.put_u64(make_timestamp())?;
        hs_writer.write_short_string(&self.agent_name)?;
        hs_writer.write_version(&self.version)?;
        hs_writer.write_short_string(&self.peer_name)?;
        // todo-minor hide in writer (+ write_opt and write_bool)
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

mod spec_reader {

    use super::*;

    // todo-minor CannotReadShortStringLength and such?
    #[derive(Error, Debug)]
    pub enum HsSpecReaderError {
        #[error("Can't read model: {0}")]
        CannotReadModelFromBytes(#[from] ModelParseError),
        #[error("Can't read received bytes: {0}")]
        CannotReadBytes(#[from] io::Error),
        #[error("Received peer address data length is {0}. Should be at least: {1}")]
        TooShortPeerAddrDataLength(u8, u8),
        #[error("Can't read feature: {0}")]
        CannotReadPeerFeatureFromBytes(#[from] FeaturesError),
        #[error("Decoding data failed")]
        // todo-tmp VlqEncodingError doesn't impl Error. VlqDecodingError::VlqDecodingError tells us nothing
        CannotVlqDecodeData(VlqEncodingError),
    }

    pub(super) struct HSSpecReader<R: ReadSigmaVlqExt>(R);

    // tmp, until VlqEncodingError is fixed
    impl From<VlqEncodingError> for HsSpecReaderError {
        fn from(err: VlqEncodingError) -> Self {
            HsSpecReaderError::CannotVlqDecodeData(err)
        }
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
        pub(super) fn new(reader: R) -> Self {
            Self(reader)
        }

        pub(super) fn read_short_string(&mut self) -> Result<ShortString, HsSpecReaderError> {
            let len = self.get_u8()?;
            let buf = self.read_model_data(len as usize)?;
            ShortString::try_from(buf).map_err(HsSpecReaderError::CannotReadModelFromBytes)
        }

        pub(super) fn read_version(&mut self) -> Result<Version, HsSpecReaderError> {
            let mut v = Version::default();
            self.read_exact(&mut v.0)?;
            Ok(v)
        }

        pub(super) fn read_peer_addr(&mut self) -> Result<PeerAddr, HsSpecReaderError> {
            let len = self.get_u8()?;
            if let Some(len) = len.checked_sub(Self::PORT_EXCESS_BYTES) {
                let buf = self.read_model_data(len as usize)?;
                return PeerAddr::try_from_vlq(buf).map_err(HsSpecReaderError::CannotReadModelFromBytes);
            }
            Err(HsSpecReaderError::TooShortPeerAddrDataLength(len, PeerAddr::SIZE_IPv4_SOCKET as u8 + Self::PORT_EXCESS_BYTES))
        }

        pub(super) fn read_features(&mut self) -> Result<Option<Features>, HsSpecReaderError> {
            let features_num = self.get_u8().ok();
            if let Some(mut num) = features_num {
                let mut features = Vec::with_capacity(num as usize);
                while num != 0 {
                    let feature_id = self.get_u8()?;
                    let feature_data = {
                        let len = self.get_u16()?;
                        self.read_model_data(len as usize)?
                    };
                    let feature_res = PeerFeature::try_from((feature_id, feature_data))?;
                    features.push(feature_res);
                    num -= 1;
                }
                return Features::try_new(features)
                    .map(|f| Some(f))
                    .map_err(HsSpecReaderError::CannotReadPeerFeatureFromBytes);
            }
            Ok(None)
        }

        fn read_model_data(&mut self, len: usize) -> Result<Vec<u8>, HsSpecReaderError> {
            let mut buf = vec![0; len];
            self.read_exact(&mut buf)?;
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
}

mod spec_writer {

    use super::*;

    #[derive(Error, Debug)]
    pub enum HsSpecWriterError {
        #[error("Can't write model to buffer: {0}")]
        CannotWriteModel(#[from] ModelSerializeError),
        #[error("Can't write bytes to resource: {0}")]
        CannotWriteBytes(#[from] io::Error),
        #[error("Can't write feature: {0}")]
        CannotWritePeerFeature(#[from] FeaturesError),
    }

    pub(super) struct HSSpecWriter<W: WriteSigmaVlqExt>(W);

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
        pub(super) fn new(writer: W) -> Self {
            Self(writer)
        }

        pub(super) fn into_inner(self) -> W {
            self.0
        }

        pub(super) fn write_short_string(&mut self, short_string: &ShortString) -> Result<(), HsSpecWriterError> {
            let data = short_string.as_bytes();
            self.put_u8(data.len() as u8)?;
            self.write_all(&data).map_err(HsSpecWriterError::CannotWriteBytes)
        }

        pub(super) fn write_version(&mut self, version: &Version) -> Result<(), HsSpecWriterError> {
            let Version(data) = version;
            self.write_all(data).map_err(HsSpecWriterError::CannotWriteBytes)
        }

        pub(super) fn write_peer_addr(&mut self, peer_addr: &PeerAddr) -> Result<(), HsSpecWriterError> {
            let data = peer_addr.try_into_vlq()?;
            self.put_u8(data.len() as u8 + Self::PORT_EXCESS_BYTES)?;
            self.write_all(&data).map_err(HsSpecWriterError::CannotWriteBytes)
        }

        pub(super) fn write_features(&mut self, features: &Features) -> Result<(), HsSpecWriterError> {
            self.put_u8(features.len() as u8)?;
            for feature in features.iter() {
                self.write_feature(feature)?;
            }
            Ok(())
        }

        fn write_feature(&mut self, feature: &PeerFeature) -> Result<(), HsSpecWriterError> {
            self.put_u8(feature.get_id())?;
            let data = feature.try_into_vlq()?;
            self.put_u16(data.len() as u16)?;
            self.write_all(&data).map_err(HsSpecWriterError::CannotWriteBytes)
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

#[cfg(test)]
mod tests {
    use std::net::ToSocketAddrs;

    use hex;

    use crate::features::{Mode, SessionId};
    use crate::models::MagicBytes;

    use super::*;

    fn hex_to_bytes(s: &str) -> Vec<u8> {
        hex::decode(s).expect("internal error: invalid hex str")
    }

    fn create_hs(agent_name: &str, version: Version, peer_name: &str, pub_address: Option<PeerAddr>, features: Option<Features>) -> Handshake {
        let agent_name = ShortString::try_from(agent_name.to_string().into_bytes()).expect("invalid agent name");
        let peer_name = ShortString::try_from(peer_name.to_string().into_bytes()).expect("invalid peer name");
        Handshake { agent_name, version, peer_name, pub_address, features}
    }

    fn create_peer_addr(addr: &str) -> PeerAddr {
        let sock_addr = addr.to_socket_addrs().map(|mut addr| addr.next()).expect("invalid sock addr str");
        sock_addr.map(PeerAddr).expect("invalid sock addr str")
    }

    fn create_features(features: Vec<PeerFeature>) -> Features {
        Features::try_new(features).expect("invalid features vec length")
    }

    fn create_mode_pf(state_type: u8, is_verifying: bool, nipopow_suffix_len: Option<u32>, blocks_to_keep: i32) -> PeerFeature {
        PeerFeature::Mode(Mode { state_type, is_verifying, nipopow_suffix_len, blocks_to_keep })
    }

    fn create_session_id_pf(magic: MagicBytes, session_id: i64) -> PeerFeature {
        PeerFeature::SessionId(SessionId { magic, session_id  })
    }

    fn create_local_addr_pf(addr: &str) -> PeerFeature {
        PeerFeature::LocalAddr(create_peer_addr(addr))
    }

    fn real_app_test_cases() -> Vec<(Handshake, Vec<u8>)> {
        let case1 = {
            let hs = create_hs(
                "ergoref",
                Version([4, 0, 5]),
                "ergo-mainnet-4.0.0",
                None,
                Some(create_features(vec![
                    create_mode_pf(0, true, None, -1),
                    create_session_id_pf(MagicBytes([1, 0, 2, 4]), -7226886467503878579)
                ]))
            );
            let hs_bytes = hex_to_bytes("c3bcaca3fb2e076572676f726566040005126572676f2d6d61696e6e65742d342e302e300002100400010001030e01000204e5c6abfafabc87cbc801");
            (hs, hs_bytes)
        };
        let case2 = {
            let hs = create_hs(
                "ergoref",
                Version([3, 3, 6]),
                "mainnet-seed-node-sf",
                Some(create_peer_addr("165.227.26.175:9030")),
                Some(create_features(vec![
                    create_mode_pf(0, true, None, -1),
                    create_session_id_pf(MagicBytes([1, 0, 2, 4]), -2393537216959524988)
                ]))
            );
            let hs_bytes = hex_to_bytes("93bdaca3fb2e076572676f726566030306146d61696e6e65742d736565642d6e6f64652d73660108a5e31aafc64602100400010001030d01000204f7c1e5d8dadac6b742");
            (hs, hs_bytes)
        };
        let case3 = {
            let hs = create_hs(
                "ergoref",
                Version([4, 0, 5]),
                "ergo-mainnet-4.0.1",
                Some(create_peer_addr("213.239.193.208:9030")),
                Some(create_features(vec![
                    create_mode_pf(0, true, None, -1),
                    create_session_id_pf(MagicBytes([1, 0, 2, 4]), 6155961833357488951)
                ]))
            );
            let hs_bytes = hex_to_bytes("dee2aca3fb2e076572676f726566040005126572676f2d6d61696e6e65742d342e302e310108d5efc1d0c64602100400010001030e01000204eecc9582ffaaafeeaa01");
            (hs, hs_bytes)
        };
        let case4 = {
            let hs = create_hs(
                "ergoref",
                Version([3, 3, 6]),
                "mainnet-seed-node-toronto",
                Some(create_peer_addr("159.89.116.15:9030")),
                Some(create_features(vec![
                    create_mode_pf(0, true, None, -1),
                    create_session_id_pf(MagicBytes([1, 0, 2, 4]), -8301048747963648041)
                ]))
            );
            let hs_bytes = hex_to_bytes("ed8aada3fb2e076572676f726566030306196d61696e6e65742d736565642d6e6f64652d746f726f6e746f01089f59740fc64602100400010001030e01000204d1a098d9dff69fb3e601");
            (hs, hs_bytes)
        };
        let case5 = {
            let hs = create_hs(
                "ergoref",
                Version([3, 3, 6]),
                "ergo-mainnet-4.0.0",
                None,
                Some(create_features(vec![
                    create_mode_pf(0, true, None, -1),
                    create_session_id_pf(MagicBytes([1, 0, 2, 4]), 3120095637531038426)
                ]))
            );
            let hs_bytes = hex_to_bytes("e3b1ada3fb2e076572676f726566030306126572676f2d6d61696e6e65742d342e302e300002100400010001030d01000204b4cbc4c4f19ce7cc56");
            (hs, hs_bytes)
        };
        vec![case1, case2, case3, case4, case5]
    }

    fn run_test(hs: Handshake, hs_bytes: Vec<u8>) {
        let hs_actual = Handshake::parse(&hs_bytes);
        assert!(hs_actual.is_ok());
        let hs_actual = hs_actual.expect("can't parse hs bytes");
        assert_eq!(hs_actual, hs);

        let hs_bytes_actual = hs_actual.serialize();
        assert!(hs_bytes_actual.is_ok());
        let hs_bytes_actual = hs_bytes_actual.expect("can't serialize hs msg");
        // avoiding timestamp bytes
        assert_eq!(&hs_bytes_actual[5..], &hs_bytes[5..]);
    }

    #[test]
    fn test_base_ergo_case() {
        // from https://github.com/ergoplatform/ergo/blob/8ad8818bb0a2bc8df3be88259e379bad7221dc68/src/test/scala/org/ergoplatform/network/HandshakeSpecification.scala
        let hs_expected = create_hs(
            "ergoref",
            Version([3, 3, 6]),
            "ergo-mainnet-3.3.6",
            None,
            Some(create_features(vec![
                create_mode_pf(0, true, None, -1),
                create_local_addr_pf("127.0.0.1:9006")
            ]))
        );
        let hs_bytes = hex_to_bytes("bcd2919cee2e076572676f726566030306126572676f2d6d61696e6e65742d332e332e36000210040001000102067f000001ae46");

        run_test(hs_expected, hs_bytes)
    }

    #[test]
    fn test_real_app_cases() {
        for (hs_expected, hs_bytes) in real_app_test_cases() {
            run_test(hs_expected, hs_bytes)
        }
    }
}
