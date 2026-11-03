use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::record_changes::allocate_entity_with_extra;
use crate::authority::mutation::MutationWorkspace;
use crate::transactions::data::{EntitySpec, RecordAspectPatchTarget};
use worth_foundational::facade::PortablePatchReadmissionPurpose;

use super::field_authoring_candidate::FieldAuthoringDomain;
use super::record_aspect_patch;

pub(super) fn apply<'a>(
    spec: &'a EntitySpec,
    workspace: &mut MutationWorkspace<'a>,
) -> Result<MutationOutcome, crate::transactions::data::TransactionCommitError> {
    let version_id = workspace.version_id();
    let target = RecordAspectPatchTarget::EntityCreation {
        kind_id: spec.kind_id,
    };
    let patch = record_aspect_patch::readmit_field_authoring(
        &workspace.execution(),
        &spec.fields,
        PortablePatchReadmissionPurpose::RecordCreation,
        workspace.entity_aspect_plan(spec.kind_id),
        target,
        FieldAuthoringDomain::Entity,
    )?;
    let authoritative_aspect_state =
        record_aspect_patch::apply_entity(&workspace.execution(), None, &patch, target)?;
    let entity_id = workspace.with_context(|context| {
        let entity_id = allocate_entity_with_extra(
            &context.execution,
            context.state,
            context.record_allocations,
            version_id,
            spec.partition_id,
            spec.kind_id,
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
        spec.partition_id,
        spec.kind_id,
        &spec.client_key,
        entity_id,
    )?;
    MutationOutcome::entity_created(
        &workspace.execution(),
        entity_id,
        spec.kind_id,
        record_aspect_patch::published_patch(patch),
    )
}
