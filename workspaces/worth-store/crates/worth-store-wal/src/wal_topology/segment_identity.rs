use crate::{WalTopologyDenial, WalTopologyDenialKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WalSegmentId {
    value: u64,
}

impl WalSegmentId {
    pub const fn new(value: u64) -> Result<Self, WalTopologyDenial> {
        if value == 0 {
            return Err(WalTopologyDenial::new(
                WalTopologyDenialKind::EmptySegmentId,
            ));
        }
        Ok(Self { value })
    }

    pub const fn get(self) -> u64 {
        self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WalSegmentGeneration {
    value: u64,
}

impl WalSegmentGeneration {
    pub const fn new(value: u64) -> Result<Self, WalTopologyDenial> {
        if value == 0 {
            return Err(WalTopologyDenial::new(
                WalTopologyDenialKind::InvalidSegmentGeneration,
            ));
        }
        Ok(Self { value })
    }

    pub const fn get(self) -> u64 {
        self.value
    }
}
