use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub struct FourCC(pub [u8; 4]);

impl FourCC {
    pub fn new(bytes: [u8; 4]) -> Self {
        Self(bytes.clone())
    }
}

impl From<[u8; 4]> for FourCC {
    fn from(bytes: [u8; 4]) -> Self {
        Self(bytes)
    }
}

impl From<&[u8; 4]> for FourCC {
    fn from(bytes: &[u8; 4]) -> Self {
        Self(bytes.clone())
    }
}