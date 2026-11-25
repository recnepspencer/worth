//! The Branch change carries its original canonical artifact.
use crate::history::data::{CommitId, RelationalDescriptiveTouch, SharedCanonicalCommitEnvelope};
use crate::identity::data::PartitionId;
use crate::storage::substrate::maintenance::{MaintenanceRoute, PartitionChange};
use crate::symbols::data::{Symbol, SymbolTableSnapshot};
use crate::transactions::data::primitives::RecordRef;
use std::collections::{BTreeMap, BTreeSet};

/// Built once at canonical root preparation, shared by Arc through publication.
/// The canonical envelope owns fields, postings, endpoint and symbol meaning;
/// this projection carries its exact artifact rather than another truth lane.
#[derive(Debug)]
pub struct CommitChange {
    pub(crate) envelope: SharedCanonicalCommitEnvelope,
    pub(crate) partitions: BTreeMap<PartitionId, PartitionChange>,
    pub(crate) schema_changed: bool,
    pub(crate) required_symbols: BTreeSet<Symbol>,
    pub(crate) created: Vec<RecordRef>,
    pub(crate) retired: Vec<RecordRef>,
    pub(crate) routes: BTreeSet<MaintenanceRoute>,
}
impl CommitChange {
    pub fn routes(&self) -> &BTreeSet<MaintenanceRoute> {
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
