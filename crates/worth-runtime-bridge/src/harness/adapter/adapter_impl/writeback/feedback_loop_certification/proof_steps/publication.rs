use super::super::*;

pub(in crate::harness::adapter::adapter_impl::writeback::feedback_loop_certification) struct FeedbackPublicationProof
{
    pub feedback_commit_identity: crate::facade::TruthCommitIdentity,
    pub carried_feedback_context: crate::facade::BridgeWritebackFeedbackContext,
    pub ordinary_commit_identity: crate::facade::TruthCommitIdentity,
    pub ordinary_route_identity: crate::facade::BridgeRouteIdentity,
    pub feedback_route_identity: crate::facade::BridgeRouteIdentity,
}

pub(in crate::harness::adapter::adapter_impl::writeback::feedback_loop_certification) fn publish_interleaved_feedback_proof(
    runtime: &crate::harness::adapter::BridgeHarnessSession,
    runtime_bridge: &crate::facade::RuntimeBridge,
    original_commit: &crate::facade::BridgeCommittedPatchEnvelope,
    feedback_context: &crate::facade::BridgeWritebackFeedbackContext,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<FeedbackPublicationProof, BridgeHarnessError> {
    let feedback_commit_identity =
        crate::truth_identity_fixtures::truth_commit_fixture("commit-feedback");
    let feedback_commit = bridge_feedback_patch(
        crate::input::envelope::BridgeCommittedPatchEnvelopeIdentity::new(
            feedback_commit_identity.clone(),
            crate::truth_identity_fixtures::truth_patch_fixture("patch-feedback"),
            original_commit.snapshot_identity().clone(),
            original_commit.branch_identity().clone(),
        ),
        feedback_context,
    );
    let carried_feedback_context = feedback_context_hint(&feedback_commit)
        .cloned()
        .ok_or_else(|| {
            BridgeHarnessError::new(
                "feedback patch did not carry first-class bridge-origin writeback context",
            )
        })?;

    let ordinary_commit_identity =
        crate::truth_identity_fixtures::truth_commit_fixture("commit-ordinary");
    runtime
        .source
        .insert_committed_patch(ordinary_truth_commit_for_feedback_interleaving(
            &ordinary_commit_identity,
            original_commit,
        ));
    let ordinary_route_identity =
        route_identity_for_commit(runtime_bridge, ordinary_commit_identity.clone(), execution)?;
    runtime.source.insert_committed_patch(feedback_commit);

    let feedback_result = runtime_bridge
        .deliver_invalidation(
            runtime_bridge
                .plan_committed_patch(
                    crate::facade::BridgeRouteRequest::for_commit(feedback_commit_identity.clone()),
                    execution,
                )
                .map_err(|error| {
                    BridgeHarnessError::new(format!(
                        "writeback feedback certification failed to plan feedback commit: {error}"
                    ))
                })?,
            execution,
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "writeback feedback certification failed to deliver feedback commit: {error}"
            ))
        })?;
    let feedback_route_identity = feedback_result.result_summary().route_identity().clone();

    Ok(FeedbackPublicationProof {
        feedback_commit_identity,
        carried_feedback_context,
        ordinary_commit_identity,
        ordinary_route_identity,
        feedback_route_identity,
    })
}

fn ordinary_truth_commit_for_feedback_interleaving(
    ordinary_commit_identity: &crate::facade::TruthCommitIdentity,
    original_commit: &crate::facade::BridgeCommittedPatchEnvelope,
) -> crate::facade::BridgeCommittedPatchEnvelope {
    crate::facade::BridgeCommittedPatchEnvelope::new(
        crate::input::envelope::BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
            crate::facade::BridgeProducerMetadata::bridge_harness_fixture(),
            ordinary_commit_identity.clone(),
            crate::truth_identity_fixtures::truth_patch_fixture("patch-ordinary"),
            original_commit.snapshot_identity().clone(),
            original_commit.branch_identity().clone(),
        ),
        vec![crate::facade::BridgeCommittedPatchItem::with_target(
            "user",
            crate::facade::BridgeCommittedPatchTarget::entity_field_path(
                worth_foundational::facade::AspectLocator::new(
                    worth_foundational::facade::LocatorAuthority::Authoritative,
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid bridge patch aspect key"),
                ),
                worth_foundational::facade::CanonicalFieldPath::single(
                    worth_foundational::facade::FieldKey::new("name".to_owned())
                        .expect("valid foundational field key"),
                ),
            ),
        )],
    )
    .expect("writeback harness committed patch envelope should construct")
}
