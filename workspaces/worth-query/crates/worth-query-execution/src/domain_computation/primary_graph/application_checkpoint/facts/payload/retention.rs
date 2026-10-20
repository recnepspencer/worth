//! Custody quotes follow the original immutable allocation tree, never a flat recopy.
use super::{FactPayload, FactStorage};
impl FactPayload {
    pub(in crate::domain_computation::primary_graph) fn retained_children(
        &self,
    ) -> Option<(&Self, &Self)> {
        match &self.storage {
            FactStorage::Joined { children } => Some((&children[0], &children[1])),
            _ => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn retained_local_storage_bytes(
        &self,
    ) -> Option<u64> {
        let bytes = self.retained_storage_bytes()?;
        match self.retained_children() {
            Some((left, right)) => bytes
                .checked_sub(left.retained_storage_bytes()?)?
                .checked_sub(right.retained_storage_bytes()?),
            None => Some(bytes),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn retained_leaf(
        &self,
    ) -> Option<&[super::Fact]> {
        match &self.storage {
            FactStorage::Original(facts) | FactStorage::Decoded { facts, .. } => Some(facts),
            FactStorage::Native(chunk) => Some(chunk.facts.elements()),
            FactStorage::Joined { .. } => None,
        }
    }
}
