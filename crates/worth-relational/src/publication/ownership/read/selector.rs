//! Resolve a selector from its actual slot lifecycle, never from missing metadata.
use super::*;
use crate::validation::engine::state_view::VisibleRelationMetadata;

pub(super) fn relation(
    view: &InvariantStateView<'_>,
    after: Option<&InvariantStateView<'_>>,
    id: RelationId,
) -> Result<VisibleRelationMetadata, TransactionCommitError> {
    let subject = RecordRef::Relation(id);
    let head = view
        .state()
        .get_current_partition(id.partition_id)
        .ok_or_else(|| damaged(&subject))?;
    let slot = head
        .relation_arena()
        .get(&id)
        .ok_or_else(|| damaged(&subject))?;
    // A historical selection may restore a genuinely retired original ID.
    // Only its actual applied image supplies routing metadata; a live slot
    // with lost metadata must still refuse as damaged.
    let restoring_retired = slot.lifecycle()
        == crate::storage::data::RecordLifecycleState::DeletedRetained
        && after
            .and_then(|image| image.relation_metadata(id))
            .is_some();
    if slot.is_materialization_unavailable() || restoring_retired {
        let restored = after
            .and_then(|after| after.relation_metadata(id))
            .ok_or_else(|| missing_restoration(subject.clone()))?;
        if slot.kind_id() != Some(restored.kind_id) {
            return Err(missing_restoration(subject));
        }
        return Ok(restored);
    }
    view.relation_metadata(id).ok_or_else(|| damaged(&subject))
}

pub(super) fn entity_kind(
    view: &InvariantStateView<'_>,
    after: Option<&InvariantStateView<'_>>,
    id: EntityId,
) -> Result<Option<KindId>, TransactionCommitError> {
    let subject = RecordRef::Entity(id);
    let head = view
        .state()
        .get_current_partition(id.partition_id)
        .ok_or_else(|| damaged(&subject))?;
    let slot = head
        .entity_arena()
        .get(&id)
        .ok_or_else(|| damaged(&subject))?;
    if slot.is_materialization_unavailable() {
        let restored = after
            .and_then(|after| after.entity_metadata(id))
            .ok_or_else(|| missing_restoration(subject.clone()))?;
        if slot.kind_id() != Some(restored.kind_id) {
            return Err(missing_restoration(subject));
        }
        return Ok(Some(restored.kind_id));
    }
    if !slot.is_live() {
        return Ok(None); // Retained authored anchors remain valid selectors.
    }
    view.entity_metadata(id)
        .map(|row| Some(row.kind_id))
        .ok_or_else(|| damaged(&subject))
}

fn missing_restoration(record: RecordRef) -> TransactionCommitError {
    use crate::transactions::data::{
        CommitConflict, ConflictClass, MutationStateInconsistencyEvidence,
    };
    let mut conflict = CommitConflict::new(ConflictClass::MutationStateInconsistency {
        detail: "unavailable custody selector requires its restored candidate metadata".into(),
        evidence: MutationStateInconsistencyEvidence::MaterializationTransition {
            record: record.clone(),
        },
    });
    conflict.context.affected_records.push(record);
    TransactionCommitError::conflict(conflict)
}

#[cfg(test)]
mod tests;
