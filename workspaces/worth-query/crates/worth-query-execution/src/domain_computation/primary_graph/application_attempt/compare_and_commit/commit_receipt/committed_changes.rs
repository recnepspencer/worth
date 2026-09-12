use std::sync::Arc;

use worth_relational::facade::{
    history::{CanonicalCommitEnvelope, RelationalCommitReceipt},
    identity::EntityId,
    lineage::LineageEventRecord,
    publication::RecordStructuralChange,
    transactions::{CommitResult, RecordRef},
};

/// Immutable structural and lineage observations from one authoritative commit.
///
/// The canonical artifact stays private. This view discloses record identities
/// and change posture, never field contents or execution authority.
#[derive(Clone, Eq, PartialEq)]
pub struct WorthQueryApplicationCommittedChanges {
    envelope: Arc<CanonicalCommitEnvelope>,
}

impl WorthQueryApplicationCommittedChanges {
    pub(in crate::domain_computation::primary_graph) fn from_commit(commit: &CommitResult) -> Self {
        Self {
            envelope: Arc::clone(&commit.publication().envelope),
        }
    }

    pub fn commit_reference(&self) -> &RelationalCommitReceipt {
        &self.envelope.commit
    }

    /// Iterates the commit's entity changes without copying its patch payloads.
    pub fn entity_changes(&self) -> impl Iterator<Item = (EntityId, RecordStructuralChange)> + '_ {
        self.envelope
            .patch
            .authoritative_record_patches
            .iter()
            .filter_map(|patch| match patch.target {
                RecordRef::Entity(entity) => Some((entity, patch.structural_change)),
                RecordRef::Relation(_) => None,
            })
    }

    pub fn lineage_events(&self) -> &[LineageEventRecord] {
        self.envelope.lineage_events()
    }
}

impl std::fmt::Debug for WorthQueryApplicationCommittedChanges {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryApplicationCommittedChanges")
            .field("commit", self.commit_reference())
            .field("entity_change_count", &self.entity_changes().count())
            .field("lineage_event_count", &self.lineage_events().len())
            .finish()
    }
}
