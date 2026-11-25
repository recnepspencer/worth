use serde::{Deserialize, Serialize};

use crate::symbols::data::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PartitionId(pub u32);

impl PartitionId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn main() -> Self {
        Self(0)
    }

    pub const fn as_u32(self) -> u32 {
        self.0
    }

    pub const fn as_u64(self) -> u64 {
        self.0 as u64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LocalSlot(pub u64);

impl LocalSlot {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Generation(pub u32);

impl Generation {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn as_u32(self) -> u32 {
        self.0
    }

    pub const fn as_u64(self) -> u64 {
        self.0 as u64
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VersionId(pub u64);

impl VersionId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub const fn saturating_next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionBound(pub VersionId);

impl VersionBound {
    pub const fn new(version: VersionId) -> Self {
        Self(version)
    }

    pub const fn version(self) -> VersionId {
        self.0
    }

    pub const fn includes_created(self, created_at: VersionId) -> bool {
        created_at.as_u64() <= self.version().as_u64()
    }

    pub const fn retains_retired(self, retired_at: VersionId) -> bool {
        retired_at.as_u64() > self.version().as_u64()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LineageId(pub u64);

impl LineageId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct KindId(pub u32);

impl KindId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn as_u32(self) -> u32 {
        self.0
    }

    pub const fn as_u64(self) -> u64 {
        self.0 as u64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StructuralFingerprint {
    pub family: Symbol,
    pub value: u128,
}

impl StructuralFingerprint {
    pub const fn new(family: Symbol, value: u128) -> Self {
        Self { family, value }
    }
}

#[cfg(test)]
mod tests {
    use super::{Generation, LocalSlot, PartitionId, VersionBound, VersionId};

    #[test]
    fn primitive_identity_accessors_expose_named_numeric_surfaces() {
        assert_eq!(PartitionId::new(7).as_u32(), 7);
        assert_eq!(PartitionId::new(7).as_u64(), 7);
        assert_eq!(LocalSlot::new(11).as_u64(), 11);
        assert_eq!(LocalSlot::new(11).index(), 11usize);
        assert_eq!(Generation::new(3).as_u32(), 3);
        assert_eq!(Generation::new(3).as_u64(), 3);
        assert_eq!(VersionId::new(5).as_u64(), 5);
        assert!(VersionId::new(0).is_zero());
        assert_eq!(VersionId::new(5).saturating_next(), VersionId::new(6));
    }

    #[test]
    fn version_bound_uses_named_version_accessors() {
        let bound = VersionBound::new(VersionId::new(8));

        assert!(bound.includes_created(VersionId::new(8)));
        assert!(bound.retains_retired(VersionId::new(9)));
        assert!(!bound.retains_retired(VersionId::new(8)));
    }
}
