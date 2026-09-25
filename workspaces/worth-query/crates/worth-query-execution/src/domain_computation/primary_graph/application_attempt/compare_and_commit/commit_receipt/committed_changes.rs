use std::{collections::BTreeMap, sync::Arc};

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::{
    history::{CanonicalCommitEnvelope, RelationalCommitReceipt},
    identity::EntityId,
    lineage::LineageEventRecord,
    publication::{PublishedAuthoritativeRecordPatch, RecordStructuralChange},
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

    /// Project one field across created entities without rescanning the whole
    /// commit for every candidate. Duplicate committed writes remain invalid.
    pub(in crate::domain_computation::primary_graph) fn created_entities_with_field_value(
        &self,
        locator: &AspectFieldLocator,
        expected: &AspectValue,
    ) -> Vec<EntityId> {
        let [_field] = locator.field_path().fields() else {
            return Vec::new();
        };
        let created = self
            .entity_changes()
            .filter_map(|(entity, change)| {
                (change == RecordStructuralChange::Created).then_some(entity)
            })
            .collect::<Vec<_>>();
        let mut observed = created
            .iter()
            .copied()
            .map(|entity| (entity, (None::<AspectValue>, false)))
            .collect::<BTreeMap<_, _>>();
        for patch in &self.envelope.patch.authoritative_record_patches {
            let RecordRef::Entity(entity) = patch.target else {
                continue;
            };
            let Some((committed, duplicate)) = observed.get_mut(&entity) else {
                continue;
            };
            for_each_committed_field_value(patch, locator, |value| {
                if committed.replace(value.clone()).is_some() {
                    *duplicate = true;
                }
            });
        }
        created
            .into_iter()
            .filter(|entity| {
                observed.get(entity).is_some_and(|(value, duplicate)| {
                    !duplicate && value.as_ref() == Some(expected)
                })
            })
            .collect()
    }

    fn committed_field_value(
        &self,
        entity: EntityId,
        locator: &AspectFieldLocator,
    ) -> Option<AspectValue> {
        let [_field] = locator.field_path().fields() else {
            return None;
        };
        let mut committed = None;
        for patch in &self.envelope.patch.authoritative_record_patches {
            if patch.target != RecordRef::Entity(entity) {
                continue;
            }
            let mut duplicate = false;
            for_each_committed_field_value(patch, locator, |value| {
                if committed.replace(value.clone()).is_some() {
                    duplicate = true;
                }
            });
            if duplicate {
                return None;
            }
        }
        committed
    }
}

fn for_each_committed_field_value(
    patch: &PublishedAuthoritativeRecordPatch,
    locator: &AspectFieldLocator,
    mut visit: impl FnMut(&AspectValue),
) {
    let [field] = locator.field_path().fields() else {
        return;
    };
    let aspect = locator.aspect().aspect_key();
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
        visit(value);
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
