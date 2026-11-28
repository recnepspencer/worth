use crate::authority::mutation::outcomes::MutationAspectPatch;
use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use worth_foundational::facade::{PortablePatchReadmissionPurpose, PortableRecordAspectPatch};

use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::record_changes::allocate_entity_with_extra;
use crate::authority::mutation::MutationWorkspace;
use crate::contracts::transactions::data::intents::mutation_intent::EntityAspectCreateIntent;
use crate::contracts::transactions::data::outcomes::conflict_class::record_aspect_patch_denial::RecordAspectPatchTarget;

use crate::authority::mutation::record_changes::allocate_relation;
use crate::contracts::transactions::data::intents::mutation_intent::RelationAspectCreateIntent;
use crate::contracts::transactions::data::outcomes::commit_conflict::{
    CommitConflict, CommitConflictOwnerPort as _,
};
use crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::contracts::transactions::data::primitives::EntityReference;

use super::{record_aspect_patch, relation_endpoint_candidate};

pub(super) fn create_entity<'a>(
    intent: &'a EntityAspectCreateIntent,
    workspace: &mut MutationWorkspace<'a>,
) -> Result<
    MutationOutcome,
    crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
>{
    let (patch, target) = admit_creation_patch(
        workspace,
        &intent.aspect_patch,
        intent.kind_id,
        Creation::Entity,
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
                crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::execution(cause.into())
            })?;
        Ok::<_, crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError>(entity_id)
    })?;
    workspace.register_created_entity(
        intent.partition_id,
        intent.kind_id,
        &intent.client_key,
        entity_id,
    )?;
    MutationOutcome::entity_created(
        &workspace.execution(),
        entity_id,
        intent.kind_id,
        record_aspect_patch::published_patch(patch),
    )
}

pub(super) fn create_relation<'a>(
    intent: &'a RelationAspectCreateIntent,
    workspace: &mut MutationWorkspace<'a>,
) -> Result<
    MutationOutcome,
    crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
>{
    let source = resolve_endpoint(workspace, &intent.source, "source")?;
    let target_entity = resolve_endpoint(workspace, &intent.target, "target")?;
    let (patch, target) = admit_creation_patch(
        workspace,
        &intent.aspect_patch,
        intent.kind_id,
        Creation::Relation {
            source,
            target: target_entity,
        },
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

enum Creation {
    Entity,
    Relation { source: EntityId, target: EntityId },
}

fn admit_creation_patch(
    workspace: &MutationWorkspace<'_>,
    original: &PortableRecordAspectPatch,
    kind_id: KindId,
    creation: Creation,
) -> Result<(MutationAspectPatch, RecordAspectPatchTarget), TransactionCommitError> {
    let execution = workspace.execution();
    let mut candidate =
        super::field_authoring_candidate::FieldCandidate::copy_portable(&execution, original)
            .map_err(|cause| TransactionCommitError::execution(cause.into()))?;
    let (plan, patch_target) = match creation {
        Creation::Entity => (
            workspace.entity_aspect_plan(kind_id),
            RecordAspectPatchTarget::EntityCreation { kind_id },
        ),
        Creation::Relation { source, target } => {
            let plan = workspace.relation_aspect_plan(kind_id);
            candidate = relation_endpoint_candidate::append_authoritative_endpoints(
                &execution, candidate, plan, source, target,
            )?;
            (plan, RecordAspectPatchTarget::RelationCreation { kind_id })
        }
    };
    let patch = record_aspect_patch::readmit_candidate(
        &execution,
        candidate,
        PortablePatchReadmissionPurpose::RecordCreation,
        plan,
        patch_target,
    )?;
    Ok((patch, patch_target))
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
