use super::{admission, Context, ObservationResult, State};
use crate::authorization::{
    RelationalAuthorizationBudgetedObservationStop, RelationalAuthorizationObservationAdmission,
};
use crate::capabilities::AspectPlanSource;
use crate::identity::data::{EntityId, KindId};
use crate::storage::data::RecordLifecycleState;
use worth_foundational::facade::{
    AspectFieldLocator, AspectValue, ContractValidatedAspectValueView,
};

pub(super) fn live<A: RelationalAuthorizationObservationAdmission>(
    context: &Context<'_, '_, '_>,
    entity: EntityId,
    kind: KindId,
    state: &mut State<'_, A>,
) -> ObservationResult<bool, A> {
    admission::prepare(state.admission, 1, 0)?;
    state.counters.entity_records_inspected += 1;
    Ok(context
        .view
        .with_exact_entity_state(entity, |metadata, _| {
            metadata.kind_id == kind && metadata.lifecycle == RecordLifecycleState::Live
        })
        .map_err(|_| RelationalAuthorizationBudgetedObservationStop::ExactBasisRequired)?
        .unwrap_or(false))
}

pub(super) fn observe<A: RelationalAuthorizationObservationAdmission>(
    context: &Context<'_, '_, '_>,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    state: &mut State<'_, A>,
) -> ObservationResult<Option<AspectValue>, A> {
    admission::prepare(state.admission, 1, 0)?;
    let Some(field) = locator.field_path().fields().first() else {
        return Ok(None);
    };
    let catalog = context.runtime.aspect_plan_catalog();
    admission::ordered_lookup(state.admission, catalog.entity_plans.len(), 1)?;
    let Some(plan) = catalog.entity_plans.get(&kind) else {
        return Ok(None);
    };
    let mut selected = None;
    for binding in &plan.executable_bindings {
        admission::prepare(
            state.admission,
            binding.aspect_key().as_str().len() as u64
                + locator.aspect().aspect_key().as_str().len() as u64
                + 1,
            0,
        )?;
        if binding.aspect_key() == locator.aspect().aspect_key() {
            selected = Some(binding);
            break;
        }
    }
    let Some(binding) = selected else {
        return Ok(None);
    };
    let crate::schema::data::AspectBinding::EntityField { field: target } = &binding.target else {
        return Ok(None);
    };
    admission::prepare(
        state.admission,
        target.as_str().len() as u64 + field.as_str().len() as u64 + 1,
        0,
    )?;
    let scalar = target == field
        && matches!(
            binding.contract.shape(),
            worth_foundational::AspectShape::Scalar(_)
        );
    let structured =
        if let worth_foundational::AspectShape::Struct(shape) = binding.contract.shape() {
            let mut declared = false;
            for declaration in shape.fields() {
                admission::prepare(
                    state.admission,
                    declaration.key().as_str().len() as u64 + field.as_str().len() as u64 + 1,
                    0,
                )?;
                if declaration.key() == field {
                    declared = true;
                    break;
                }
            }
            declared
        } else {
            false
        };
    if !scalar && !structured {
        return Ok(None);
    }
    admission::prepare(state.admission, 1, 0)?;
    let value = context
        .view
        .with_exact_entity_state(entity, |metadata, authoritative| {
            if metadata.kind_id != kind || metadata.lifecycle != RecordLifecycleState::Live {
                return Ok(None);
            }
            let Some(authoritative) = authoritative else {
                return Ok(None);
            };
            admission::ordered_lookup(
                state.admission,
                authoritative.aspects().len(),
                locator.aspect().aspect_key().as_str().len() + 1,
            )?;
            let Some(value) = authoritative.get(locator.aspect().aspect_key()) else {
                return Ok(None);
            };
            let value = match value.view() {
                ContractValidatedAspectValueView::Scalar(value) if scalar => Some(value),
                ContractValidatedAspectValueView::Struct(value) if structured => {
                    let count = value.fields().size_hint().1.ok_or(
                        RelationalAuthorizationBudgetedObservationStop::AccountingOverflow,
                    )?;
                    admission::ordered_lookup(state.admission, count, field.as_str().len() + 1)?;
                    value.get(field)
                }
                _ => None,
            };
            let Some(value) = value else {
                return Ok(None);
            };
            admission::prepare(
                state.admission,
                value.semantic_byte_width() as u64,
                value.owned_allocation_capacity_bytes() as u64,
            )?;
            Ok(Some(value.clone()))
        })
        .map_err(|_| RelationalAuthorizationBudgetedObservationStop::ExactBasisRequired)?;
    value.unwrap_or(Ok(None))
}
