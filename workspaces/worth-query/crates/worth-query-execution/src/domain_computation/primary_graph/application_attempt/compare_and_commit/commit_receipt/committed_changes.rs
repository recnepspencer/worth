use std::sync::Arc;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::{
    history::{CanonicalCommitEnvelope, RelationalCommitReceipt},
    identity::EntityId,
    lineage::LineageEventRecord,
    publication::RecordStructuralChange,
    transactions::{CommitResult, RecordRef},
};

/// Immutable structural and lineage observations from one authoritative commit.
///
/// The canonical artifact stays private. Public methods disclose record
/// identities and change posture; only crate-private lifecycle replay reads
/// exact committed fields, without granting execution authority.
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

    pub(in crate::domain_computation::primary_graph) fn committed_field_values(
        &self,
        entity: EntityId,
        fields: &[&AspectFieldLocator],
    ) -> Option<Vec<AspectValue>> {
        fields
            .iter()
            .map(|locator| self.committed_field_value(entity, locator))
            .collect()
    }

    fn committed_field_value(
        &self,
        entity: EntityId,
        locator: &AspectFieldLocator,
    ) -> Option<AspectValue> {
        let [field] = locator.field_path().fields() else {
            return None;
        };
        let aspect = locator.aspect().aspect_key();
        let mut committed = None;
        for patch in &self.envelope.patch.authoritative_record_patches {
            if patch.target != RecordRef::Entity(entity) {
                continue;
            }
            let whole = patch
                .authoritative_patch
                .struct_set_for(aspect)
                .and_then(|value| value.get(field));
            for value in whole.into_iter().chain(
                patch
                    .authoritative_patch
                    .field_sets_for(aspect)
                    .filter(|set| &set.field == field)
                    .map(|set| &set.value),
            ) {
                if committed.replace(value.clone()).is_some() {
                    return None;
                }
            }
        }
        committed
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
