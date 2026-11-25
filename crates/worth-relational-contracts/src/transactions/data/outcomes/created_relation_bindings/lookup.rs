//! Zero-allocation borrowed lookups for relation creation bindings.
use std::borrow::Borrow;
use std::cmp::Ordering;

use super::{CommitCreatedRelationBindings, StableCreatedRelationRef, StableEntityReference};
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::symbols::data::ClientKey;
use crate::transactions::data::primitives::{CreatedRelationRef, EntityReference};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BorrowedEntityRef<'a> {
    Existing(EntityId),
    Created(PartitionId, KindId, &'a ClientKey),
}

impl<'a> BorrowedEntityRef<'a> {
    pub fn from_ref(reference: &'a EntityReference) -> Self {
        match reference {
            EntityReference::Existing(id) => Self::Existing(*id),
            EntityReference::Created(created) => {
                Self::Created(created.partition_id, created.kind_id, &created.client_key)
            }
        }
    }

    fn as_stable(self) -> Option<BorrowedStableEntityRef<'a>> {
        match self {
            Self::Existing(id) => Some(BorrowedStableEntityRef::Existing(id)),
            Self::Created(partition_id, kind_id, key) => Some(BorrowedStableEntityRef::Created(
                partition_id,
                kind_id,
                key.as_raw_str()?,
            )),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum BorrowedStableEntityRef<'a> {
    Existing(EntityId),
    Created(PartitionId, KindId, &'a str),
}

impl<'a> BorrowedStableEntityRef<'a> {
    fn from_stable(reference: &'a StableEntityReference) -> Self {
        match reference {
            StableEntityReference::Existing(id) => Self::Existing(*id),
            StableEntityReference::Created {
                partition_id,
                kind_id,
                client_key,
            } => Self::Created(*partition_id, *kind_id, client_key.as_str()),
        }
    }
}

trait ExactRelationLookupKey {
    fn key(
        &self,
    ) -> (
        PartitionId,
        KindId,
        &ClientKey,
        BorrowedEntityRef<'_>,
        BorrowedEntityRef<'_>,
    );
}

impl ExactRelationLookupKey for CreatedRelationRef {
    fn key(
        &self,
    ) -> (
        PartitionId,
        KindId,
        &ClientKey,
        BorrowedEntityRef<'_>,
        BorrowedEntityRef<'_>,
    ) {
        (
            self.partition_id,
            self.kind_id,
            &self.client_key,
            BorrowedEntityRef::from_ref(&self.source),
            BorrowedEntityRef::from_ref(&self.target),
        )
    }
}

struct BorrowedExactRelationKey<'a>(
    PartitionId,
    KindId,
    &'a ClientKey,
    BorrowedEntityRef<'a>,
    BorrowedEntityRef<'a>,
);

impl ExactRelationLookupKey for BorrowedExactRelationKey<'_> {
    fn key(
        &self,
    ) -> (
        PartitionId,
        KindId,
        &ClientKey,
        BorrowedEntityRef<'_>,
        BorrowedEntityRef<'_>,
    ) {
        (self.0, self.1, self.2, self.3, self.4)
    }
}

impl<'a> Borrow<dyn ExactRelationLookupKey + 'a> for CreatedRelationRef {
    fn borrow(&self) -> &(dyn ExactRelationLookupKey + 'a) {
        self
    }
}

impl PartialEq for dyn ExactRelationLookupKey + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for dyn ExactRelationLookupKey + '_ {}

impl PartialOrd for dyn ExactRelationLookupKey + '_ {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for dyn ExactRelationLookupKey + '_ {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}

trait StableRelationLookupKey {
    fn key(
        &self,
    ) -> (
        PartitionId,
        KindId,
        &str,
        BorrowedStableEntityRef<'_>,
        BorrowedStableEntityRef<'_>,
    );
}

impl StableRelationLookupKey for StableCreatedRelationRef {
    fn key(
        &self,
    ) -> (
        PartitionId,
        KindId,
        &str,
        BorrowedStableEntityRef<'_>,
        BorrowedStableEntityRef<'_>,
    ) {
        (
            self.partition_id,
            self.kind_id,
            self.client_key.as_str(),
            BorrowedStableEntityRef::from_stable(&self.source),
            BorrowedStableEntityRef::from_stable(&self.target),
        )
    }
}

struct BorrowedStableRelationKey<'a>(
    PartitionId,
    KindId,
    &'a str,
    BorrowedStableEntityRef<'a>,
    BorrowedStableEntityRef<'a>,
);

impl StableRelationLookupKey for BorrowedStableRelationKey<'_> {
    fn key(
        &self,
    ) -> (
        PartitionId,
        KindId,
        &str,
        BorrowedStableEntityRef<'_>,
        BorrowedStableEntityRef<'_>,
    ) {
        (self.0, self.1, self.2, self.3, self.4)
    }
}

impl<'a> Borrow<dyn StableRelationLookupKey + 'a> for StableCreatedRelationRef {
    fn borrow(&self) -> &(dyn StableRelationLookupKey + 'a) {
        self
    }
}

impl PartialEq for dyn StableRelationLookupKey + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for dyn StableRelationLookupKey + '_ {}

impl PartialOrd for dyn StableRelationLookupKey + '_ {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for dyn StableRelationLookupKey + '_ {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}

impl CommitCreatedRelationBindings {
    pub fn resolve_parts_work_bound(
        &self,
        client_key: &ClientKey,
        source: BorrowedEntityRef<'_>,
        target: BorrowedEntityRef<'_>,
    ) -> Option<u64> {
        let endpoint_bytes = |reference: BorrowedEntityRef<'_>| match reference {
            BorrowedEntityRef::Existing(_) => 4usize,
            BorrowedEntityRef::Created(_, _, key) => key
                .as_raw_str()
                .map_or(4, |raw| raw.len().saturating_add(4)),
        };
        let width = client_key
            .as_raw_str()
            .map_or(4, |raw| raw.len().saturating_add(4))
            .checked_add(endpoint_bytes(source))?
            .checked_add(endpoint_bytes(target))?;
        let entries = self
            .by_exact_reference
            .len()
            .max(self.by_stable_reference.len());
        let levels = usize::BITS as usize - entries.max(1).leading_zeros() as usize;
        u64::try_from(
            entries
                .min(11)
                .checked_mul(levels)?
                .checked_add(1)?
                .checked_mul(width)?
                .checked_mul(2)?,
        )
        .ok()
    }

    pub fn resolve_parts(
        &self,
        partition_id: PartitionId,
        kind_id: KindId,
        client_key: &ClientKey,
        source: BorrowedEntityRef<'_>,
        target: BorrowedEntityRef<'_>,
    ) -> Option<RelationId> {
        let exact = BorrowedExactRelationKey(partition_id, kind_id, client_key, source, target);
        if let Some(relation_id) = self
            .by_exact_reference
            .get(&exact as &dyn ExactRelationLookupKey)
        {
            return Some(*relation_id);
        }
        let stable = BorrowedStableRelationKey(
            partition_id,
            kind_id,
            client_key.as_raw_str()?,
            source.as_stable()?,
            target.as_stable()?,
        );
        self.by_stable_reference
            .get(&stable as &dyn StableRelationLookupKey)
            .copied()
    }
}
