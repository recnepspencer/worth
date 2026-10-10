use crate::harness::adapter::types::BridgeHarnessError;
use crate::harness::fixtures::BridgeHarnessFixture;
use crate::routing::canonicalization::digest_string;

use super::certification_digest_basis::semantic_route_planning_policy_digest;
use super::matrices::{
    AdmittedPolicyMatrixRow, AdmittedPolicyMatrixRowEvidence, PolicyRejectionMatrixRow,
    PolicyRejectionMatrixRowEvidence, RequestPolicyMatrixRow, RoutePolicyMatrixRow,
    RoutePolicyMatrixRowEvidence,
};

pub(super) struct AdmittedPolicyBundle {
    pub(super) contract: crate::facade::AdmittedBridgePolicyContract,
    pub(super) lowered: crate::facade::LoweredBridgeExecutionPolicy,
    pub(super) provenance: crate::facade::BridgePolicyProvenanceRecord,
    pub(super) replay_bundle: crate::facade::BridgePolicyReplayBundle,
    pub(super) route_policy: crate::facade::BridgeRoutePlanningPolicy,
}

pub(super) fn admitted_policy_bundle(
    runtime_bridge: &crate::facade::RuntimeBridge,
    declaration: crate::facade::BridgePolicyDeclaration,
) -> Result<AdmittedPolicyBundle, BridgeHarnessError> {
    let contract = runtime_bridge
        .admit_policy_declaration(declaration)
        .map_err(|rejection| {
            BridgeHarnessError::new(format!("policy admission failed: {rejection:?}"))
        })?;
    let lowered = runtime_bridge.lower_admitted_policy(&contract);
    let provenance = runtime_bridge.canonicalize_policy_provenance(&contract, &lowered);
    let replay_bundle = runtime_bridge.replay_policy_bundle(&contract, &lowered, &provenance);
    let route_policy = runtime_bridge
        .project_route_planning_policy(&lowered)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "route planning policy projection failed during certification: {error}"
            ))
        })?;
    Ok(AdmittedPolicyBundle {
        contract,
        lowered,
        provenance,
        replay_bundle,
        route_policy,
    })
}

pub(super) fn rejected_policy_bundle(
    runtime_bridge: &crate::facade::RuntimeBridge,
    declaration: crate::facade::BridgePolicyDeclaration,
) -> Result<crate::facade::BridgePolicyRejection, BridgeHarnessError> {
    match runtime_bridge.admit_policy_declaration(declaration) {
        Ok(_) => Err(BridgeHarnessError::new(
            "policy rejection certification unexpectedly admitted declaration",
        )),
        Err(rejection) => Ok(rejection),
    }
}

pub(super) fn rejection_policy_row(
    label: &str,
    rejection: &crate::facade::BridgePolicyRejection,
) -> PolicyRejectionMatrixRow {
    PolicyRejectionMatrixRow::from_evidence(PolicyRejectionMatrixRowEvidence {
        label: label.to_string(),
        declaration_identity: rejection.declaration_identity().clone(),
        failure_kind: rejection.kind(),
        stage: rejection.stage(),
        field_kind: rejection.field_kind(),
        primary_source: rejection.primary_source(),
        secondary_source: rejection.conflicting_source(),
        digest: rejection.digest().to_string(),
    })
}

pub(super) fn admitted_policy_row(
    label: &str,
    bundle: &AdmittedPolicyBundle,
) -> AdmittedPolicyMatrixRow {
    AdmittedPolicyMatrixRow::from_evidence(AdmittedPolicyMatrixRowEvidence {
        label: label.to_string(),
        declaration_identity: bundle
            .contract
            .validated_declaration()
            .declaration()
            .declaration_identity()
            .clone(),
        request_kind: bundle
            .contract
            .validated_declaration()
            .declaration()
            .request_kind(),
        execution_class: bundle.contract.resolved_execution_class(),
        diagnostics_tier: bundle.contract.resolved_diagnostics_tier(),
        route_artifacts: bundle.contract.resolved_route_artifacts(),
        replay_artifacts: bundle.contract.resolved_replay_artifacts(),
        policy_digest: bundle.contract.digest().to_string(),
        lowered_policy_digest: bundle.lowered.digest().to_string(),
        provenance_digest: bundle.provenance.digest().to_string(),
        replay_digest: bundle.replay_bundle.digest().to_string(),
    })
}

pub(super) fn request_policy_row(
    provenance_row: crate::facade::BridgePolicyProvenanceReportRow,
    bundle: &AdmittedPolicyBundle,
) -> RequestPolicyMatrixRow {
    RequestPolicyMatrixRow::new(
        provenance_row,
        bundle.route_policy.digest(),
        semantic_route_planning_policy_digest(&bundle.route_policy),
    )
}

pub(super) fn route_policy_row(label: &str, bundle: &AdmittedPolicyBundle) -> RoutePolicyMatrixRow {
    RoutePolicyMatrixRow::from_evidence(RoutePolicyMatrixRowEvidence {
        label: label.to_string(),
        route_planning_policy_digest: bundle.route_policy.digest().to_string(),
        semantic_route_planning_policy_digest: semantic_route_planning_policy_digest(
            &bundle.route_policy,
        ),
        lowered_policy_identity: bundle.route_policy.lowered_policy_identity().clone(),
        execution_class: bundle.route_policy.execution_class(),
        diagnostics_tier: bundle.route_policy.diagnostics_tier(),
        route_artifacts: bundle.route_policy.route_artifacts(),
        replay_artifacts: bundle.route_policy.replay_artifacts(),
    })
}

pub(super) fn combined_counter_snapshot<const N: usize>(
    bundles: [&AdmittedPolicyBundle; N],
    policy_request_count: usize,
    truth_view_interleave_count: usize,
    preview_equivalence_preserved_count: usize,
    ambient_policy_leak_count: usize,
    replay_mismatch_count: usize,
) -> crate::facade::BridgePolicyCounters {
    let declarations = bundles
        .iter()
        .map(|bundle| bundle.contract.validated_declaration().declaration())
        .collect::<Vec<_>>();
    let contracts = bundles
        .iter()
        .map(|bundle| &bundle.contract)
        .collect::<Vec<_>>();
    let provenances = bundles
        .iter()
        .map(|bundle| &bundle.provenance)
        .collect::<Vec<_>>();
    let replay_bundles = bundles
        .iter()
        .map(|bundle| &bundle.replay_bundle)
        .collect::<Vec<_>>();

    crate::facade::BridgePolicyCounters::from_admitted_artifacts(
        declarations.as_slice(),
        contracts.as_slice(),
        provenances.as_slice(),
        replay_bundles.as_slice(),
        0,
        replay_mismatch_count,
        ambient_policy_leak_count,
        policy_request_count,
        truth_view_interleave_count,
        preview_equivalence_preserved_count,
        0,
        0,
        0,
    )
}

pub(super) fn first_commit_routing_digest(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    route_policy: &crate::facade::BridgeRoutePlanningPolicy,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<Option<String>, BridgeHarnessError> {
    fixture
        .committed_patches()
        .first()
        .map(|patch| {
            runtime_bridge
                .deliver_invalidation(
                    runtime_bridge
                        .plan_committed_patch_with_route_policy(
                            crate::facade::BridgeRouteRequest::for_commit(
                                patch.commit_identity().clone(),
                            ),
                            route_policy,
                            execution,
                        )
                        .map_err(|error| {
                            BridgeHarnessError::new(format!(
                                "policy certification route planning failed: {error}"
                            ))
                        })?,
                    execution,
                )
                .map_err(|error| {
                    BridgeHarnessError::new(format!(
                        "policy certification route delivery failed: {error}"
                    ))
                })
                .map(|result| {
                    digest_string(
                        "policy-certification-routing",
                        result.result_summary().route_identity().as_str(),
                    )
                    .to_string()
                })
        })
        .transpose()
}

pub(super) fn first_snapshot_identity(
    fixture: &BridgeHarnessFixture,
) -> crate::facade::TruthSnapshotIdentity {
    fixture
        .snapshots()
        .first()
        .map(|snapshot| snapshot.identity().clone())
        .unwrap_or_else(|| crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"))
}
