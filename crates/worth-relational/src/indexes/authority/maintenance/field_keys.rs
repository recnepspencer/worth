//! Selected field projection borrows the exact root; unrelated aspects are
//! never materialized merely to create one canonical index key.
use super::work::MaintenanceWork;
use crate::identity::data::{EntityId, RelationId};
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;
use crate::indexes::projected_field_values::{
    entity_index_projection_scope, relation_index_projection_scope, IndexProjectionSource,
};
use crate::runtime::VisibilityProjectionView;
use crate::schema::data::LoweredAspectContractPlan;
use crate::storage::data::{AuthoritativeFieldComparisonKey as Key, RecordLifecycleState};
use crate::visibility::materialization::read_records::authoritative_state_query_locus_value;
use worth_foundational::facade::{
    AspectFieldLocator, AuthoritativeRecordAspectState, ContractValidatedAspectValueView,
};

pub(super) fn entity(
    view: Option<&VisibilityProjectionView<'_>>,
    id: EntityId,
    locator: &AspectFieldLocator,
    work: &mut MaintenanceWork,
) -> Result<Option<(Key, EntityId)>, Denial> {
    let Some(view) = view else { return Ok(None) };
    work.prepare(1, 0)?;
    let root = view.selected_root().ok_or(Denial::SnapshotUnavailable)?;
    work.prepare(33, 0)?;
    let Some(partition) = root.partition_state(id.partition_id) else {
        return Ok(None);
    };
    let Some(slot) = partition.entity_arena.get_slot(id.slot_index()) else {
        return Ok(None);
    };
    if slot.lifecycle() != RecordLifecycleState::Live
        || (!id.generation.is_zero() && slot.generation() != id.generation_value())
    {
        return Ok(None);
    }
    let Some(kind) = slot.kind_id() else {
        return Ok(None);
    };
    let source = IndexProjectionSource::selected(view);
    let Some(catalog) = source.aspect_plans() else {
        return Ok(None);
    };
    work.lookup(catalog.entity_plans.len(), 1)?;
    let Some(plan) = source.entity_aspect_plan(kind) else {
        return Ok(None);
    };
    prepare_scope(plan, locator, work)?;
    if entity_index_projection_scope(plan, locator).is_none() {
        return Ok(None);
    }
    value(
        slot.extra().authoritative_aspect_state.as_ref(),
        locator,
        work,
    )
    .map(|key| key.map(|key| (key, id)))
}

pub(super) fn relation(
    view: Option<&VisibilityProjectionView<'_>>,
    id: RelationId,
    locator: &AspectFieldLocator,
    work: &mut MaintenanceWork,
) -> Result<Option<(Key, RelationId)>, Denial> {
    let Some(view) = view else { return Ok(None) };
    work.prepare(1, 0)?;
    let root = view.selected_root().ok_or(Denial::SnapshotUnavailable)?;
    work.prepare(33, 0)?;
    let Some(partition) = root.partition_state(id.partition_id) else {
        return Ok(None);
    };
    let Some(slot) = partition.relation_arena.get_slot(id.slot_index()) else {
        return Ok(None);
    };
    if !matches!(
        slot.lifecycle(),
        RecordLifecycleState::Live | RecordLifecycleState::RetainedDanglingForAudit
    ) || (!id.generation.is_zero() && slot.generation() != id.generation_value())
        || slot.extra().endpoints.is_none()
    {
        return Ok(None);
    }
    let Some(kind) = slot.kind_id() else {
        return Ok(None);
    };
    let source = IndexProjectionSource::selected(view);
    let Some(catalog) = source.aspect_plans() else {
        return Ok(None);
    };
    work.lookup(catalog.relation_plans.len(), 1)?;
    let Some(plan) = source.relation_aspect_plan(kind) else {
        return Ok(None);
    };
    prepare_scope(plan, locator, work)?;
    if relation_index_projection_scope(plan, locator).is_none() {
        return Ok(None);
    }
    value(
        slot.extra().authoritative_aspect_state.as_ref(),
        locator,
        work,
    )
    .map(|key| key.map(|key| (key, id)))
}

fn prepare_scope(
    plan: &LoweredAspectContractPlan,
    locator: &AspectFieldLocator,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    let width = locator
        .aspect()
        .aspect_key()
        .as_str()
        .len()
        .checked_add(1)
        .ok_or(Denial::WorkBudgetExceeded)?;
    let visits = plan
        .executable_bindings
        .len()
        .checked_mul(width)
        .ok_or(Denial::WorkBudgetExceeded)?;
    // Fund the inventory walk before reading binding declarations.
    work.prepare(visits as u64, 0)?;
    let field_width = locator
        .field_path()
        .fields()
        .first()
        .map_or(1, |field| field.as_str().len() + 1);
    for binding in &plan.executable_bindings {
        work.prepare(field_width as u64, 0)?;
        if let worth_foundational::AspectShape::Struct(shape) = binding.contract.shape() {
            let comparisons = shape
                .fields()
                .len()
                .checked_mul(field_width)
                .ok_or(Denial::WorkBudgetExceeded)?;
            work.prepare(comparisons as u64, 0)?;
        }
    }
    Ok(())
}

fn value(
    state: Option<&AuthoritativeRecordAspectState>,
    locator: &AspectFieldLocator,
    work: &mut MaintenanceWork,
) -> Result<Option<Key>, Denial> {
    let Some(state) = state else { return Ok(None) };
    let aspect = locator.aspect().aspect_key();
    work.lookup(state.aspects().len(), aspect.as_str().len() + 1)?;
    if let Some(value) = state.get(aspect) {
        if let ContractValidatedAspectValueView::Struct(value) = value.view() {
            let fields = value
                .fields()
                .size_hint()
                .1
                .ok_or(Denial::WorkBudgetExceeded)?;
            let width = locator
                .field_path()
                .fields()
                .first()
                .map_or(1, |field| field.as_str().len() + 1);
            work.lookup(fields, width)?;
        }
    }
    // The original projection remains the value authority. Its repeated
    // aspect navigation is separately prepaid.
    work.lookup(state.aspects().len(), aspect.as_str().len() + 1)?;
    let Some(value) = authoritative_state_query_locus_value(Some(state), locator) else {
        return Ok(None);
    };
    work.prepare(1, 0)?;
    let bytes = Key::required_encoded_capacity_bytes(value).ok_or(Denial::WorkBudgetExceeded)?;
    work.prepare(
        bytes.checked_add(1).ok_or(Denial::WorkBudgetExceeded)?,
        bytes,
    )?;
    Ok(Some(Key::from_aspect_value(value)))
}
