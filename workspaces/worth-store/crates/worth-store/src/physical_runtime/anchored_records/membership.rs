//! The verified member table of the value each anchor currently selects.
//!
//! Every member read and every carried member is checked against the value
//! its anchor selects in the caller's protected root. Verifying that value
//! reads the whole anchored record, so the verified table is kept per anchor
//! and reused while the anchor still selects the same record with the same
//! digest: a member check then costs O(log members). The table is derived
//! from the durable record alone and is replaced, never patched, when the
//! anchor moves. At most one entry is kept per anchor-table slot.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_store_physical_format::{PersistedRecordIdentity, RecordAnchor, RecordAnchorName};

use super::frame::AnchoredRecordHeader;
use super::verified_value::{read_verified, PayloadRetention};
use super::{
    AnchoredRecordMemberLocator, AnchoredRecordMembership, AnchoredRecordName,
    AnchoredRecordReadFailure, AnchoredRecordVersion, AnchoredRecords,
};
use crate::physical_runtime::{PhysicalRecordId, PhysicalRecordReader};

/// The record an anchor selects, as the membership key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SelectedRecord {
    pub(super) record: PersistedRecordIdentity,
    pub(super) record_sha256: [u8; 32],
    pub(super) generation: u64,
}

impl SelectedRecord {
    pub(super) const fn of(anchor: RecordAnchor) -> Self {
        Self {
            record: anchor.record(),
            record_sha256: anchor.record_payload_sha256(),
            generation: anchor.generation(),
        }
    }
}

/// The verified members of one durable anchored value.
#[derive(Debug)]
pub(super) struct ValueMembership {
    pub(super) selected: SelectedRecord,
    pub(super) header: AnchoredRecordHeader,
    /// In the publisher's order.
    pub(super) members: Vec<AnchoredRecordMemberLocator>,
    listed: BTreeSet<AnchoredRecordMemberLocator>,
}

impl ValueMembership {
    pub(super) fn new(
        selected: SelectedRecord,
        header: AnchoredRecordHeader,
        members: Vec<AnchoredRecordMemberLocator>,
    ) -> Self {
        let listed = members.iter().copied().collect();
        Self {
            selected,
            header,
            members,
            listed,
        }
    }

    /// The version and members this value is reported as.
    pub(super) fn described(&self, name: AnchoredRecordName) -> AnchoredRecordMembership {
        AnchoredRecordMembership {
            version: AnchoredRecordVersion {
                name,
                generation: self.selected.generation,
                record: PhysicalRecordId::from_persisted(self.selected.record),
                payload_sha256: self.header.payload_sha256,
                payload_bytes: self.header.payload_bytes,
            },
            members: self.members.clone(),
        }
    }

    /// Whether this value lists `locator` as one of its members.
    pub(super) fn lists(&self, locator: &AnchoredRecordMemberLocator) -> bool {
        self.listed.contains(locator)
    }

    /// Whether this value holds exactly `payload_sha256` and these member
    /// contents, in order; record placement is not compared.
    pub(super) fn holds_content(
        &self,
        payload_sha256: [u8; 32],
        payload_bytes: u64,
        members: impl ExactSizeIterator<Item = ([u8; 32], u64)>,
    ) -> bool {
        self.header.payload_sha256 == payload_sha256
            && self.header.payload_bytes == payload_bytes
            && self.members.len() == members.len()
            && self
                .members
                .iter()
                .zip(members)
                .all(|(listed, (sha256, length))| {
                    listed.sha256() == sha256 && listed.length() == length
                })
    }
}

/// Per anchor, the membership of the last value verified or published.
#[derive(Debug, Default)]
pub(in crate::physical_runtime) struct AnchoredMembershipCache {
    values: BTreeMap<RecordAnchorName, Arc<ValueMembership>>,
}

impl AnchoredRecords<'_> {
    /// The verified membership of the value `anchor` selects in `reader`'s
    /// root, from the cache when it still describes that exact record.
    pub(super) fn membership(
        &self,
        reader: &PhysicalRecordReader,
        name: AnchoredRecordName,
        anchor: RecordAnchor,
    ) -> Result<Arc<ValueMembership>, AnchoredRecordReadFailure> {
        let selected = SelectedRecord::of(anchor);
        if let Some(known) = self.cached_membership(name, selected) {
            return Ok(known);
        }
        let verified = read_verified(reader, name, anchor, PayloadRetention::DigestOnly)?;
        Ok(self.remember(
            name,
            ValueMembership::new(selected, verified.header, verified.members),
        ))
    }

    /// The cached membership of `selected`, if the cache holds exactly it.
    fn cached_membership(
        &self,
        name: AnchoredRecordName,
        selected: SelectedRecord,
    ) -> Option<Arc<ValueMembership>> {
        let cache = self.runtime.anchored_membership_cache();
        cache
            .values
            .get(&name.anchor())
            .filter(|known| known.selected == selected)
            .cloned()
    }

    /// Records a verified or just-published membership as the latest known
    /// value of `name`, unless a newer generation is already known.
    pub(super) fn remember(
        &self,
        name: AnchoredRecordName,
        membership: ValueMembership,
    ) -> Arc<ValueMembership> {
        let membership = Arc::new(membership);
        let mut cache = self.runtime.anchored_membership_cache();
        let newer_known = cache
            .values
            .get(&name.anchor())
            .is_some_and(|known| known.selected.generation > membership.selected.generation);
        if !newer_known {
            cache.values.insert(name.anchor(), membership.clone());
        }
        membership
    }
}
