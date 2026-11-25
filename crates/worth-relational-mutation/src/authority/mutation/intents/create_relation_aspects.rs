use crate::contracts::transactions::data::outcomes::commit_conflict::{
    CommitConflict, CommitConflictOwnerPort as _,
};
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use worth_foundational::facade::PortablePatchReadmissionPurpose;

use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::record_changes::allocate_relation;
use crate::authority::mutation::MutationWorkspace;
use crate::contracts::transactions::data::intents::mutation_intent::RelationAspectCreateIntent;
use crate::contracts::transactions::data::outcomes::conflict_class::record_aspect_patch_denial::RecordAspectPatchTarget;
use crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::contracts::transactions::data::primitives::EntityReference;

use super::{record_aspect_patch, relation_endpoint_candidate};

pub(super) fn apply<'a>(
    intent: &'a RelationAspectCreateIntent,
    workspace: &mut MutationWorkspace<'a>,
) -> Result<
    MutationOutcome,
    crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
>{
    let source = resolve_endpoint(workspace, &intent.source, "source")?;
    let target_entity = resolve_endpoint(workspace, &intent.target, "target")?;
    let target = RecordAspectPatchTarget::RelationCreation {
        kind_id: intent.kind_id,
    };
    let plan = workspace.relation_aspect_plan(intent.kind_id);
    let candidate = relation_endpoint_candidate::append_authoritative_endpoints(
        &workspace.execution(),
        super::field_authoring_candidate::FieldCandidate::copy_portable(
            &workspace.execution(),
            &intent.aspect_patch,
        )
        .map_err(|cause| {
            crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::execution(cause.into())
        })?,
        plan,
        source,
        target_entity,
    )?;
    let patch = record_aspect_patch::readmit_candidate(
        &workspace.execution(),
        candidate,
        PortablePatchReadmissionPurpose::RecordCreation,
        plan,
        target,
    )?;
    let authoritative_aspect_state =
        record_aspect_patch::apply_relation(&workspace.execution(), None, &patch, target)?;
    let version_id = workspace.version_id();
    let relation_id = workspace.with_context(|context| {
        let relation_id = allocate_relation(
            &context.execution,
            context.state,
            context.record_allocations,
            version_id,
            intent.partition_id,
            intent.kind_id,
            source,
            target_entity,
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
                crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::execution(cause.into())
            })?;
        Ok::<_, crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError>(relation_id)
    })?;
    workspace.register_created_relation(
        intent.partition_id,
        intent.kind_id,
        &intent.client_key,
        &intent.source,
        &intent.target,
        relation_id,
    )?;
    MutationOutcome::relation_created(
        &workspace.execution(),
        relation_id,
        source,
        target_entity,
        intent.kind_id,
        record_aspect_patch::published_patch(patch),
    )
}

fn resolve_endpoint(
    workspace: &MutationWorkspace<'_>,
    reference: &EntityReference,
    label: &str,
) -> Result<crate::contracts::identity::data::record_ids::EntityId, CommitConflict> {
    workspace
        .resolve_entity_reference(reference)
        .ok_or_else(|| {
            CommitConflict::new(ConflictClass::InvalidRelationEndpoint {
                detail: format!(
                    "relation aspect creation requires a live or same-batch created {label} entity"
                ),
            })
        })
}
