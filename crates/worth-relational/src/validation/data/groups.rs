use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InvariantGroup {
    StorageCoherence = 0,
    VersionVisibility = 1,
    AdjacencyIntegrity = 2,
    IdentityCoherence = 3,
    SchemaCompliance = 4,
    LineageIntegrity = 5,
    PublicationCoherence = 6,
    DurabilityConsistency = 7,
    RelationIntegrity = 8,
}

const ALL_INVARIANT_GROUPS: [InvariantGroup; 9] = [
    InvariantGroup::StorageCoherence,
    InvariantGroup::VersionVisibility,
    InvariantGroup::AdjacencyIntegrity,
    InvariantGroup::IdentityCoherence,
    InvariantGroup::SchemaCompliance,
    InvariantGroup::LineageIntegrity,
    InvariantGroup::PublicationCoherence,
    InvariantGroup::DurabilityConsistency,
    InvariantGroup::RelationIntegrity,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InvariantCostClass {
    Touched,
    Partition,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct InvariantGroupSet {
    mask: u32,
}

impl InvariantGroupSet {
    pub const COUNT: usize = InvariantGroup::COUNT;

    pub const fn empty() -> Self {
        Self { mask: 0 }
    }

    pub const fn all() -> Self {
        Self {
            mask: InvariantGroup::all_mask(),
        }
    }

    pub const fn of(group: InvariantGroup) -> Self {
        Self { mask: group.mask() }
    }

    pub const fn union(self, other: Self) -> Self {
        Self {
            mask: self.mask | other.mask,
        }
    }

    pub const fn intersection(self, other: Self) -> Self {
        Self {
            mask: self.mask & other.mask,
        }
    }

    pub const fn contains(self, group: InvariantGroup) -> bool {
        (self.mask & group.mask()) != 0
    }

    pub const fn intersects(self, other: Self) -> bool {
        (self.mask & other.mask) != 0
    }

    pub const fn is_empty(self) -> bool {
        self.mask == 0
    }

    pub const fn mask(self) -> u32 {
        self.mask
    }

    pub const fn from_mask(mask: u32) -> Self {
        Self { mask }
    }
}

impl InvariantGroup {
    pub const COUNT: usize = ALL_INVARIANT_GROUPS.len();

    pub const fn mask(self) -> u32 {
        1u32 << (self as u8)
    }

    pub const fn all() -> [InvariantGroup; Self::COUNT] {
        ALL_INVARIANT_GROUPS
    }

    pub const fn all_mask() -> u32 {
        let mut index = 0;
        let mut mask = 0u32;
        while index < ALL_INVARIANT_GROUPS.len() {
            mask |= ALL_INVARIANT_GROUPS[index].mask();
            index += 1;
        }
        mask
    }
}

#[cfg(test)]
mod tests {
    use super::{InvariantGroup, InvariantGroupSet};

    #[test]
    fn group_sets_intersect_when_they_share_a_group() {
        let left = InvariantGroupSet::of(InvariantGroup::StorageCoherence)
            .union(InvariantGroupSet::of(InvariantGroup::SchemaCompliance));
        let right = InvariantGroupSet::of(InvariantGroup::SchemaCompliance);
        let disjoint = InvariantGroupSet::of(InvariantGroup::VersionVisibility);

        assert!(left.intersects(right));
        assert!(!left.intersects(disjoint));
        assert!(InvariantGroupSet::all().intersects(disjoint));
    }

    #[test]
    fn all_group_mask_covers_every_declared_group() {
        let all = InvariantGroupSet::all();
        for group in InvariantGroup::all() {
            assert!(all.contains(group));
        }
        assert_eq!(InvariantGroupSet::COUNT, InvariantGroup::COUNT);
        assert_eq!(all.mask(), InvariantGroup::all_mask());
    }
}
