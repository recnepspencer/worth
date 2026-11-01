use worth_foundational::facade::PortablePatchReadmissionPurpose;

use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::record_changes::allocate_relation;
use crate::authority::mutation::MutationWorkspace;
use crate::transactions::data::{
    CommitConflict, ConflictClass, CreatedRelationRef, EntityReference, RecordAspectPatchTarget,
    RelationAspectCreateIntent,
};

use super::{record_aspect_patch, relation_endpoint_candidate};

pub(super) fn apply(
    intent: &RelationAspectCreateIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, crate::transactions::data::TransactionCommitError> {
    let source = resolve_endpoint(workspace, &intent.source, "source")?;
    let target_entity = resolve_endpoint(workspace, &intent.target, "target")?;
    let target = RecordAspectPatchTarget::RelationCreation {
        kind_id: intent.kind_id,
    };
    let plan = workspace.relation_aspect_plan(intent.kind_id);
    let candidate = relation_endpoint_candidate::append_authoritative_endpoints(
        intent.aspect_patch.clone(),
        plan,
        source,
        target_entity,
    );
    let patch = record_aspect_patch::readmit(
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
            .mark_relation_slot_touched(relation_id.partition_id, relation_id.slot_index());
        Ok::<_, CommitConflict>(relation_id)
    })?;
    workspace.register_created_relation(
        CreatedRelationRef {
            partition_id: intent.partition_id,
            kind_id: intent.kind_id,
            client_key: intent.client_key.clone(),
            source: intent.source.clone(),
            target: intent.target.clone(),
        },
        relation_id,
    );
    Ok(MutationOutcome::relation_created(
        relation_id,
        source,
        target_entity,
        intent.kind_id,
        record_aspect_patch::published_patch(patch),
    ))
}

fn resolve_endpoint(
    workspace: &MutationWorkspace<'_>,
    reference: &EntityReference,
    label: &str,
) -> Result<crate::identity::data::EntityId, CommitConflict> {
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
