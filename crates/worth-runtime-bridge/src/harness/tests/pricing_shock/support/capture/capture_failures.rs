use super::*;
use crate::facade::{BridgePolicyFieldKind, BridgePolicyRejectionKind, BridgeRouteErrorKind};

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_restart_replay_bundle(
    policy: BridgeRuntimePolicy,
    truth: &RelationalPricing,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingRestartReplayBundle {
    let original_runtime = truth.runtime_with_policy(RecordingSignalBridgeSink::default(), policy);
    original_runtime
        .route(truth.steel_commit.clone(), execution)
        .expect("pricing restart control route should succeed");
    let canonical_record = original_runtime
        .diagnostics()
        .last_canonical_route_record()
        .expect("pricing restart replay should retain a canonical route record");

    let restarted_runtime = truth.runtime_with_policy(RecordingSignalBridgeSink::default(), policy);
    let replay = restarted_runtime
        .replay_canonical_record(&canonical_record, execution)
        .expect("pricing restart replay should preserve canonical truth across rebuild");

    PricingRestartReplayBundle {
        source_commit: replay.source_commit().clone(),
        source_snapshot: replay.source_snapshot().clone(),
        route_identity: replay.route_identity().clone(),
        invalidation_identity: replay.invalidation_identity().clone(),
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_restart_failure_bundle(
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingRestartFailureBundle {
    let truth = RelationalPricing::new(execution, 100, 40, 56);
    let original_runtime = truth.runtime(RecordingSignalBridgeSink::default());
    original_runtime
        .route(truth.steel_commit.clone(), execution)
        .expect("pricing restart mismatch control route should succeed");
    let canonical_record = original_runtime
        .diagnostics()
        .last_canonical_route_record()
        .expect("pricing restart mismatch should retain a canonical route record");

    let restarted_runtime = truth.fault_runtime(
        PricingFault::PatchItems(
            truth.steel_commit.clone(),
            vec![truth.cost_item(PricingMaterial::Rubber)],
        ),
        BridgeRuntimePolicy::development(),
    );

    let error = restarted_runtime
        .replay_canonical_record(&canonical_record, execution)
        .expect_err("pricing restart replay should reject route drift after truth change");
    let failure_record = restarted_runtime
        .diagnostics()
        .last_failure_record()
        .expect("pricing restart replay mismatch should retain a failure record");

    PricingRestartFailureBundle {
        error_kind: error.kind(),
        replay_mismatch_count: failure_record.counters().route_replay_mismatch_count(),
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_replay_policy_failure_bundle(
    truth: &RelationalPricing,
) -> (BridgePolicyRejectionKind, BridgePolicyFieldKind) {
    let runtime = truth.runtime_with_policy(
        RecordingSignalBridgeSink::default(),
        BridgeRuntimePolicy::operational().with_replay_artifacts(false),
    );
    let declaration = BridgePolicyDeclaration::new(
        BridgePolicyDeclarationIdentity::admit_bridge_owned(
            "policy:pricing-showcase-replay-conflict",
        ),
        BridgeRequestKind::Preview,
        BridgeExecutionPolicyClass::Optimized,
        BridgeDiagnosticsTier::Minimal,
        true,
        false,
    );
    let rejection = runtime
        .admit_policy_declaration(declaration)
        .expect_err("replay requirement should fail when runtime policy disables replay");

    (rejection.kind(), rejection.field_kind())
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_route_policy_conflict_bundle(
    truth: &RelationalPricing,
) -> BridgeRouteErrorKind {
    let permissive = truth.runtime_with_policy(
        RecordingSignalBridgeSink::default(),
        BridgeRuntimePolicy::development(),
    );
    let restrictive = truth.runtime_with_policy(
        RecordingSignalBridgeSink::default(),
        BridgeRuntimePolicy::operational().with_replay_artifacts(false),
    );
    let declaration = BridgePolicyDeclaration::new(
        BridgePolicyDeclarationIdentity::admit_bridge_owned("policy:pricing-route-policy-conflict"),
        BridgeRequestKind::Authoritative,
        BridgeExecutionPolicyClass::DeterministicCanonical,
        BridgeDiagnosticsTier::Standard,
        true,
        true,
    );
    let admitted = permissive
        .admit_policy_declaration(declaration)
        .expect("permissive runtime should admit replay-capable route policy");
    let lowered = permissive.lower_admitted_policy(&admitted);
    let error = restrictive
        .project_route_planning_policy(&lowered)
        .expect_err("restrictive runtime should reject divergent route policy");

    error.kind()
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_trust_attack_bundle(
    truth: &RelationalPricing,
) -> PricingTrustAttackBundle {
    let (replay_policy_error_kind, replay_policy_failure_class) =
        capture_pricing_replay_policy_failure_bundle(truth);
    let route_policy_error_kind = capture_pricing_route_policy_conflict_bundle(truth);

    PricingTrustAttackBundle {
        replay_policy_error_kind,
        replay_policy_failure_class,
        route_policy_error_kind,
    }
}

use crate::adapter::{
    CommittedPatchSource, RelationalBridgeSourceError, RelationalBridgeSourceErrorTag,
    RelationalCommittedPatchRequest, SnapshotReadSource, TruthBranchHeadSource,
};
use crate::facade::{
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchItem, BridgeSnapshotReadError,
    SnapshotReadPacket, SnapshotReadPacketResult, TruthBranchIdentity, TruthCommitIdentity,
    TruthSnapshotIdentity, TruthSnapshotReader,
};
use crate::input::envelope::BridgeCommittedPatchEnvelopeIdentity;
use worth_execution::ExecutionRequest;

#[derive(Clone)]
pub(in crate::harness::tests::pricing_shock) enum PricingFault {
    PatchItems(TruthCommitIdentity, Vec<BridgeCommittedPatchItem>),
    CommitSnapshot(TruthCommitIdentity, TruthSnapshotIdentity),
    SwapHead(TruthBranchIdentity, TruthBranchIdentity),
    MissingHead(TruthBranchIdentity, String),
    UnavailableSnapshot(TruthSnapshotIdentity),
    SwapSnapshotContents(TruthSnapshotIdentity, TruthSnapshotIdentity),
}

#[derive(Clone)]
pub(in crate::harness::tests::pricing_shock::support) struct PricingFaultSource<S>(
    pub S,
    pub PricingFault,
);

impl<S: CommittedPatchSource> CommittedPatchSource for PricingFaultSource<S> {
    fn load_committed_patch(
        &self,
        request: RelationalCommittedPatchRequest,
        execution: ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        let original = self.0.load_committed_patch(request, execution)?;
        let (snapshot, items) = match &self.1 {
            PricingFault::CommitSnapshot(commit, replacement)
                if commit == original.commit_identity() =>
            {
                (
                    replacement.clone(),
                    original.patch_body().canonical_items().to_vec(),
                )
            }
            PricingFault::PatchItems(commit, replacement)
                if commit == original.commit_identity() =>
            {
                (original.snapshot_identity().clone(), replacement.clone())
            }
            _ => return Ok(original),
        };

        Ok(BridgeCommittedPatchEnvelope::new(
            BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
                original.producer_metadata().clone(),
                original.commit_identity().clone(),
                original.patch_identity().clone(),
                snapshot,
                original.branch_identity().clone(),
            ),
            items,
        )
        .expect("a fault only alters the selected committed envelope component"))
    }
}

impl<S: TruthBranchHeadSource> TruthBranchHeadSource for PricingFaultSource<S> {
    fn load_branch_head_patch(
        &self,
        request: &TruthBranchIdentity,
        execution: ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        let original = self.0.load_branch_head_patch(request, execution)?;
        match &self.1 {
            PricingFault::SwapHead(branch, other) if branch == request => {
                self.0.load_branch_head_patch(other, execution)
            }
            PricingFault::MissingHead(branch, commit) if branch == request => {
                Err(RelationalBridgeSourceError::new(
                    RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                    format!("no committed envelope for branch-head commit {commit}"),
                ))
            }
            _ => Ok(original),
        }
    }
}

impl<S: SnapshotReadSource> SnapshotReadSource for PricingFaultSource<S> {
    fn open_snapshot(
        &self,
        request: &TruthSnapshotIdentity,
        execution: ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        let original = self.0.open_snapshot(request, execution)?;
        match &self.1 {
            PricingFault::UnavailableSnapshot(snapshot) if snapshot == request => {
                Err(RelationalBridgeSourceError::new(
                    RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                    "injected retained pricing snapshot unavailability",
                ))
            }
            PricingFault::SwapSnapshotContents(snapshot, replacement) if snapshot == request => {
                Ok(Box::new(MislabelledSnapshot {
                    identity: snapshot.clone(),
                    contents: self.0.open_snapshot(replacement, execution)?,
                }))
            }
            _ => Ok(original),
        }
    }
}

struct MislabelledSnapshot {
    identity: TruthSnapshotIdentity,
    contents: Box<dyn TruthSnapshotReader>,
}

impl TruthSnapshotReader for MislabelledSnapshot {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        self.identity.clone()
    }

    fn read_packet(
        &self,
        packet: &SnapshotReadPacket,
        execution: ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        let result = self.contents.read_packet(packet, execution)?;
        Ok(SnapshotReadPacketResult::new(
            self.identity.clone(),
            result.records().to_vec(),
        ))
    }
}
