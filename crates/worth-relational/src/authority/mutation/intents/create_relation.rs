use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::record_changes::allocate_relation;
use crate::authority::mutation::MutationWorkspace;
use crate::transactions::data::{
    CommitConflict, ConflictClass, CreatedRelationRef, EntityReference, RecordAspectPatchTarget,
    RelationSpec,
};
use worth_foundational::facade::PortablePatchReadmissionPurpose;

use super::field_authoring_candidate::{self, FieldAuthoringDomain};
use super::{record_aspect_patch, relation_endpoint_candidate};

pub(super) fn apply(
    spec: &RelationSpec,
    workspace: &mut MutationWorkspace<'_>,
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
            .mark_relation_slot_touched(relation_id.partition_id, relation_id.slot_index());
        Ok::<_, CommitConflict>(relation_id)
    })?;
    workspace.register_created_relation(
        CreatedRelationRef {
            partition_id: spec.partition_id,
            kind_id: spec.kind_id,
            client_key: spec.client_key.clone(),
            source: spec.source.clone(),
            target: spec.target.clone(),
        },
        relation_id,
    );
    Ok(MutationOutcome::relation_created(
        relation_id,
        source,
        target,
        spec.kind_id,
        record_aspect_patch::published_patch(patch),
    ))
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
