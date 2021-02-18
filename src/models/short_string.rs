use std::ops::Deref;

use super::errors::ModelError;

#[derive(Debug, PartialEq, Eq)]
pub struct ShortString(String);

impl ShortString {
    pub const MAX_SIZE: usize = 255;

    pub fn try_from(data: Vec<u8>) -> Result<Self, ModelError> {
        if data.len() > Self::MAX_SIZE {
            return Err(ModelError::InvalidShortStringLength(data.len()));
        }
        let s = String::from_utf8(data).map_err(ModelError::InvalidUtf8Buffer)?;
        Ok(Self(s))
    }
}

impl Deref for ShortString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
