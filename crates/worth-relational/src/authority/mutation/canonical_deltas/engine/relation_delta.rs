use crate::identity::data::EntityId;
use crate::publication::patch::data::RecordStructuralChange;
use crate::transactions::data::RecordRef;

use crate::authority::mutation::canonical_deltas::data::{
    BindingEvaluationContext, CanonicalDeltaError, CanonicalRecordAspectDelta,
};
use crate::authority::mutation::canonical_deltas::materialized_state::evaluate_authoritative_binding_delta;
use crate::authority::mutation::canonical_deltas::patch_authority::evaluate_authoritative_patch_delta;
use crate::authority::mutation::field_versions::field_changes_for_evidence;
use crate::authority::mutation::MutationWorkspace;
use worth_foundational::facade::{AspectBinding, AspectShape};

use super::delta_assembly::{assemble_delta, evaluate_bindings};
use super::state_views::RelationState;

pub(super) fn evaluate_relation_lifecycle_delta(
    workspace: &MutationWorkspace<'_>,
    relation_id: crate::identity::data::RelationId,
    kind_id: crate::identity::data::KindId,
    source: EntityId,
    target: EntityId,
    authoritative_patch: Option<&worth_foundational::facade::AuthoritativeRecordAspectPatch>,
    structural_change: RecordStructuralChange,
) -> Result<CanonicalRecordAspectDelta, CanonicalDeltaError> {
    match authoritative_patch {
        Some(authoritative_patch) => {
            let plan = workspace
                .relation_aspect_plan(kind_id)
                .ok_or(CanonicalDeltaError::MissingRelationAspectPlan { kind_id })?;
            Ok(evaluate_authoritative_patch_delta(
                RecordRef::Relation(relation_id),
                kind_id,
                plan,
                structural_change,
                authoritative_patch,
            ))
        }
        None => evaluate_relation_delta(
            workspace,
            relation_id,
            kind_id,
            RelationState {
                source: None,
                target: None,
                authoritative_state: None,
            },
            RelationState {
                source: Some(source),
                target: Some(target),
                authoritative_state: None,
            },
            structural_change,
        ),
    }
}

pub(super) fn evaluate_relation_delta(
    workspace: &MutationWorkspace<'_>,
    relation_id: crate::identity::data::RelationId,
    kind_id: crate::identity::data::KindId,
    old_state: RelationState<'_>,
    new_state: RelationState<'_>,
    structural_change: RecordStructuralChange,
) -> Result<CanonicalRecordAspectDelta, CanonicalDeltaError> {
    let plan = workspace
        .relation_aspect_plan(kind_id)
        .ok_or(CanonicalDeltaError::MissingRelationAspectPlan { kind_id })?;
    let evaluated_bindings = evaluate_bindings(
        plan,
        BindingEvaluationContext::Relation {
            structural_change,
            old_authoritative_state: old_state.authoritative_state,
            new_authoritative_state: new_state.authoritative_state,
            old_source: old_state.source,
            new_source: new_state.source,
            old_target: old_state.target,
            new_target: new_state.target,
        },
    )?;
    Ok(assemble_delta(
        RecordRef::Relation(relation_id),
        kind_id,
        plan,
        structural_change,
        evaluated_bindings,
    ))
}

pub(super) fn evaluate_relation_update_delta(
    workspace: &MutationWorkspace<'_>,
    relation_id: crate::identity::data::RelationId,
    kind_id: crate::identity::data::KindId,
    old_state: RelationState<'_>,
    new_state: RelationState<'_>,
    authoritative_patch: Option<&worth_foundational::facade::AuthoritativeRecordAspectPatch>,
) -> Result<CanonicalRecordAspectDelta, CanonicalDeltaError> {
    match authoritative_patch {
        Some(patch) => {
            let plan = workspace
                .relation_aspect_plan(kind_id)
                .ok_or(CanonicalDeltaError::MissingRelationAspectPlan { kind_id })?;
            let mut delta = evaluate_authoritative_patch_delta(
                RecordRef::Relation(relation_id),
                kind_id,
                plan,
                RecordStructuralChange::Updated,
                patch,
            );
            for binding in &mut delta.evaluated_bindings {
                if !matches!(&binding.binding, AspectBinding::RelationField { .. })
                    || !matches!(
                        &binding.aspect_shape,
                        AspectShape::Scalar(_) | AspectShape::Struct(_)
                    )
                {
                    continue;
                }
                let lowered = plan
                    .executable_bindings
                    .iter()
                    .find(|candidate| candidate.aspect_key() == &binding.aspect_key)
                    .expect("patch binding came from the same lowered relation plan");
                let (value_evidence, _) = evaluate_authoritative_binding_delta(
                    lowered,
                    BindingEvaluationContext::Relation {
                        structural_change: RecordStructuralChange::Updated,
                        old_authoritative_state: old_state.authoritative_state,
                        new_authoritative_state: new_state.authoritative_state,
                        old_source: old_state.source,
                        new_source: new_state.source,
                        old_target: old_state.target,
                        new_target: new_state.target,
                    },
                )?;
                binding.field_revision_changes = Some(field_changes_for_evidence(
                    &binding.binding,
                    &binding.aspect_shape,
                    &value_evidence,
                    RecordStructuralChange::Updated,
                ));
            }
            Ok(delta)
        }
        None => evaluate_relation_delta(
            workspace,
            relation_id,
            kind_id,
            old_state,
            new_state,
            RecordStructuralChange::Updated,
        ),
    }
}
