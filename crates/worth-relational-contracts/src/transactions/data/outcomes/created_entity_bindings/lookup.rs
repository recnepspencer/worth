//! Zero-allocation borrowed lookups for entity creation bindings.
use std::borrow::Borrow;
use std::cmp::Ordering;

use super::{CommitCreatedEntityBindings, StableCreatedEntityRef};
use crate::identity::data::{EntityId, KindId, PartitionId};
use crate::symbols::data::ClientKey;
use crate::transactions::data::primitives::CreatedEntityRef;

trait ExactEntityLookupKey {
    fn key(&self) -> (PartitionId, KindId, &ClientKey);
}

impl ExactEntityLookupKey for CreatedEntityRef {
    fn key(&self) -> (PartitionId, KindId, &ClientKey) {
        (self.partition_id, self.kind_id, &self.client_key)
    }
}

struct BorrowedExactEntityKey<'a>(PartitionId, KindId, &'a ClientKey);

impl ExactEntityLookupKey for BorrowedExactEntityKey<'_> {
    fn key(&self) -> (PartitionId, KindId, &ClientKey) {
        (self.0, self.1, self.2)
    }
}

impl<'a> Borrow<dyn ExactEntityLookupKey + 'a> for CreatedEntityRef {
    fn borrow(&self) -> &(dyn ExactEntityLookupKey + 'a) {
        self
    }
}

impl PartialEq for dyn ExactEntityLookupKey + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for dyn ExactEntityLookupKey + '_ {}

impl PartialOrd for dyn ExactEntityLookupKey + '_ {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for dyn ExactEntityLookupKey + '_ {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}

trait StableEntityLookupKey {
    fn key(&self) -> (PartitionId, KindId, &str);
}

impl StableEntityLookupKey for StableCreatedEntityRef {
    fn key(&self) -> (PartitionId, KindId, &str) {
        (self.partition_id, self.kind_id, self.client_key.as_str())
    }
}

struct BorrowedStableEntityKey<'a>(PartitionId, KindId, &'a str);

impl StableEntityLookupKey for BorrowedStableEntityKey<'_> {
    fn key(&self) -> (PartitionId, KindId, &str) {
        (self.0, self.1, self.2)
    }
}

impl<'a> Borrow<dyn StableEntityLookupKey + 'a> for StableCreatedEntityRef {
    fn borrow(&self) -> &(dyn StableEntityLookupKey + 'a) {
        self
    }
}

impl PartialEq for dyn StableEntityLookupKey + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for dyn StableEntityLookupKey + '_ {}

impl PartialOrd for dyn StableEntityLookupKey + '_ {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for dyn StableEntityLookupKey + '_ {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}

impl CommitCreatedEntityBindings {
    pub fn resolve_parts_work_bound(&self, client_key: &ClientKey) -> Option<u64> {
        let raw_bytes = client_key.as_raw_str().map(str::len);
        let comparison_width = raw_bytes.unwrap_or(0).checked_add(4)?;
        let tree_work = |entries: usize| {
            let levels = usize::BITS as usize - entries.max(1).leading_zeros() as usize;
            entries
                .min(11)
                .checked_mul(levels)?
                .checked_add(1)?
                .checked_mul(comparison_width)
        };
        let mut work = tree_work(self.by_exact_reference.len())?;
        if raw_bytes.is_some() {
            work = work.checked_add(tree_work(self.by_stable_reference.len())?)?;
        }
        u64::try_from(work).ok()
    }

    pub fn resolve_parts(
        &self,
        partition_id: PartitionId,
        kind_id: KindId,
        client_key: &ClientKey,
    ) -> Option<EntityId> {
        let exact = BorrowedExactEntityKey(partition_id, kind_id, client_key);
        if let Some(entity_id) = self
            .by_exact_reference
            .get(&exact as &dyn ExactEntityLookupKey)
        {
            return Some(*entity_id);
        }
        let raw = client_key.as_raw_str()?;
        let stable = BorrowedStableEntityKey(partition_id, kind_id, raw);
        self.by_stable_reference
            .get(&stable as &dyn StableEntityLookupKey)
            .copied()
    }
}
