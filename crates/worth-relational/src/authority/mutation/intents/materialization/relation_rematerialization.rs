use super::*;

pub(super) fn rematerialize_relation(
    intent: &crate::transactions::data::RematerializeRelationIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, crate::transactions::data::TransactionCommitError> {
    let target_record = RecordAspectPatchTarget::RelationCreation {
        kind_id: intent.kind_id,
    };
    let plan = workspace.relation_aspect_plan(intent.kind_id);
    let candidate = field_authoring_candidate::lower(
        &workspace.execution(),
        &intent.fields,
        PortablePatchReadmissionPurpose::RecordCreation,
        plan,
        intent.kind_id,
        FieldAuthoringDomain::Relation,
    )
    .map_err(|denial| record_aspect_patch::MutationPatchDenial::candidate(target_record, denial))?;
    let candidate = relation_endpoint_candidate::append_authoritative_endpoints(
        &workspace.execution(),
        candidate,
        plan,
        intent.source,
        intent.target,
    )?;
    let patch = record_aspect_patch::readmit_candidate(
        &workspace.execution(),
        candidate,
        PortablePatchReadmissionPurpose::RecordCreation,
        plan,
        target_record,
    )?;
    let mut state =
        record_aspect_patch::apply_relation(&workspace.execution(), None, &patch, target_record)?;
    let version_id = workspace.version_id();
    workspace.with_context(|context| {
        let partition = context
            .state
            .get_partition_mut_for_request(intent.relation_id.partition_id, &context.execution)
            .map_err(|cause| {
                crate::transactions::data::TransactionCommitError::execution(cause.into())
            })?;
        partition
            .relation_arena
            .rematerialize_for_request(
                &context.execution,
                &intent.relation_id,
                intent.kind_id,
                version_id,
                {
                    state.endpoints = Some(RelationEndpoints {
                        source: intent.source,
                        target: intent.target,
                    });
                    state
                },
            )
            .map_err(|denial| {
                let detail = match denial {
                    crate::storage::substrate::MetadataWriteDenial::Request(cause) => {
                        return crate::transactions::data::TransactionCommitError::execution(
                            cause.into(),
                        )
                    }
                    crate::storage::substrate::MetadataWriteDenial::Invalid(detail) => detail,
                };
                materialization_conflict(
                    crate::transactions::data::RecordRef::Relation(intent.relation_id),
                    detail,
                )
                .into()
            })?;
        context
            .state
            .mark_relation_slot_touched_for_request(
                intent.relation_id.partition_id,
                intent.relation_id.slot_index(),
                &context.execution,
            )
            .map_err(|cause| {
                crate::transactions::data::TransactionCommitError::execution(cause.into())
            })?;
        Ok::<_, crate::transactions::data::TransactionCommitError>(())
    })?;
    let state = workspace.with_context(|context| {
        context.state.retained_relation_extra(
            intent.relation_id.partition_id,
            intent.relation_id.slot_index(),
        )
    });
    let mut outcome = MutationOutcome::with_capacity(&workspace.execution(), 1, 1)?;
    outcome.record_change(
        &workspace.execution(),
        RecordMutation::RelationRematerialized {
            relation_id: intent.relation_id,
            kind_id: intent.kind_id,
            source: intent.source,
            target: intent.target,
            new_authoritative_aspect_state: state,
            authoritative_patch: record_aspect_patch::published_patch(patch),
        },
    )?;
    outcome.record_event(
        &workspace.execution(),
        MutationEvent::RelationUpdated {
            relation_id: intent.relation_id,
        },
    )?;
    Ok(outcome)
}
