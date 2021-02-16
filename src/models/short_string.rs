use super::errors::ModelError;

#[derive(Debug, PartialEq, Eq)]
pub struct ShortString(String);

impl ShortString {
    pub const SIZE: usize = 255;

    pub fn try_from(data: Vec<u8>) -> Result<Self, ModelError> {
        if data.len() > Self::SIZE {
            return Err(ModelError::InvalidShortStringLength(data.len()));
        }
        let s = String::from_utf8(data).map_err(ModelError::InvalidUtf8Buffer)?; // err type
        Ok(Self(s))
    }
}
