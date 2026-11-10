use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::record_changes::{
    allocate_entity_with_extra, delete_entity_with_cascade,
};
use crate::authority::mutation::stale_targets::ensure_entity_target_is_current;
use crate::authority::mutation::MutationWorkspace;
use crate::transactions::data::{RecordAspectPatchTarget, ReplaceEntityIntent};
use worth_foundational::facade::PortablePatchReadmissionPurpose;

use super::field_authoring_candidate::FieldAuthoringDomain;
use super::record_aspect_patch;

pub(super) fn apply(
    intent: &ReplaceEntityIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, crate::transactions::data::TransactionCommitError> {
    let version_id = workspace.version_id();
    let cascade_delete_policy = workspace.cascade_delete_policy();
    let target = RecordAspectPatchTarget::EntityCreation {
        kind_id: intent.replacement.kind_id,
    };
    let patch = record_aspect_patch::readmit_field_authoring(
        &workspace.execution(),
        &intent.replacement.fields,
        PortablePatchReadmissionPurpose::RecordCreation,
        workspace.entity_aspect_plan(intent.replacement.kind_id),
        target,
        FieldAuthoringDomain::Entity,
    )?;
    let replacement_state =
        record_aspect_patch::apply_entity(&workspace.execution(), None, &patch, target)?;
    let mut outcome = MutationOutcome::entity_deleted(&workspace.execution(), intent.entity_id)?;
    let replacement_id = workspace.with_context(|context| {
        ensure_entity_target_is_current(context.state, intent.entity_id)?;
        delete_entity_with_cascade(
            &context.execution,
            context.state,
            version_id,
            intent.entity_id,
            context.schema,
            cascade_delete_policy,
            &mut outcome,
        )?;
        let replacement_id = allocate_entity_with_extra(
            &context.execution,
            context.state,
            context.record_allocations,
            version_id,
            intent.replacement.partition_id,
            intent.replacement.kind_id,
            replacement_state,
        )?;
        context
            .state
            .mark_entity_slot_touched_for_request(
                replacement_id.partition_id,
                replacement_id.slot_index(),
                &context.execution,
            )
            .map_err(|cause| {
                crate::transactions::data::TransactionCommitError::execution(cause.into())
            })?;
        Ok::<_, crate::transactions::data::TransactionCommitError>(replacement_id)
    })?;
    outcome.extend(
        &workspace.execution(),
        MutationOutcome::entity_replaced(
            &workspace.execution(),
            intent.entity_id,
            replacement_id,
            intent.replacement.kind_id,
            record_aspect_patch::published_patch(patch),
        )?,
    )?;
    Ok(outcome)
}
