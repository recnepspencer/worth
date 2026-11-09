//! Selection effects must equal the actual retained producer-owned versions.
use super::*;
use crate::{
    identity::data::{EntityId, RelationId, VersionId},
    publication::data::{AuthoredRetirementRefusal as Cause, PublicationError},
    storage::overlay::{
        current_entity_partition_source_for_slot, current_relation_partition_source_for_slot,
        CurrentPartitionAccess, OverlayStateView,
    },
    transactions::data::RecordRef,
    validation::engine::state_view::InvariantStateView,
};

pub(super) fn validate(
    selection: &RelationalHistoricalOutputSelection,
    basis: &AdmittedRelationalBranchBasis,
    working: &WorkingState,
    version: VersionId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    let original = &selection.publication.origin;
    let selected_root = &selection.selected.inner.root;
    let before = InvariantStateView::new(selected_root.as_ref(), original.commit.version_id);
    let overlay = OverlayStateView::new(basis.inner.root.as_ref(), working);
    let after = InvariantStateView::new(&overlay, version);
    let source = original.meaning.source_scope;
    if after.entity_metadata(source).is_none() {
        return Err(refuse(original.commit.commit_id, source));
    }
    for member in original
        .members
        .iter()
        .filter(|m| m.posture != crate::mvcc::RequiredOutputRolePosture::Retire)
    {
        crate::execution::request_stage::checkpoint(context, 1).map_err(origin_request)?;
        let old = before
            .entity_metadata(member.id)
            .ok_or_else(|| refuse(original.commit.commit_id, member.id))?;
        let now = after
            .entity_metadata(member.id)
            .ok_or_else(|| refuse(original.commit.commit_id, member.id))?;
        let payload = before.entity_aspect_state(member.id);
        crate::execution::request_stage::checkpoint(
            context,
            payload.map_or(0, |p| p.owned_allocation_capacity_bytes()) as u64,
        )
        .map_err(origin_request)?;
        if old.kind_id != member.kind
            || now.kind_id != member.kind
            || payload != after.entity_aspect_state(member.id)
            || current_entity_partition_source_for_slot(
                &overlay,
                member.id.partition_id,
                member.id.slot_index(),
            )
            .and_then(|(_, partition)| {
                partition
                    .entity_arena()
                    .current_metadata_at(member.id.slot_index())
            })
            .is_none_or(|m| m.effective_at != version)
        {
            return Err(refuse(original.commit.commit_id, member.id));
        }
    }
    for edge in &original.relations {
        crate::execution::request_stage::checkpoint(context, 1).map_err(origin_request)?;
        let refusal = || {
            TransactionCommitError::publication(PublicationError::authored_retirement(
                Cause::HistoricalSelectionNotOwned {
                    publication: original.commit.commit_id,
                },
                RecordRef::Relation(edge.id),
            ))
        };
        let old = before.relation_metadata(edge.id).ok_or_else(refusal)?;
        let now = after.relation_metadata(edge.id).ok_or_else(refusal)?;
        let payload = before.relation_aspect_state(edge.id);
        crate::execution::request_stage::checkpoint(
            context,
            payload.map_or(0, |p| p.owned_allocation_capacity_bytes()) as u64,
        )
        .map_err(origin_request)?;
        if (old.kind_id, old.source, old.target) != (edge.kind, edge.source, edge.target)
            || (now.kind_id, now.source, now.target) != (edge.kind, edge.source, edge.target)
            || payload != after.relation_aspect_state(edge.id)
            || current_relation_partition_source_for_slot(
                &overlay,
                edge.id.partition_id,
                edge.id.slot_index(),
            )
            .and_then(|(_, partition)| {
                partition
                    .relation_arena()
                    .current_metadata_at(edge.id.slot_index())
            })
            .is_none_or(|m| m.effective_at != version)
        {
            return Err(refusal());
        }
    }
    Ok(())
}

pub(super) fn refuse(publication: CommitId, entity: EntityId) -> TransactionCommitError {
    TransactionCommitError::publication(PublicationError::authored_retirement(
        Cause::HistoricalSelectionNotOwned { publication },
        RecordRef::Entity(entity),
    ))
}
pub(super) fn entity_version(
    basis: &AdmittedRelationalBranchBasis,
    id: EntityId,
) -> Option<VersionId> {
    basis
        .inner
        .root
        .get_current_partition(id.partition_id)?
        .entity_arena()
        .current_metadata_at(id.slot_index())
        .map(|row| row.effective_at)
}
pub(super) fn relation_version(
    basis: &AdmittedRelationalBranchBasis,
    id: RelationId,
) -> Option<VersionId> {
    basis
        .inner
        .root
        .get_current_partition(id.partition_id)?
        .relation_arena()
        .current_metadata_at(id.slot_index())
        .map(|row| row.effective_at)
}
