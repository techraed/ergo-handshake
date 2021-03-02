use std::convert::TryFrom;
use std::ops::Deref;

use super::errors::ModelParseError;

#[derive(Debug, PartialEq, Eq)]
pub struct ShortString(String);

impl ShortString {
    pub const MAX_SIZE: usize = 255;
}

impl TryFrom<Vec<u8>> for ShortString {
    type Error = ModelParseError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        if data.len() > Self::MAX_SIZE {
            return Err(ModelParseError::InvalidShortStringLength(data.len()));
        }
        let s = String::from_utf8(data)?;
        Ok(Self(s))
    }
}

impl Deref for ShortString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
