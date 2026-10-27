use std::{num::NonZeroUsize, sync::Arc};

use worth_foundational::facade::AspectKey;

use crate::history::data::{BranchId, CommitId};
use crate::identity::data::VersionId;
use crate::publication::patch::data::RecordStructuralChange;
use crate::transactions::data::RecordRef;

use super::cursor::RetainedCanonicalRecordHistory;

/// Limits apply to canonical record visits and newly allocated page backing.
/// Already retained canonical envelopes remain under native retention custody.
#[derive(Clone, Copy, Debug)]
pub struct RelationalRecordHistoryPageLimits {
    pub maximum_entries: NonZeroUsize,
    pub maximum_source_visits: NonZeroUsize,
    pub maximum_owned_bytes: NonZeroUsize,
}

/// A read-only view of one genuine canonical record occurrence. No payload copy.
#[derive(Clone, Debug)]
pub struct RelationalRecordHistoryChange {
    pub(super) source: Arc<RetainedCanonicalRecordHistory>,
    pub(super) record_index: usize,
}

impl RelationalRecordHistoryChange {
    pub fn commit_id(&self) -> CommitId {
        self.source.envelope.commit.commit_id
    }
    pub fn version_id(&self) -> VersionId {
        self.source.envelope.commit.version_id
    }
    pub fn branch_id(&self) -> &BranchId {
        &self.source.envelope.commit.branch_id
    }
    pub fn record(&self) -> &RecordRef {
        &self.patch().target
    }
    pub fn structural_change(&self) -> RecordStructuralChange {
        self.patch().structural_change
    }
    pub fn contains_opaque_aspect(&self) -> bool {
        self.patch().contains_opaque_aspect
    }

    /// Selected keys only, O(selected aspects × log(canonical operations)).
    pub fn changed_aspects(&self) -> impl Iterator<Item = &AspectKey> {
        self.source
            .aspects
            .iter()
            .filter(|key| self.patch().authoritative_patch.changes_aspect(key))
    }

    fn patch(&self) -> &crate::publication::patch::data::PublishedAuthoritativeRecordPatch {
        &self.source.envelope.patch.authoritative_record_patches[self.record_index]
    }
}

/// One finite page inside a single canonical commit. An empty page may be incomplete.
#[derive(Debug)]
pub struct RelationalRecordHistoryPage {
    pub(super) slots: worth_execution::ExecutionArray<Option<RelationalRecordHistoryChange>>,
    pub(super) entry_count: usize,
    pub(super) source_visits: usize,
    pub(super) next_record: usize,
    pub(super) complete: bool,
}

/// Staged native work. Dropping this proof leaves its cursor unchanged.
/// Source identity and the accepted position cannot be reconstructed by callers.
#[derive(Debug)]
pub struct RelationalPreparedRecordHistoryPage<'request, 'authority> {
    pub(super) source: Arc<RetainedCanonicalRecordHistory>,
    pub(super) execution: worth_execution::ExecutionRequest<'request, 'authority>,
    pub(super) expected_record: usize,
    pub(super) page: RelationalRecordHistoryPage,
}

impl RelationalPreparedRecordHistoryPage<'_, '_> {
    pub fn page(&self) -> &RelationalRecordHistoryPage {
        &self.page
    }
}

impl RelationalRecordHistoryPage {
    pub fn entries(&self) -> impl ExactSizeIterator<Item = &RelationalRecordHistoryChange> {
        self.slots[..self.entry_count]
            .iter()
            .map(|entry| entry.as_ref().expect("populated prefix"))
    }
    pub const fn source_visits(&self) -> usize {
        self.source_visits
    }
    pub const fn is_complete(&self) -> bool {
        self.complete
    }
    pub fn owned_backing_bytes(&self) -> usize {
        self.slots.len() * std::mem::size_of::<Option<RelationalRecordHistoryChange>>()
    }
}
