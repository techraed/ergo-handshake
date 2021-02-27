use std::convert::{TryInto, TryFrom};

use sigma_ser::vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};

use crate::utils::{default_vlq_writer, default_vlq_reader};

use super::{FeatureSerializeError, FeatureParseError};

#[derive(Debug, PartialEq, Eq)]
pub struct Mode {
    pub state_type: u8,
    pub is_verifying: bool,
    pub nipopow_suffix_len: Option<u32>,
    pub blocks_to_keep: i32,
}

impl TryFrom<&Mode> for Vec<u8> {
    type Error = FeatureSerializeError;

    fn try_into(&self) -> Result<Vec<u8>, Self::Error> {
        let mut vlq_writer = default_vlq_writer(Vec::new());
        let Mode { state_type, is_verifying, nipopow_suffix_len, blocks_to_keep} = self;

        vlq_writer.put_u8(state_type)?;
        vlq_writer.put_u8(is_verifying as u8)?;
        if let Some(popow_suf) = nipopow_suffix_len {
            vlq_writer.put_u8(1)?;
            vlq_writer.put_u32(popow_suf)?;
        } else {
            vlq_writer.put_u8(0)?;
        }
        vlq_writer.put_i32(blocks_to_keep)?;

        Ok(vlq_writer.into_inner())
    }
}

impl TryFrom<Vec<u8>> for Mode {
    type Error = FeatureParseError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
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
}