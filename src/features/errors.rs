use std::io;

use thiserror::Error;

use sigma_ser::vlq_encode::VlqEncodingError;

use crate::models::{ModelParseError, ModelSerializeError};

use super::Features;

#[derive(Error, Debug)]
pub enum FeaturesError {
    #[error("Can't create Futures with provided amount of PeerFeature - {0}, maximum allowed {}", Features::MAX_LEN)]
    TooMuchFeatures(usize),
    #[error("{0}")]
    CannotSerializeFeature(#[source] FeatureSerializeError),
    #[error("{0}")]
    CannotParseFeature(#[source] FeatureParseError)
}

#[derive(Error, Debug)]
pub enum FeatureParseError {
    #[error("Feature can't be read from resource: {0}")]
    CannotReadData(#[from] io::Error),
    #[error("TODO MESSAGE")] // todo-crucial VLQ doesn't impl Error
    CannotVlqDecodeData(String),
    #[error("{0}")]
    CannotParseLocalAddress(#[source] ModelParseError)
}

#[derive(Error, Debug)]
pub enum FeatureSerializeError {
    #[error("{0}")]
    CannotSerializeLocalAddress(#[source] ModelSerializeError),
    #[error("Feature can't be written to resource: {0}")]
    CannotWriteData(#[from] io::Error)
}

// todo-crucial VLQ doesn't impl Error
impl From<VlqEncodingError> for FeatureParseError {
    fn from(e: VlqEncodingError) -> Self {
        FeatureParseError::CannotVlqDecodeData("TODO".to_string())
    }
}