use crate::contracts::schema::data::aspect_contracts::lowered_plan::LoweredAspectContractBinding;
use worth_foundational::facade::{
    AspectBinding, AspectKey, AspectShape, AuthoritativeRecordAspectState,
    ContractValidatedAspectValueView, StructAspectValue,
};

use super::borrowed_evidence::BorrowedAspectDeltaEvidence as Evidence;
use super::data::{
    BindingEvaluationContext, CanonicalAspectDeltaEvidence, CanonicalDeltaError,
    LifecycleTransitionClass,
};
use super::lifecycle_transition_evidence::lifecycle_transition;

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
) -> Result<(Evidence<'a>, bool), CanonicalDeltaError> {
    let aspect_key = binding.aspect_key();
    match &binding.target {
        AspectBinding::EntityField { field } => {
            if !matches!(context, BindingEvaluationContext::Entity { .. }) {
                return Err(invalid_binding_context(
                    aspect_key,
                    "entity field aspect cannot be evaluated against relation context",
                ));
            }
            field_evidence(binding, context, || {
                entity_field_binding_requires_authoritative_patch(aspect_key, field)
            })
        }
        AspectBinding::RelationField { .. } => {
            if !matches!(context, BindingEvaluationContext::Relation { .. }) {
                return Err(invalid_binding_context(
                    aspect_key,
                    "relation field aspect cannot be evaluated against entity context",
                ));
            }
            field_evidence(binding, context, || {
                invalid_binding_context(
                    aspect_key,
                    "relation field aspect requires scalar or struct contract shape",
                )
            })
        }
        AspectBinding::RelationSourceEndpoint | AspectBinding::RelationTargetEndpoint => {
            let source = matches!(&binding.target, AspectBinding::RelationSourceEndpoint);
            let (old_source, new_source, old_target, new_target) =
                context.relation_endpoints().ok_or_else(|| {
                    let detail = if source {
                        "relation source endpoint aspect cannot be evaluated against entity context"
                    } else {
                        "relation target endpoint aspect cannot be evaluated against entity context"
                    };
                    invalid_binding_context(aspect_key, detail)
                })?;
            let (old, new) = if source {
                (old_source, new_source)
            } else {
                (old_target, new_target)
            };
            Ok((Evidence::EndpointIdentity { old, new }, old != new))
        }
        AspectBinding::LifecycleTransition => {
            let transition = lifecycle_transition(context.structural_change());
            Ok((
                Evidence::Lifecycle { transition },
                transition != LifecycleTransitionClass::NoTransition,
            ))
        }
        AspectBinding::StructuralRegion
        | AspectBinding::StructuralPartition
        | AspectBinding::StructuralFacet => Ok((
            Evidence::Structural {
                change: context.structural_change(),
            },
            true,
        )),
        _ => Err(invalid_binding_context(
            aspect_key,
            "unsupported authoritative aspect binding",
        )),
    }
}

fn field_evidence<'a>(
    binding: &LoweredAspectContractBinding,
    context: BindingEvaluationContext<'a>,
    unsupported_shape: impl FnOnce() -> CanonicalDeltaError,
) -> Result<(Evidence<'a>, bool), CanonicalDeltaError> {
    let key = binding.aspect_key();
    let before = context.old_authoritative_state();
    let after = context.new_authoritative_state();
    match binding.contract.shape() {
        AspectShape::Scalar(_) => {
            let old = materialize_authoritative_scalar_state(key, before)?;
            let new = materialize_authoritative_scalar_state(key, after)?;
            let changed = old != new;
            Ok((
                Evidence::ScalarAspectValueTransition {
                    old_present: old.is_some(),
                    new_present: new.is_some(),
                    old_value: if changed { old } else { None },
                    new_value: if changed { new } else { None },
                },
                changed,
            ))
        }
        AspectShape::Struct(_) => {
            let old = materialize_authoritative_struct_state(key, before)?;
            let new = materialize_authoritative_struct_state(key, after)?;
            let changed = old != new;
            Ok((
                Evidence::StructAspectValueTransition {
                    old_present: old.is_some(),
                    new_present: new.is_some(),
                    old_value: if changed { old } else { None },
                    new_value: if changed { new } else { None },
                },
                changed,
            ))
        }
        _ => Err(unsupported_shape()),
    }
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
        target:
            crate::contracts::transactions::data::aspect_field_patch::planned_single_field_locator(
                aspect_key.clone(),
                field.clone(),
            ),
    }
}
