//! Root-local routing metadata; no full record or historical projection.
use super::{root_view::IndexRootView, work::MaintenanceWork};
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;
use crate::storage::data::RecordLifecycleState;
#[derive(Clone, Copy)]
pub(super) struct RelationMetadata {
    pub(super) id: RelationId,
    pub(super) kind: KindId,
    pub(super) source: EntityId,
    pub(super) target: EntityId,
}
pub(super) fn entity_kind(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    view: Option<&IndexRootView>,
    id: EntityId,
    work: &mut MaintenanceWork,
) -> Result<Option<KindId>, Denial> {
    let Some(view) = view else {
        return Ok(None);
    };
    work.read()?;
    let Some(head) = view.head(execution, id.partition_id, work)? else {
        return Ok(None);
    };
    let Some(slot) = head.entity_arena().get_slot(id.slot_index()) else {
        return Ok(None);
    };
    if slot.lifecycle() != RecordLifecycleState::Live
        || slot
            .retired_at()
            .is_some_and(|version| version <= view.version_id())
        || (!id.generation.is_zero() && slot.generation() != id.generation_value())
    {
        return Ok(None);
    }
    let Some(kind) = slot.kind_id() else {
        return Ok(None);
    };
    work.lookup(
        view.root().schema_authority().registry().entity_kinds.len(),
        1,
    )?;
    Ok(view
        .root()
        .schema_authority()
        .registry()
        .entity_kinds
        .contains_key(&kind)
        .then_some(kind))
}
pub(super) fn relation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    view: Option<&IndexRootView>,
    id: RelationId,
    work: &mut MaintenanceWork,
) -> Result<Option<RelationMetadata>, Denial> {
    let Some(view) = view else {
        return Ok(None);
    };
    work.read()?;
    let Some(head) = view.head(execution, id.partition_id, work)? else {
        return Ok(None);
    };
    let Some(slot) = head.relation_arena().get_slot(id.slot_index()) else {
        return Ok(None);
    };
    if !matches!(
        slot.lifecycle(),
        RecordLifecycleState::Live | RecordLifecycleState::RetainedDanglingForAudit
    ) || (slot.lifecycle() == RecordLifecycleState::Live
        && slot
            .retired_at()
            .is_some_and(|version| version <= view.version_id()))
        || (!id.generation.is_zero() && slot.generation() != id.generation_value())
    {
        return Ok(None);
    }
    let Some(kind) = slot.kind_id() else {
        return Ok(None);
    };
    work.lookup(
        view.root()
            .schema_authority()
            .registry()
            .relation_kinds
            .len(),
        1,
    )?;
    if !view
        .root()
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
        .map(|ends| RelationMetadata {
            id: RelationId::new(id.partition_id, id.local_slot_value(), slot.generation()),
            kind,
            source: ends.source,
            target: ends.target,
        }))
}
