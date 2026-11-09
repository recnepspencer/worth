use super::*;
use crate::symbols::data::ClientKey;

pub(super) fn apply_prepared_row<'a>(
    intent: &'a BulkEntityCreateIntent,
    workspace: &mut MutationWorkspace<'a>,
    outcome: &mut MutationOutcome,
    version_id: crate::identity::data::VersionId,
    client_key: &'a ClientKey,
    aspect_plan: (
        crate::authority::mutation::outcomes::MutationAspectPatch,
        crate::storage::substrate::EntityExtra,
    ),
) -> Result<(), TransactionCommitError> {
    let entity_id = workspace.with_context(|context| {
        let entity_id = allocate_entity_with_extra(
            &context.execution,
            context.state,
            context.record_allocations,
            version_id,
            intent.partition_id,
            intent.kind_id,
            aspect_plan.1,
        )?;
        context
            .state
            .mark_entity_slot_touched_for_request(
                entity_id.partition_id,
                entity_id.slot_index(),
                &context.execution,
            )
            .map_err(|cause| TransactionCommitError::execution(cause.into()))?;
        Ok::<_, TransactionCommitError>(entity_id)
    })?;
    workspace.register_created_entity(
        intent.partition_id,
        intent.kind_id,
        client_key,
        entity_id,
    )?;
    outcome.record_change(
        &workspace.execution(),
        RecordMutation::EntityCreated {
            entity_id,
            kind_id: intent.kind_id,
            authoritative_patch: record_aspect_patch::published_patch(aspect_plan.0),
        },
    )?;
    Ok(())
}
