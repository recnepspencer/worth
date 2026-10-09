//! Read only routing metadata from the selected immutable visibility basis.
use super::{reads, work::MaintenanceWork};
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;
use crate::runtime::VisibilityProjectionView;
use crate::storage::data::RecordLifecycleState;

#[derive(Clone, Copy)]
pub(super) struct RelationMetadata {
    pub(super) id: RelationId,
    pub(super) kind: KindId,
    pub(super) source: EntityId,
    pub(super) target: EntityId,
}

pub(super) fn entity_kind(
    view: Option<&VisibilityProjectionView<'_>>,
    id: EntityId,
    work: &mut MaintenanceWork,
) -> Result<Option<KindId>, Denial> {
    let Some(view) = view else { return Ok(None) };
    if !view.is_exact_basis() {
        return Ok(reads::entity(Some(view), id, work)?.map(|r| r.kind.kind_id));
    }
    work.read()?;
    work.prepare(view.exact_entity_state_read_work_bound().unwrap(), 0)?;
    Ok(view
        .with_exact_entity_state(id, |metadata, _| metadata.kind_id)
        .expect("exact basis checked"))
}

pub(super) fn relation(
    view: Option<&VisibilityProjectionView<'_>>,
    id: RelationId,
    work: &mut MaintenanceWork,
) -> Result<Option<RelationMetadata>, Denial> {
    let Some(view) = view else { return Ok(None) };
    if !view.is_exact_basis() {
        return Ok(
            reads::relation(Some(view), id, work)?.map(|r| RelationMetadata {
                id: r.relation_id,
                kind: r.kind.kind_id,
                source: r.source,
                target: r.target,
            }),
        );
    }
    work.read()?;
    work.prepare(view.exact_relation_metadata_read_work_bound().unwrap(), 0)?;
    let root = view.selected_root().expect("exact basis checked");
    let Some(partition) = root.partition_state(id.partition_id) else {
        return Ok(None);
    };
    let Some(slot) = partition.relation_arena.get_slot(id.slot_index()) else {
        return Ok(None);
    };
    // Unlike the public live-only metadata probe, maintenance also observes
    // retained dangling relations so deleting an endpoint removes derived rows.
    if !matches!(
        slot.lifecycle(),
        RecordLifecycleState::Live | RecordLifecycleState::RetainedDanglingForAudit
    ) || (!id.generation.is_zero() && slot.generation() != id.generation_value())
    {
        return Ok(None);
    }
    let Some(kind) = slot.kind_id() else {
        return Ok(None);
    };
    if !root
        .schema_authority()
        .registry()
        .relation_kinds
        .contains_key(&kind)
    {
        return Ok(None);
    }
    Ok(slot
        .extra()
        .endpoints
        .as_ref()
        .map(|endpoints| RelationMetadata {
            id: RelationId::new(id.partition_id, id.local_slot_value(), slot.generation()),
            kind,
            source: endpoints.source,
            target: endpoints.target,
        }))
}
