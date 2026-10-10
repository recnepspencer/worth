use crate::harness::fixtures::BridgeHarnessFixture;
use crate::routing::canonicalization::digest_string;

use super::churn_certification::SpeculationPreviewReplayBundleSet;

pub(super) fn preview_declaration(
    declaration_identity: crate::facade::BridgePreviewSessionDeclarationIdentity,
    binding_identity: crate::facade::BridgeSpeculativeBranchBindingIdentity,
    truth_branch_identity: crate::facade::TruthBranchIdentity,
    signal_branch_identity: crate::facade::BridgeSignalBranchIdentity,
    snapshot_identity: crate::facade::TruthSnapshotIdentity,
) -> crate::facade::BridgePreviewSessionDeclaration {
    crate::facade::BridgePreviewSessionDeclaration::new(
        declaration_identity,
        crate::facade::BridgeRequestKind::Preview,
        crate::facade::BridgeSpeculativeBranchBinding::new(
            binding_identity,
            truth_branch_identity.clone(),
            signal_branch_identity,
        ),
        crate::facade::BridgePreviewSessionBasis::new(
            crate::facade::BridgeTruthViewSelector::branch_snapshot(
                truth_branch_identity,
                snapshot_identity,
            ),
            crate::facade::BridgeSourceCapabilitySet::new(vec![
                crate::facade::BridgeSourceCapability::SnapshotRead,
                crate::facade::BridgeSourceCapability::BranchRead,
            ]),
            crate::facade::BridgePreviewRetainedArtifactSchema::PreviewLifecycleArtifactsV1,
        ),
    )
}

pub(super) fn authoritative_routing_digest(
    runtime_bridge: &crate::facade::RuntimeBridge,
    commit_identity: crate::facade::TruthCommitIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<String, super::BridgeHarnessError> {
    let result = runtime_bridge
        .deliver_invalidation(
            runtime_bridge
                .plan_committed_patch(crate::facade::BridgeRouteRequest::for_commit(
                    commit_identity.clone(),
                ), execution)
                .map_err(|error| {
                    super::BridgeHarnessError::new(format!(
                        "authoritative route planning failed during speculation certification: {error}"
                    ))
                })?,
         execution)
        .map_err(|error| {
            super::BridgeHarnessError::new(format!(
                "authoritative route delivery failed during speculation certification: {error}"
            ))
        })?;
    Ok(digest_string(
        "speculation-authoritative-routing-digest",
        result.result_summary().route_identity().as_str(),
    )
    .to_string())
}

pub(super) fn first_commit_routing_digest(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<Option<String>, super::BridgeHarnessError> {
    fixture
        .committed_patches()
        .first()
        .map(|patch| patch.commit_identity().clone())
        .map(|commit_identity| {
            authoritative_routing_digest(runtime_bridge, commit_identity, resource_request)
        })
        .transpose()
}

pub(super) fn speculative_resource_digest(
    execution_digest: &str,
    discard_digest: Option<&str>,
    promotion_digest: Option<&str>,
) -> String {
    digest_string(
        "speculative-resource-digest",
        &format!(
            "execution={execution_digest}|discard={}|promotion={}",
            discard_digest.unwrap_or("none"),
            promotion_digest.unwrap_or("none"),
        ),
    )
    .to_string()
}

pub(super) fn speculative_commit_digest(
    promoted_execution_digest: &str,
    promotion_digest: &str,
    discarded_execution_digest: &str,
    discard_digest: &str,
) -> String {
    digest_string(
        "speculative-commit-digest",
        &format!(
            "promoted-execution={promoted_execution_digest}|promotion={promotion_digest}|discarded-execution={discarded_execution_digest}|discard={discard_digest}",
        ),
    )
    .to_string()
}

pub(super) fn replay_digest(
    promoted_replay_bundle: &crate::facade::BridgePreviewReplayBundle,
    discarded_replay_bundle: &crate::facade::BridgePreviewReplayBundle,
) -> String {
    digest_string(
        "speculation-replay-digest",
        &format!(
            "promoted-replay={}|discarded-replay={}",
            promoted_replay_bundle.digest(),
            discarded_replay_bundle.digest()
        ),
    )
    .to_string()
}

pub(super) fn preview_lifecycle_digest(
    replay_bundle_set: &SpeculationPreviewReplayBundleSet,
) -> String {
    let mut basis = String::new();
    for replay_bundle in replay_bundle_set.replay_bundles() {
        if !basis.is_empty() {
            basis.push('|');
        }
        basis.push_str(replay_bundle.digest());
    }
    digest_string("preview-lifecycle-digest", &basis).to_string()
}
