use std::convert::TryFrom;
use std::ops::{Deref, DerefMut};

use crate::models::PeerAddr;
use crate::utils::{TryFromVlq, TryIntoVlq};

pub use feature_errors::*;
pub use mode::Mode;
pub use session_id::SessionId;

use errors as feature_errors;

mod errors;
mod mode;
mod session_id;

#[derive(Debug, PartialEq, Eq)]
pub struct Features(Vec<PeerFeature>);

#[derive(Debug, PartialEq, Eq)]
pub enum PeerFeature {
    Mode(Mode),
    LocalAddr(PeerAddr),
    SessionId(SessionId),
    Unrecognized,
}

impl Features {
    pub const MAX_LEN: usize = 255;

    pub fn try_new(features: Vec<PeerFeature>) -> Result<Self, FeaturesError> {
        if features.len() > 255 {
            return Err(FeaturesError::TooMuchFeatures(features.len()));
        }
        Ok(Self(features))
    }
}

impl Deref for Features {
    type Target = Vec<PeerFeature>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Features {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PeerFeature {
    pub const MODE_ID: u8 = 16;
    pub const LOCAL_ADDR_ID: u8 = 2;
    pub const SESSION_ID: u8 = 3;

    pub fn get_id(&self) -> u8 {
        match self {
            PeerFeature::Mode(_) => Self::MODE_ID,
            PeerFeature::LocalAddr(_) => Self::LOCAL_ADDR_ID,
            PeerFeature::SessionId(_) => Self::SESSION_ID,
            PeerFeature::Unrecognized => panic!("unrecognized features was used"),
        }
    }
}

impl TryFrom<(u8, Vec<u8>)> for PeerFeature {
    type Error = FeaturesError;

    fn try_from((id, data): (u8, Vec<u8>)) -> Result<Self, Self::Error> {
        let res = match id {
            PeerFeature::MODE_ID => Mode::try_from_vlq(data).map(PeerFeature::Mode),
            PeerFeature::LOCAL_ADDR_ID => PeerAddr::try_from_vlq(data)
                .map(PeerFeature::LocalAddr)
                .map_err(FeatureParseError::CannotParseLocalAddress),
            PeerFeature::SESSION_ID => SessionId::try_from_vlq(data).map(PeerFeature::SessionId),
            _ => Ok(PeerFeature::Unrecognized),
        };
        res.map_err(FeaturesError::CannotParseFeature)
    }
}

impl TryIntoVlq for PeerFeature {
    type Error = FeaturesError;

    fn try_into_vlq(&self) -> Result<Vec<u8>, Self::Error> {
        let res = match self {
            PeerFeature::Mode(mode) => mode.try_into_vlq(),
            PeerFeature::LocalAddr(peer_addr) => peer_addr.try_into_vlq().map_err(FeatureSerializeError::CannotSerializeLocalAddress),
            PeerFeature::SessionId(session_id) => session_id.try_into_vlq(),
            PeerFeature::Unrecognized => panic!("unrecognized features was used"),
        };
        res.map_err(FeaturesError::CannotSerializeFeature)
    }
}
