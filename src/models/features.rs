use std::io::Write;

use sigma_ser::vlq_encode::{ReadSigmaVlqExt, VlqEncodingError, WriteSigmaVlqExt};

use crate::utils::{default_vlq_reader, default_vlq_writer};

use super::peer_addr::PeerAddr;

const MODE_ID: u8 = 16;
const LOCAL_ADDR_ID: u8 = 2;

#[derive(Debug, PartialEq, Eq)]
pub enum PeerFeature {
    Mode(Mode),
    LocalAddr(PeerAddr),
    Unrecognized,
}

impl PeerFeature {
    pub const MODE_ID: u8 = 16;
    pub const LOCAL_ADDR_ID: u8 = 2;

    pub fn get_id(&self) -> u8 {
        match self {
            PeerFeature::Mode(_) => Self::MODE_ID,
            PeerFeature::LocalAddr(_) => Self::LOCAL_ADDR_ID,
            PeerFeature::Unrecognized => unreachable!(), // todo-crucial potentially possible! Should be handled properly!
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Mode {
    pub state_type: u8,
    pub is_verifying: bool,
    pub nipopow_suffix_len: Option<u32>,
    pub blocks_to_keep: i32,
}

pub(crate) fn serialize_feature(feature: &PeerFeature) -> Result<Vec<u8>, VlqEncodingError> {
    match feature {
        PeerFeature::Mode(mode) => serialize_mode(mode),
        PeerFeature::LocalAddr(peer_addr) => peer_addr.as_bytes().map_err(|e| VlqEncodingError::Io(e.to_string())),
        PeerFeature::Unrecognized => unreachable!("unrecognized features used in hand shake instance"),
    }
}

// todo-minor why not ModelError
fn serialize_mode(mode: &Mode) -> Result<Vec<u8>, VlqEncodingError> {
    let mut vlq_writer = default_vlq_writer(Vec::new());

    vlq_writer.put_u8(mode.state_type)?;
    vlq_writer.put_u8(mode.is_verifying as u8)?;
    if let Some(popow_suf) = mode.nipopow_suffix_len {
        vlq_writer.put_u8(1)?;
        vlq_writer.put_u32(popow_suf)?;
    } else {
        vlq_writer.put_u8(0)?;
    }
    vlq_writer.put_i32(mode.blocks_to_keep)?;

    Ok(vlq_writer.into_inner())
}

pub(crate) fn parse_feature(id: u8, data: Vec<u8>) -> Result<PeerFeature, VlqEncodingError> {
    match id {
        PeerFeature::MODE_ID => parse_mode(data).map(PeerFeature::Mode),
        PeerFeature::LOCAL_ADDR_ID => PeerAddr::try_from(data)
            .map(PeerFeature::LocalAddr)
            .map_err(|e| VlqEncodingError::Io(e.to_string())),
        _ => Ok(PeerFeature::Unrecognized),
    }
}

// todo-minor why not ModelError
fn parse_mode(data: Vec<u8>) -> Result<Mode, VlqEncodingError> {
    let mut vlq_reader = default_vlq_reader(data);

    let state_type = vlq_reader.get_u8()?;
    let is_verifying = vlq_reader.get_u8()? == 1;
    let nipopow_suffix_len = {
        let is_nipopow = vlq_reader.get_u8()? == 1;
        if is_nipopow {
            Some(vlq_reader.get_u32()?)
        } else {
            None
        }
    };
    let blocks_to_keep = vlq_reader.get_i32()?;

    Ok(Mode {
        state_type,
        is_verifying,
        nipopow_suffix_len,
        blocks_to_keep,
    })
}
