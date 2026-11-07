//! A single immutable projection of canonical commit truth for SDK maintainers.
use crate::history::data::{CanonicalCommitEnvelope, CommitId, RelationalDescriptiveTouch};
use crate::identity::data::{KindId, PartitionId, VersionId};
use crate::storage::data::RecordLifecycleState;
use crate::storage::overlay::PartitionMutationJournal;
use crate::symbols::data::{Symbol, SymbolTableSnapshot};
use crate::transactions::data::RecordRef;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordChangeState {
    pub record: RecordRef,
    pub kind: KindId,
    pub lifecycle: RecordLifecycleState,
    pub retired_at: Option<VersionId>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangedRecord {
    pub before: Option<RecordChangeState>,
    pub after: Option<RecordChangeState>,
}

/// Sealed storage-owner journal; access never enumerates the selected partition.
#[derive(Debug)]
pub struct PartitionChange {
    pub(crate) journal: PartitionMutationJournal,
    pub(crate) records: Vec<ChangedRecord>,
}
impl PartitionChange {
    pub fn records(&self) -> &[ChangedRecord] {
        &self.records
    }
    pub fn entity_slots(&self) -> &BTreeSet<usize> {
        &self.journal.entity_slots
    }
    pub fn relation_slots(&self) -> &BTreeSet<usize> {
        &self.journal.relation_slots
    }
    pub fn outgoing_slots(&self) -> &BTreeSet<usize> {
        &self.journal.adjacency_slots
    }
    pub fn incoming_slots(&self) -> &BTreeSet<usize> {
        &self.journal.reverse_adjacency_slots
    }
}

/// Built once at canonical root preparation, shared by Arc through publication.
/// The canonical envelope owns fields, postings, endpoint and symbol meaning;
/// this projection carries its exact artifact rather than another truth lane.
#[derive(Debug)]
pub struct CommitChange {
    pub(crate) envelope: Arc<CanonicalCommitEnvelope>,
    pub(crate) partitions: BTreeMap<PartitionId, PartitionChange>,
    pub(crate) schema_changed: bool,
    pub(crate) required_symbols: BTreeSet<Symbol>,
    pub(crate) created: Vec<RecordRef>,
    pub(crate) retired: Vec<RecordRef>,
    pub(crate) routes: BTreeSet<super::MaintenanceRoute>,
}
impl CommitChange {
    pub fn routes(&self) -> &BTreeSet<super::MaintenanceRoute> {
        &self.routes
    }
    pub fn commit_id(&self) -> CommitId {
        self.envelope.commit.commit_id
    }
    pub fn partition(&self, id: PartitionId) -> Option<&PartitionChange> {
        self.partitions.get(&id)
    }
    pub fn partitions(&self) -> impl Iterator<Item = (PartitionId, &PartitionChange)> {
        self.partitions.iter().map(|(&id, change)| (id, change))
    }
    /// Field locators, old/new posting keys and relation endpoint coordinates
    /// are the canonical owner's sealed touch evidence, with no re-projection.
    pub fn touches(&self) -> Option<&[RelationalDescriptiveTouch]> {
        self.envelope.descriptive_touches().exact_touches()
    }
    pub fn schema_changed(&self) -> bool {
        self.schema_changed
    }
    pub fn symbol_texts(&self) -> &SymbolTableSnapshot {
        &self.envelope.symbol_texts
    }
    pub fn required_commit_symbols(&self) -> &BTreeSet<Symbol> {
        &self.required_symbols
    }
    pub fn created(&self) -> &[RecordRef] {
        &self.created
    }
    pub fn retired(&self) -> &[RecordRef] {
        &self.retired
    }
}
