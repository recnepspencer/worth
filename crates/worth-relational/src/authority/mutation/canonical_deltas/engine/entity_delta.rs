use crate::publication::patch::data::{ordered_aspect_keys, RecordStructuralChange};
use crate::transactions::data::RecordRef;

use crate::authority::mutation::canonical_deltas::data::{
    BindingEvaluationContext, CanonicalDeltaError, CanonicalRecordAspectDelta,
};
use crate::authority::mutation::canonical_deltas::patch_authority::{
    authoritative_patch_binding_evidence, evaluate_authoritative_patch_delta,
};
use crate::authority::mutation::field_versions::changed_fields;
use crate::authority::mutation::MutationWorkspace;

use super::delta_assembly::{assemble_delta, evaluate_bindings};
use super::state_views::EntityAuthoritativeState;

pub(super) fn evaluate_entity_lifecycle_delta(
    workspace: &MutationWorkspace<'_>,
    entity_id: crate::identity::data::EntityId,
    kind_id: crate::identity::data::KindId,
    authoritative_patch: Option<&worth_foundational::facade::AuthoritativeRecordAspectPatch>,
    structural_change: RecordStructuralChange,
) -> Result<CanonicalRecordAspectDelta, CanonicalDeltaError> {
    match authoritative_patch {
        Some(authoritative_patch) => {
            let plan = workspace
                .entity_aspect_plan(kind_id)
                .ok_or(CanonicalDeltaError::MissingEntityAspectPlan { kind_id })?;
            Ok(evaluate_authoritative_patch_delta(
                RecordRef::Entity(entity_id),
                kind_id,
                plan,
                structural_change,
                authoritative_patch,
            ))
        }
        None => evaluate_entity_delta(
            workspace,
            entity_id,
            kind_id,
            EntityAuthoritativeState {
                authoritative_state: None,
            },
            EntityAuthoritativeState {
                authoritative_state: None,
            },
            structural_change,
        ),
    }
}

pub(super) fn evaluate_entity_update_delta(
    workspace: &MutationWorkspace<'_>,
    entity_id: crate::identity::data::EntityId,
    kind_id: crate::identity::data::KindId,
    old_state: EntityAuthoritativeState<'_>,
    new_state: EntityAuthoritativeState<'_>,
    authoritative_patch: Option<&worth_foundational::facade::AuthoritativeRecordAspectPatch>,
) -> Result<CanonicalRecordAspectDelta, CanonicalDeltaError> {
    match authoritative_patch {
        Some(authoritative_patch) => {
            let plan = workspace
                .entity_aspect_plan(kind_id)
                .ok_or(CanonicalDeltaError::MissingEntityAspectPlan { kind_id })?;
            let mut delta = evaluate_entity_delta(
                workspace,
                entity_id,
                kind_id,
                old_state,
                new_state,
                RecordStructuralChange::Updated,
            )?;
            for binding in &mut delta.evaluated_bindings {
                let Some(lowered_binding) = plan
                    .executable_bindings
                    .iter()
                    .find(|candidate| candidate.aspect_key() == &binding.aspect_key)
                else {
                    continue;
                };
                if let Some(evidence) = authoritative_patch_binding_evidence(
                    lowered_binding,
                    RecordStructuralChange::Updated,
                    authoritative_patch,
                ) {
                    binding.field_revision_changes =
                        Some(changed_fields(binding, RecordStructuralChange::Updated));
                    binding.evidence = evidence;
                }
            }
            delta.changed_aspects = ordered_aspect_keys(
                delta
                    .evaluated_bindings
                    .iter()
                    .filter(|binding| binding.changed)
                    .map(|binding| binding.aspect_key.clone()),
            );
            Ok(delta)
        }
        None => evaluate_entity_delta(
            workspace,
            entity_id,
            kind_id,
            old_state,
            new_state,
            RecordStructuralChange::Updated,
        ),
    }
}

pub(super) fn evaluate_entity_delta(
    workspace: &MutationWorkspace<'_>,
    entity_id: crate::identity::data::EntityId,
    kind_id: crate::identity::data::KindId,
    old_state: EntityAuthoritativeState<'_>,
    new_state: EntityAuthoritativeState<'_>,
    structural_change: RecordStructuralChange,
) -> Result<CanonicalRecordAspectDelta, CanonicalDeltaError> {
    let plan = workspace
        .entity_aspect_plan(kind_id)
        .ok_or(CanonicalDeltaError::MissingEntityAspectPlan { kind_id })?;
    let evaluated_bindings = evaluate_bindings(
        plan,
        BindingEvaluationContext::Entity {
            structural_change,
            old_authoritative_state: old_state.authoritative_state,
            new_authoritative_state: new_state.authoritative_state,
        },
    )?;
    Ok(assemble_delta(
        RecordRef::Entity(entity_id),
        kind_id,
        plan,
        structural_change,
        evaluated_bindings,
    ))
}
