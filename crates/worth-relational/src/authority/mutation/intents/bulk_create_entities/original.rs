use super::*;
use crate::symbols::data::ClientKey;

pub(in crate::authority::mutation::intents) fn apply_original<'a>(
    intent: &'a BulkEntityCreateIntent,
    workspace: &mut MutationWorkspace<'a>,
) -> Result<MutationOutcome, TransactionCommitError> {
    let mut outcome =
        MutationOutcome::with_capacity(&workspace.execution(), intent.field_patches.len(), 1)?;
    outcome.record_event(
        &workspace.execution(),
        crate::authority::mutation::outcomes::MutationEvent::BulkEntitiesCreated {
            partition_id: intent.partition_id,
            kind_id: intent.kind_id,
            count: 0,
        },
    )?;
    let version = workspace.version_id();
    for (client, fields) in intent.client_keys.iter().zip(&intent.field_patches) {
        let target = RecordAspectPatchTarget::EntityCreation {
            kind_id: intent.kind_id,
        };
        let patch = record_aspect_patch::readmit_field_authoring(
            &workspace.execution(),
            fields,
            PortablePatchReadmissionPurpose::RecordCreation,
            workspace.entity_aspect_plan(intent.kind_id),
            target,
            FieldAuthoringDomain::Entity,
        )?;
        let extra =
            record_aspect_patch::apply_entity(&workspace.execution(), None, &patch, target)?;
        apply_prepared_row(
            intent,
            workspace,
            &mut outcome,
            version,
            client,
            (patch, extra),
        )?;
    }
    outcome.set_last_event_count(intent.field_patches.len());
    Ok(outcome)
}

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
