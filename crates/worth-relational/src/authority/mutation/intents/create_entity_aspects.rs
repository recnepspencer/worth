use worth_foundational::facade::PortablePatchReadmissionPurpose;

use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::record_changes::allocate_entity_with_extra;
use crate::authority::mutation::MutationWorkspace;
use crate::transactions::data::{EntityAspectCreateIntent, RecordAspectPatchTarget};

use super::record_aspect_patch;

pub(super) fn apply<'a>(
    intent: &'a EntityAspectCreateIntent,
    workspace: &mut MutationWorkspace<'a>,
) -> Result<MutationOutcome, crate::transactions::data::TransactionCommitError> {
    let target = RecordAspectPatchTarget::EntityCreation {
        kind_id: intent.kind_id,
    };
    let patch = record_aspect_patch::readmit_original_portable(
        &workspace.execution(),
        &intent.aspect_patch,
        PortablePatchReadmissionPurpose::RecordCreation,
        workspace.entity_aspect_plan(intent.kind_id),
        target,
    )?;
    let authoritative_aspect_state =
        record_aspect_patch::apply_entity(&workspace.execution(), None, &patch, target)?;
    let version_id = workspace.version_id();
    let entity_id = workspace.with_context(|context| {
        let entity_id = allocate_entity_with_extra(
            &context.execution,
            context.state,
            context.record_allocations,
            version_id,
            intent.partition_id,
            intent.kind_id,
            authoritative_aspect_state,
        )?;
        context
            .state
            .mark_entity_slot_touched_for_request(
                entity_id.partition_id,
                entity_id.slot_index(),
                &context.execution,
            )
            .map_err(|cause| {
                crate::transactions::data::TransactionCommitError::execution(cause.into())
            })?;
        Ok::<_, crate::transactions::data::TransactionCommitError>(entity_id)
    })?;
    workspace.register_created_entity(
        intent.partition_id,
        intent.kind_id,
        &intent.client_key,
        entity_id,
    )?;
    Ok(MutationOutcome::entity_created(
        &workspace.execution(),
        entity_id,
        intent.kind_id,
        record_aspect_patch::published_patch(patch),
    )?)
}
