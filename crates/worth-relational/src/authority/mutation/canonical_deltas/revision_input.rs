//! One borrowed binding uses the existing value comparison and patch selector.
use super::data::{BindingEvaluationContext as Context, CanonicalDeltaError};
use super::materialized_state::borrowed_authoritative_binding_delta;
use super::patch_authority::borrowed_authoritative_patch_binding_evidence;
use super::BorrowedAspectDeltaEvidence;
use crate::authority::mutation::{MutationApplyDenial, MutationWorkspace, RecordMutation};
use crate::capabilities::AspectPlanSource;
use crate::identity::data::{KindId, VersionId};
use crate::publication::patch::data::RecordStructuralChange as Structural;
use crate::schema::data::{AspectBinding, LoweredAspectContractBinding};
use crate::transactions::data::RecordRef;
use worth_foundational::facade::{AspectShape, AuthoritativeRecordAspectPatch};

pub(crate) fn write_original_revision_stamps(
    mutation: &RecordMutation,
    workspace: &mut MutationWorkspace<'_>,
    plans: &impl AspectPlanSource,
    version: VersionId,
) -> Result<(), MutationApplyDenial> {
    // The plan is immutable schema DATA. Workspace mutation only changes the
    // staged records; borrow the schema outside its mutable write opportunity.
    let (target, kind, context, patch) = context(mutation);
    let plan = match target {
        RecordRef::Entity(_) => plans
            .entity_aspect_plan(kind)
            .ok_or(CanonicalDeltaError::MissingEntityAspectPlan { kind_id: kind }),
        RecordRef::Relation(_) => plans
            .relation_aspect_plan(kind)
            .ok_or(CanonicalDeltaError::MissingRelationAspectPlan { kind_id: kind }),
    }
    .map_err(|error| error.to_commit_conflict())?;
    for binding in &plan.executable_bindings {
        let Some((evidence, changed)) = evidence(mutation, binding, context, patch)
            .map_err(|error| error.to_commit_conflict())?
        else {
            continue;
        };
        workspace.write_original_binding_revision_stamps(
            target.clone(),
            binding,
            &evidence,
            context.structural_change(),
            changed,
            version,
        )?;
    }
    Ok(())
}

fn evidence<'a>(
    mutation: &RecordMutation,
    binding: &'a LoweredAspectContractBinding,
    context: Context<'a>,
    patch: Option<&'a AuthoritativeRecordAspectPatch>,
) -> Result<Option<(BorrowedAspectDeltaEvidence<'a>, bool)>, CanonicalDeltaError> {
    match mutation {
        RecordMutation::EntityDeleted {
            authoritative_patch: None,
            old_authoritative_aspect_state,
            ..
        } => Ok(super::patch_authority::borrowed_deletion_binding_evidence(
            binding,
            context.structural_change(),
            old_authoritative_aspect_state
                .as_ref()
                .and_then(|extra| extra.authoritative_aspect_state()),
        )
        .map(|evidence| (evidence, true))),
        RecordMutation::EntityUpdated { .. } => {
            borrowed_authoritative_binding_delta(binding, context).map(Some)
        }
        RecordMutation::RelationUpdated { .. } if patch.is_some() => {
            let Some(patched) = borrowed_authoritative_patch_binding_evidence(
                binding,
                context.structural_change(),
                patch.expect("guarded patch"),
            ) else {
                return Ok(None);
            };
            if matches!(binding.target, AspectBinding::RelationField { .. })
                && matches!(
                    binding.contract.shape(),
                    AspectShape::Scalar(_) | AspectShape::Struct(_)
                )
            {
                borrowed_authoritative_binding_delta(binding, context).map(Some)
            } else {
                Ok(Some((patched, true)))
            }
        }
        _ => match patch {
            Some(patch) => Ok(borrowed_authoritative_patch_binding_evidence(
                binding,
                context.structural_change(),
                patch,
            )
            .map(|evidence| (evidence, true))),
            None => borrowed_authoritative_binding_delta(binding, context).map(Some),
        },
    }
}

fn context(
    mutation: &RecordMutation,
) -> (
    RecordRef,
    KindId,
    Context<'_>,
    Option<&AuthoritativeRecordAspectPatch>,
) {
    macro_rules! entity {
        ($id:expr,$kind:expr,$change:expr,$old:expr,$new:expr,$patch:expr) => {
            (
                RecordRef::Entity(*$id),
                *$kind,
                Context::Entity {
                    structural_change: $change,
                    old_authoritative_state: $old,
                    new_authoritative_state: $new,
                },
                $patch,
            )
        };
    }
    macro_rules! relation {
        ($id:expr,$kind:expr,$change:expr,$old:expr,$new:expr,$os:expr,$ns:expr,$ot:expr,$nt:expr,$patch:expr) => {
            (
                RecordRef::Relation(*$id),
                *$kind,
                Context::Relation {
                    structural_change: $change,
                    old_authoritative_state: $old,
                    new_authoritative_state: $new,
                    old_source: $os,
                    new_source: $ns,
                    old_target: $ot,
                    new_target: $nt,
                },
                $patch,
            )
        };
    }
    match mutation {
        RecordMutation::EntityCreated {
            entity_id,
            kind_id,
            authoritative_patch,
        } => entity!(
            entity_id,
            kind_id,
            Structural::Created,
            None,
            None,
            authoritative_patch.as_ref().map(|patch| patch.as_patch())
        ),
        RecordMutation::EntityDeleted {
            entity_id,
            kind_id,
            authoritative_patch,
            ..
        } => entity!(
            entity_id,
            kind_id,
            Structural::Deleted,
            None,
            None,
            authoritative_patch.as_ref().map(|patch| patch.as_patch())
        ),
        RecordMutation::EntityRematerialized {
            entity_id,
            kind_id,
            authoritative_patch,
            ..
        } => entity!(
            entity_id,
            kind_id,
            Structural::Rematerialized,
            None,
            None,
            authoritative_patch.as_ref().map(|patch| patch.as_patch())
        ),
        RecordMutation::EntityUpdated {
            entity_id,
            kind_id,
            old_authoritative_aspect_state,
            new_authoritative_aspect_state,
            authoritative_patch,
        } => entity!(
            entity_id,
            kind_id,
            Structural::Updated,
            old_authoritative_aspect_state
                .as_ref()
                .and_then(|extra| extra.authoritative_aspect_state()),
            new_authoritative_aspect_state
                .as_ref()
                .and_then(|extra| extra.authoritative_aspect_state()),
            authoritative_patch.as_ref().map(|patch| patch.as_patch())
        ),
        RecordMutation::EntityMaterializationSuspended {
            entity_id,
            kind_id,
            old_authoritative_aspect_state,
        } => entity!(
            entity_id,
            kind_id,
            Structural::MaterializationSuspended,
            old_authoritative_aspect_state
                .as_ref()
                .and_then(|extra| extra.authoritative_aspect_state()),
            None,
            None
        ),
        RecordMutation::RelationCreated {
            relation_id,
            kind_id,
            source,
            target,
            authoritative_patch,
        } => relation!(
            relation_id,
            kind_id,
            Structural::Created,
            None,
            None,
            None,
            Some(*source),
            None,
            Some(*target),
            authoritative_patch.as_ref().map(|patch| patch.as_patch())
        ),
        RecordMutation::RelationRematerialized {
            relation_id,
            kind_id,
            source,
            target,
            authoritative_patch,
            ..
        } => relation!(
            relation_id,
            kind_id,
            Structural::Rematerialized,
            None,
            None,
            None,
            Some(*source),
            None,
            Some(*target),
            authoritative_patch.as_ref().map(|patch| patch.as_patch())
        ),
        RecordMutation::RelationUpdated {
            relation_id,
            kind_id,
            old_source,
            new_source,
            old_target,
            new_target,
            old_authoritative_aspect_state,
            new_authoritative_aspect_state,
            authoritative_patch,
        } => relation!(
            relation_id,
            kind_id,
            Structural::Updated,
            old_authoritative_aspect_state
                .as_ref()
                .and_then(|extra| extra.authoritative_aspect_state()),
            new_authoritative_aspect_state
                .as_ref()
                .and_then(|extra| extra.authoritative_aspect_state()),
            Some(*old_source),
            Some(*new_source),
            Some(*old_target),
            Some(*new_target),
            authoritative_patch.as_ref().map(|patch| patch.as_patch())
        ),
        RecordMutation::RelationDeleted {
            relation_id,
            kind_id,
            source,
            target,
            authoritative_aspect_state,
        } => relation!(
            relation_id,
            kind_id,
            Structural::Deleted,
            authoritative_aspect_state
                .as_ref()
                .and_then(|extra| extra.authoritative_aspect_state()),
            None,
            Some(*source),
            None,
            Some(*target),
            None,
            None
        ),
        RecordMutation::RelationMaterializationSuspended {
            relation_id,
            kind_id,
            source,
            target,
            old_authoritative_aspect_state,
        } => relation!(
            relation_id,
            kind_id,
            Structural::MaterializationSuspended,
            old_authoritative_aspect_state
                .as_ref()
                .and_then(|extra| extra.authoritative_aspect_state()),
            None,
            Some(*source),
            None,
            Some(*target),
            None,
            None
        ),
        RecordMutation::RelationRetainedForAudit {
            relation_id,
            kind_id,
            source,
            target,
            authoritative_aspect_state,
        } => relation!(
            relation_id,
            kind_id,
            Structural::RetainedForAudit,
            authoritative_aspect_state
                .as_ref()
                .and_then(|extra| extra.authoritative_aspect_state()),
            authoritative_aspect_state
                .as_ref()
                .and_then(|extra| extra.authoritative_aspect_state()),
            Some(*source),
            Some(*source),
            Some(*target),
            Some(*target),
            None
        ),
    }
}
