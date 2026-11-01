use crate::identity::data::EntityId;
use crate::publication::patch::data::RecordStructuralChange;
use crate::schema::data::{AspectBinding, LoweredAspectContractBinding};
use worth_foundational::facade::AspectKey;
use worth_foundational::facade::{
    AspectShape, AuthoritativeRecordAspectState, ContractValidatedAspectValueView,
    StructAspectValue,
};

use super::data::{
    BindingEvaluationContext, CanonicalAspectDeltaEvidence, CanonicalDeltaError,
    LifecycleTransitionClass,
};
use super::lifecycle_transition_evidence::lifecycle_transition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum MaterializedAspectState<'a> {
    ScalarValue(Option<&'a worth_foundational::facade::AspectValue>),
    StructValue(Option<&'a StructAspectValue>),
    EndpointIdentity(Option<EntityId>),
    LifecycleTransition(LifecycleTransitionClass),
    StructuralChange(RecordStructuralChange),
}

pub(super) fn evaluate_authoritative_binding_delta(
    binding: &LoweredAspectContractBinding,
    context: BindingEvaluationContext<'_>,
) -> Result<(CanonicalAspectDeltaEvidence, bool), CanonicalDeltaError> {
    borrowed_authoritative_binding_delta(binding, context)
        .map(|(evidence, changed)| (evidence.into_owned(binding.aspect_key()), changed))
}

pub(super) fn borrowed_authoritative_binding_delta<'a>(
    binding: &LoweredAspectContractBinding,
    context: BindingEvaluationContext<'a>,
) -> Result<(super::BorrowedAspectDeltaEvidence<'a>, bool), CanonicalDeltaError> {
    let old_state = materialize_old_binding_state(binding, context)?;
    let new_state = materialize_new_binding_state(binding, context)?;
    binding_evidence_from_states(binding.aspect_key(), old_state, new_state)
}

fn materialize_old_binding_state<'a>(
    binding: &LoweredAspectContractBinding,
    context: BindingEvaluationContext<'a>,
) -> Result<MaterializedAspectState<'a>, CanonicalDeltaError> {
    materialize_binding_state(binding, context, MaterializationSide::Old)
}

fn materialize_new_binding_state<'a>(
    binding: &LoweredAspectContractBinding,
    context: BindingEvaluationContext<'a>,
) -> Result<MaterializedAspectState<'a>, CanonicalDeltaError> {
    materialize_binding_state(binding, context, MaterializationSide::New)
}

#[derive(Debug, Clone, Copy)]
enum MaterializationSide {
    Old,
    New,
}

fn materialize_binding_state<'a>(
    binding: &LoweredAspectContractBinding,
    context: BindingEvaluationContext<'a>,
    side: MaterializationSide,
) -> Result<MaterializedAspectState<'a>, CanonicalDeltaError> {
    match &binding.target {
        AspectBinding::EntityField { field } => {
            materialize_entity_field_binding_state(binding, context, side, field)
        }
        AspectBinding::RelationField { .. } => {
            materialize_relation_field_binding_state(binding, context, side)
        }
        AspectBinding::RelationSourceEndpoint => {
            materialize_relation_source_endpoint_binding_state(binding, context, side)
        }
        AspectBinding::RelationTargetEndpoint => {
            materialize_relation_target_endpoint_binding_state(binding, context, side)
        }
        AspectBinding::LifecycleTransition => Ok(MaterializedAspectState::LifecycleTransition(
            lifecycle_transition(context.structural_change()),
        )),
        AspectBinding::StructuralRegion
        | AspectBinding::StructuralPartition
        | AspectBinding::StructuralFacet => Ok(MaterializedAspectState::StructuralChange(
            context.structural_change(),
        )),
        _ => Err(invalid_binding_context(
            binding.aspect_key(),
            "unsupported authoritative aspect binding",
        )),
    }
}

fn materialize_entity_field_binding_state<'a>(
    binding: &LoweredAspectContractBinding,
    context: BindingEvaluationContext<'a>,
    side: MaterializationSide,
    field: &worth_foundational::facade::FieldKey,
) -> Result<MaterializedAspectState<'a>, CanonicalDeltaError> {
    if !matches!(context, BindingEvaluationContext::Entity { .. }) {
        return Err(invalid_binding_context(
            binding.aspect_key(),
            "entity field aspect cannot be evaluated against relation context",
        ));
    }
    match binding.contract.shape() {
        AspectShape::Scalar(_) => Ok(MaterializedAspectState::ScalarValue(
            materialize_authoritative_scalar_state(
                binding.aspect_key(),
                authoritative_state_for_side(context, side),
            )?,
        )),
        AspectShape::Struct(_) => Ok(MaterializedAspectState::StructValue(
            materialize_authoritative_struct_state(
                binding.aspect_key(),
                authoritative_state_for_side(context, side),
            )?,
        )),
        _ => Err(entity_field_binding_requires_authoritative_patch(
            binding.aspect_key(),
            field,
        )),
    }
}

fn materialize_relation_field_binding_state<'a>(
    binding: &LoweredAspectContractBinding,
    context: BindingEvaluationContext<'a>,
    side: MaterializationSide,
) -> Result<MaterializedAspectState<'a>, CanonicalDeltaError> {
    if !matches!(context, BindingEvaluationContext::Relation { .. }) {
        return Err(invalid_binding_context(
            binding.aspect_key(),
            "relation field aspect cannot be evaluated against entity context",
        ));
    }
    match binding.contract.shape() {
        AspectShape::Scalar(_) => Ok(MaterializedAspectState::ScalarValue(
            materialize_authoritative_scalar_state(
                binding.aspect_key(),
                authoritative_state_for_side(context, side),
            )?,
        )),
        AspectShape::Struct(_) => Ok(MaterializedAspectState::StructValue(
            materialize_authoritative_struct_state(
                binding.aspect_key(),
                authoritative_state_for_side(context, side),
            )?,
        )),
        _ => Err(invalid_binding_context(
            binding.aspect_key(),
            "relation field aspect requires scalar or struct contract shape",
        )),
    }
}

fn materialize_relation_source_endpoint_binding_state<'a>(
    binding: &LoweredAspectContractBinding,
    context: BindingEvaluationContext<'a>,
    side: MaterializationSide,
) -> Result<MaterializedAspectState<'a>, CanonicalDeltaError> {
    let Some((old_source, new_source, _, _)) = context.relation_endpoints() else {
        return Err(invalid_binding_context(
            binding.aspect_key(),
            "relation source endpoint aspect cannot be evaluated against entity context",
        ));
    };
    Ok(MaterializedAspectState::EndpointIdentity(match side {
        MaterializationSide::Old => old_source,
        MaterializationSide::New => new_source,
    }))
}

fn materialize_relation_target_endpoint_binding_state<'a>(
    binding: &LoweredAspectContractBinding,
    context: BindingEvaluationContext<'a>,
    side: MaterializationSide,
) -> Result<MaterializedAspectState<'a>, CanonicalDeltaError> {
    let Some((_, _, old_target, new_target)) = context.relation_endpoints() else {
        return Err(invalid_binding_context(
            binding.aspect_key(),
            "relation target endpoint aspect cannot be evaluated against entity context",
        ));
    };
    Ok(MaterializedAspectState::EndpointIdentity(match side {
        MaterializationSide::Old => old_target,
        MaterializationSide::New => new_target,
    }))
}

fn materialize_authoritative_scalar_state<'a>(
    aspect_key: &AspectKey,
    authoritative_state: Option<&'a AuthoritativeRecordAspectState>,
) -> Result<Option<&'a worth_foundational::facade::AspectValue>, CanonicalDeltaError> {
    let Some(entry) = authoritative_state.and_then(|state| state.get(aspect_key)) else {
        return Ok(None);
    };
    match entry.view() {
        ContractValidatedAspectValueView::Scalar(value) => Ok(Some(value)),
        ContractValidatedAspectValueView::Struct(_) => {
            Err(CanonicalDeltaError::AspectValueMaterialization {
                aspect_key: aspect_key.clone(),
                detail: format!(
                    "authoritative state for relation aspect {:?} is struct-valued but scalar evidence was requested",
                    aspect_key
                ),
            })
        }
    }
}

fn materialize_authoritative_struct_state<'a>(
    aspect_key: &AspectKey,
    authoritative_state: Option<&'a AuthoritativeRecordAspectState>,
) -> Result<Option<&'a StructAspectValue>, CanonicalDeltaError> {
    let Some(entry) = authoritative_state.and_then(|state| state.get(aspect_key)) else {
        return Ok(None);
    };
    match entry.view() {
        ContractValidatedAspectValueView::Struct(value) => Ok(Some(value)),
        ContractValidatedAspectValueView::Scalar(_) => {
            Err(CanonicalDeltaError::AspectValueMaterialization {
                aspect_key: aspect_key.clone(),
                detail: format!(
                    "authoritative state for relation aspect {:?} is scalar-valued but struct evidence was requested",
                    aspect_key
                ),
            })
        }
    }
}

fn binding_evidence_from_states<'a>(
    aspect_key: &AspectKey,
    old_state: MaterializedAspectState<'a>,
    new_state: MaterializedAspectState<'a>,
) -> Result<(super::BorrowedAspectDeltaEvidence<'a>, bool), CanonicalDeltaError> {
    match (old_state, new_state) {
        (
            MaterializedAspectState::ScalarValue(old_value),
            MaterializedAspectState::ScalarValue(new_value),
        ) => {
            let changed = (old_value.is_some() || new_value.is_some()) && old_value != new_value;
            Ok((
                super::BorrowedAspectDeltaEvidence::ScalarAspectValueTransition {
                    old_present: old_value.is_some(),
                    new_present: new_value.is_some(),
                    old_value: if changed { old_value } else { None },
                    new_value: if changed { new_value } else { None },
                },
                changed,
            ))
        }
        (
            MaterializedAspectState::StructValue(old_value),
            MaterializedAspectState::StructValue(new_value),
        ) => {
            let changed = (old_value.is_some() || new_value.is_some()) && old_value != new_value;
            Ok((
                super::BorrowedAspectDeltaEvidence::StructAspectValueTransition {
                    old_present: old_value.is_some(),
                    new_present: new_value.is_some(),
                    old_value: if changed { old_value } else { None },
                    new_value: if changed { new_value } else { None },
                },
                changed,
            ))
        }
        (
            MaterializedAspectState::EndpointIdentity(old_endpoint),
            MaterializedAspectState::EndpointIdentity(new_endpoint),
        ) => {
            let changed = old_endpoint != new_endpoint;
            Ok((
                super::BorrowedAspectDeltaEvidence::EndpointIdentity {
                    old: old_endpoint,
                    new: new_endpoint,
                },
                changed,
            ))
        }
        (
            MaterializedAspectState::LifecycleTransition(transition),
            MaterializedAspectState::LifecycleTransition(_),
        ) => Ok((
            super::BorrowedAspectDeltaEvidence::Lifecycle {
                transition,
            },
            transition != LifecycleTransitionClass::NoTransition,
        )),
        (
            MaterializedAspectState::StructuralChange(change),
            MaterializedAspectState::StructuralChange(_),
        ) => Ok((
            super::BorrowedAspectDeltaEvidence::Structural { change },
            true,
        )),
        (old_state, new_state) => Err(CanonicalDeltaError::InvalidLoweredBindingForRecordClass {
            aspect_key: aspect_key.clone(),
            detail: format!(
                "materialized aspect state mismatch during canonical delta evaluation: old={old_state:?}, new={new_state:?}"
            ),
        }),
    }
}

fn authoritative_state_for_side<'a>(
    context: BindingEvaluationContext<'a>,
    side: MaterializationSide,
) -> Option<&'a AuthoritativeRecordAspectState> {
    match side {
        MaterializationSide::Old => context.old_authoritative_state(),
        MaterializationSide::New => context.new_authoritative_state(),
    }
}

fn invalid_binding_context(aspect_key: &AspectKey, detail: &str) -> CanonicalDeltaError {
    CanonicalDeltaError::InvalidLoweredBindingForRecordClass {
        aspect_key: aspect_key.clone(),
        detail: format!("{detail} during canonical delta evaluation"),
    }
}

fn entity_field_binding_requires_authoritative_patch(
    aspect_key: &AspectKey,
    field: &worth_foundational::facade::FieldKey,
) -> CanonicalDeltaError {
    CanonicalDeltaError::EntityFieldBindingRequiresAuthoritativePatchEvidence {
        target: crate::transactions::data::planned_single_field_locator(
            aspect_key.clone(),
            field.clone(),
        ),
    }
}
