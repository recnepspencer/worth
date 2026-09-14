use crate::identity::data::EntityId;
use crate::publication::patch::data::RecordStructuralChange;
use crate::transactions::data::RecordRef;

use crate::authority::mutation::canonical_deltas::data::{
    BindingEvaluationContext, CanonicalDeltaError, CanonicalRecordAspectDelta,
};
use crate::authority::mutation::canonical_deltas::patch_authority::evaluate_authoritative_patch_delta;
use crate::authority::mutation::MutationWorkspace;

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
            Ok(evaluate_authoritative_patch_delta(
                RecordRef::Relation(relation_id),
                kind_id,
                plan,
                RecordStructuralChange::Updated,
                patch,
            ))
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
