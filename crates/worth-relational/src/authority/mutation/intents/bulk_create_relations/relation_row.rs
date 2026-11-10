use super::*;

pub(super) fn apply_relation_row<'a>(
    intent: &'a BulkRelationCreateIntent,
    workspace: &mut MutationWorkspace<'a>,
    outcome: &mut MutationOutcome,
    version_id: crate::identity::data::VersionId,
    row_index: usize,
    source: &EntityReference,
    target: &EntityReference,
    fields: &AspectFieldPatch,
) -> Result<(), TransactionCommitError> {
    let source_id = resolve_entity_reference(workspace, source)?;
    let target_id = resolve_entity_reference(workspace, target)?;
    let patch_target = RecordAspectPatchTarget::RelationCreation {
        kind_id: intent.kind_id,
    };
    let plan = workspace.relation_aspect_plan(intent.kind_id);
    let candidate = field_authoring_candidate::lower(
        &workspace.execution(),
        fields,
        PortablePatchReadmissionPurpose::RecordCreation,
        plan,
        intent.kind_id,
        FieldAuthoringDomain::Relation,
    )
    .map_err(|denial| record_aspect_patch::MutationPatchDenial::candidate(patch_target, denial))?;
    let candidate = relation_endpoint_candidate::append_authoritative_endpoints(
        &workspace.execution(),
        candidate,
        plan,
        source_id,
        target_id,
    )?;
    let authoritative_patch = record_aspect_patch::readmit_candidate(
        &workspace.execution(),
        candidate,
        PortablePatchReadmissionPurpose::RecordCreation,
        plan,
        patch_target,
    )?;
    let authoritative_aspect_state = record_aspect_patch::apply_relation(
        &workspace.execution(),
        None,
        &authoritative_patch,
        patch_target,
    )?;
    let relation_id = workspace.with_context(|context| {
        let relation_id = allocate_relation(
            &context.execution,
            context.state,
            context.record_allocations,
            version_id,
            intent.partition_id,
            intent.kind_id,
            source_id,
            target_id,
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
    if let Some(client_key) = intent.client_keys.get(row_index) {
        // Both staging adapters emit canonical ImportReductionKey row order.
        // Binding keys borrow that SAME original row instead of cloned packets.
        let (original_source,original_target)=intent.endpoints.get(row_index)
            .expect(/* Proof: Only the leased and ordered staging adapters call this private row function, each with the ordinal of the intent endpoint row it staged. */ "row ordinal belongs to its original endpoints");
        workspace.register_created_relation(
            intent.partition_id,
            intent.kind_id,
            client_key,
            original_source,
            original_target,
            relation_id,
        )?;
    }
    outcome.record_change(
        &workspace.execution(),
        RecordMutation::RelationCreated {
            relation_id,
            kind_id: intent.kind_id,
            source: source_id,
            target: target_id,
            authoritative_patch: record_aspect_patch::published_patch(authoritative_patch),
        },
    )?;
    Ok(())
}
