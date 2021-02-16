use sigma_ser::vlq_encode::{ReadSigmaVlqExt, VlqEncodingError};

use crate::utils::default_vlq_reader;

use super::peer_addr::PeerAddr;

const MODE_ID: u8 = 16;
const LOCAL_ADDR_ID: u8 = 2;

#[derive(Debug, PartialEq, Eq)]
pub enum PeerFeature {
    Mode(Mode),
    LocalAddr(PeerAddr),
    Unrecognized,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Mode {
    pub state_type: u8,
    pub is_verifying: bool,
    pub nipopow_suffix_len: Option<u32>,
    pub blocks_to_keep: i32,
}

pub(crate) fn parse_feature(id: u8, data: Vec<u8>) -> Result<PeerFeature, VlqEncodingError> {
    match id {
        MODE_ID => parse_mode(data).map(PeerFeature::Mode),
        LOCAL_ADDR_ID => PeerAddr::try_from(data)
            .map(PeerFeature::LocalAddr)
            .map_err(|e| VlqEncodingError::Io(e.to_string())),
        _ => Ok(PeerFeature::Unrecognized),
    }
}

// todo why not ModelError
fn parse_mode(data: Vec<u8>) -> Result<Mode, VlqEncodingError> {
    let mut reader = default_vlq_reader(data);

    let state_type = reader.get_u8()?;
    let is_verifying = reader.get_u8()? == 1;
    let nipopow_suffix_len = {
        let is_nipopow = reader.get_u8()? == 1;
        if is_nipopow {
            Some(reader.get_u32()?)
        } else {
            None
        }
    };
    let blocks_to_keep = reader.get_i32()?;

    Ok(Mode {
        state_type,
        is_verifying,
        nipopow_suffix_len,
        blocks_to_keep,
    })
}

// pub(super) trait SerializableFeature {
//     fn write(&self, writer: &mut Cursor<Vec<u8>>) {
//         // to strict writer type
//         writer.put_u8(self.feature_id());
//         let feature_bytes = self.to_bytes();
//         writer.put_u16(feature_bytes.len() as u16); // need check
//         writer.write(&feature_bytes);
//     }
//
//     fn to_bytes(&self) -> Vec<u8>; // 65535
//
//     fn feature_id(&self) -> u8; // no const for box dyn
// }
//
// impl SerializableFeature for Mode {
//     fn to_bytes(&self) -> Vec<u8> {
//         let mut writer = Cursor::new(Vec::new());
//         writer.put_u8(self.state_type);
//         writer.put_u8(self.is_verifying as u8);
//         writer.put_u8(self.is_nipopow as u8);
//         writer.put_i32(self.blocks_kept);
//
//         writer.into_inner()
//     }
//
//     #[inline]
//     fn feature_id(&self) -> u8 {
//         MODE_ID
//     }
// }
//
// impl SerializableFeature for LocalAddress {
//     fn to_bytes(&self) -> Vec<u8> {
//         let LocalAddress(ref socket_addr) = self;
//         let mut writer = Cursor::new(Vec::new());
//         match socket_addr.ip() {
//             IpAddr::V4(v4) => writer.write(&v4.octets()),
//             IpAddr::V6(v6) => writer.write(&v6.octets()), // spec wants 6 bytes!
//         };
//         writer.put_u32(socket_addr.port() as u32);
//
//         writer.into_inner()
//     }
//
//     #[inline]
//     fn feature_id(&self) -> u8 {
//         LOCAL_ADDR_ID
//     }
// }
