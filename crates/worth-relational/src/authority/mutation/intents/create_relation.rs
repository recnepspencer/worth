use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::record_changes::allocate_relation;
use crate::authority::mutation::MutationWorkspace;
use crate::transactions::data::{
    CommitConflict, ConflictClass, EntityReference, RecordAspectPatchTarget, RelationSpec,
};
use worth_foundational::facade::PortablePatchReadmissionPurpose;

use super::field_authoring_candidate::{self, FieldAuthoringDomain};
use super::{record_aspect_patch, relation_endpoint_candidate};

pub(super) fn apply<'a>(
    spec: &'a RelationSpec,
    workspace: &mut MutationWorkspace<'a>,
) -> Result<MutationOutcome, crate::transactions::data::TransactionCommitError> {
    let version_id = workspace.version_id();
    let source = resolve_entity_reference(workspace, &spec.source)?;
    let target = resolve_entity_reference(workspace, &spec.target)?;
    let target_record = RecordAspectPatchTarget::RelationCreation {
        kind_id: spec.kind_id,
    };
    let plan = workspace.relation_aspect_plan(spec.kind_id);
    let candidate = field_authoring_candidate::lower(
        &workspace.execution(),
        &spec.fields,
        PortablePatchReadmissionPurpose::RecordCreation,
        plan,
        spec.kind_id,
        FieldAuthoringDomain::Relation,
    )
    .map_err(|denial| record_aspect_patch::MutationPatchDenial::candidate(target_record, denial))?;
    let candidate = relation_endpoint_candidate::append_authoritative_endpoints(
        &workspace.execution(),
        candidate,
        plan,
        source,
        target,
    )?;
    let patch = record_aspect_patch::readmit_candidate(
        &workspace.execution(),
        candidate,
        PortablePatchReadmissionPurpose::RecordCreation,
        plan,
        target_record,
    )?;
    let authoritative_aspect_state =
        record_aspect_patch::apply_relation(&workspace.execution(), None, &patch, target_record)?;
    let relation_id = workspace.with_context(|context| {
        let relation_id = allocate_relation(
            &context.execution,
            context.state,
            context.record_allocations,
            version_id,
            spec.partition_id,
            spec.kind_id,
            source,
            target,
            authoritative_aspect_state,
        )?;
        context
            .state
            .mark_relation_slot_touched_for_request(
                relation_id.partition_id,
                relation_id.slot_index(),
                &context.execution,
            )
            .map_err(|cause| {
                crate::transactions::data::TransactionCommitError::execution(cause.into())
            })?;
        Ok::<_, crate::transactions::data::TransactionCommitError>(relation_id)
    })?;
    workspace.register_created_relation(
        spec.partition_id,
        spec.kind_id,
        &spec.client_key,
        &spec.source,
        &spec.target,
        relation_id,
    )?;
    Ok(MutationOutcome::relation_created(
        &workspace.execution(),
        relation_id,
        source,
        target,
        spec.kind_id,
        record_aspect_patch::published_patch(patch),
    )?)
}

fn resolve_entity_reference(
    workspace: &MutationWorkspace<'_>,
    entity_reference: &EntityReference,
) -> Result<crate::identity::data::EntityId, CommitConflict> {
    workspace
        .resolve_entity_reference(entity_reference)
        .ok_or_else(|| {
            CommitConflict::new(ConflictClass::InvalidRelationEndpoint {
                detail:
                    "relation endpoints must resolve within the same authoritative commit scope"
                        .to_string(),
            })
        })
}
