use super::retained_mapping_support::{
    binding_for, bridge_bulk_planning_reference, bridge_route_reference, missing_bridge_reference,
    query_observation_reference,
};
use super::{runtime, BridgeRuntimePolicy};
use crate::facade::{
    BridgeBulkWorkloadRequest, BridgeBulkWorkloadSegment, BridgeCanonicalBulkPlanRecord,
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEnvelopeDenialKind,
    BridgeCausalEvidenceFamily, BridgeRouteRequest,
};

#[test]
fn causal_envelope_maps_bulk_planning_by_exact_workload_identity() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let routed = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit-causal-bulk-route"),
            execution,
        )
        .expect("route should succeed");
    let bulk_record = retain_bulk_record(&runtime, "target", execution);
    assert_eq!(bulk_record.planning_failure_count(), 1);
    assert!(bulk_record
        .planning_failures()
        .iter()
        .all(|failure| !failure.digest().is_empty()));
    let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:bulk-planning",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:bulk-planning",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            query_observation_reference(
                crate::facade::BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:bulk-planning",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_route_reference(routed.result().result_summary()),
            bridge_bulk_planning_reference(&bulk_record),
        ],
    )
    .expect("request should be valid");

    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect("bulk planning record should bind");

    assert_eq!(envelope.counters().bridge_retained_lookup_count(), 2);
    assert_eq!(envelope.counters().retained_bridge_binding_count(), 2);
    assert_eq!(envelope.counters().bridge_record_unindexed_scan_count(), 0);
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceFamily::BridgeBulkPlanning,
            bulk_record.workload_identity().as_str()
        )
        .retained_record_digest_for_reporting(),
        Some(bulk_planning_digest(&bulk_record).as_str())
    );
}

#[test]
fn causal_envelope_denies_missing_bulk_planning_without_unindexed_scan() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let routed = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit-causal-missing-bulk"),
            execution,
        )
        .expect("route should succeed");
    let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:missing-bulk",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:missing-bulk",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            query_observation_reference(
                crate::facade::BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:missing-bulk",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_route_reference(routed.result().result_summary()),
            missing_bridge_reference(
                BridgeCausalEvidenceFamily::BridgeBulkPlanning,
                "missing-bulk-workload",
            ),
        ],
    )
    .expect("request should be valid");

    let denial = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect_err("missing bulk planning record should deny");

    assert_eq!(
        denial.kind(),
        BridgeCausalEnvelopeDenialKind::MissingRetainedBridgeRecord
    );
    assert_eq!(
        denial.family(),
        BridgeCausalEvidenceFamily::BridgeBulkPlanning
    );
    assert_eq!(denial.counters().bridge_retained_lookup_count(), 2);
    assert_eq!(denial.counters().retained_bridge_binding_count(), 1);
    assert_eq!(denial.counters().missing_bridge_record_count(), 1);
    assert_eq!(denial.counters().bridge_record_unindexed_scan_count(), 0);
}

#[test]
fn causal_envelope_bulk_planning_lookup_cost_ignores_unrelated_records() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut envelope_identities = Vec::new();

    for unrelated_records in [0, 3, 8] {
        let runtime = runtime(BridgeRuntimePolicy::default());
        for index in 0..unrelated_records {
            retain_bulk_record(&runtime, &format!("noise-{index}"), execution);
        }
        let target_record = retain_bulk_record(&runtime, "target", execution);
        let routed = runtime
            .route(
                crate::truth_identity_fixtures::truth_commit_fixture("commit-causal-bulk-scale"),
                execution,
            )
            .expect("route should succeed");
        let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
            crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
                crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                    "query-admission:bulk-scale",
                ),
                crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                    "causal-anchor:bulk-scale",
                ),
            )
            .expect("query admission summary should be valid"),
            vec![
                query_observation_reference(
                    crate::facade::BridgeCausalEvidenceReferenceIdentity::query_observation(
                        crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                            "query-observation:bulk-scale",
                        ),
                    )
                    .expect("query observation reference identity should be valid"),
                ),
                bridge_route_reference(routed.result().result_summary()),
                bridge_bulk_planning_reference(&target_record),
            ],
        )
        .expect("request should be valid");

        let envelope = runtime
            .diagnostics()
            .assemble_causal_explanation_envelope(request)
            .expect("target bulk planning should bind");

        assert_eq!(
            runtime.diagnostics().bulk_records().len(),
            unrelated_records + 1
        );
        assert_eq!(envelope.counters().bridge_retained_lookup_count(), 2);
        assert_eq!(envelope.counters().retained_bridge_binding_count(), 2);
        assert_eq!(envelope.counters().bridge_record_unindexed_scan_count(), 0);
        envelope_identities.push(
            envelope
                .identity()
                .envelope_identity_for_reporting()
                .to_string(),
        );
    }

    assert_eq!(envelope_identities[0], envelope_identities[1]);
    assert_eq!(envelope_identities[1], envelope_identities[2]);
}

fn retain_bulk_record(
    runtime: &crate::facade::RuntimeBridge,
    suffix: &str,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> BridgeCanonicalBulkPlanRecord {
    let workload = BridgeBulkWorkloadRequest::new(vec![
        BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
            crate::truth_identity_fixtures::truth_commit_fixture(format!("commit-bulk-{suffix}-a")),
        )),
        BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
            crate::truth_identity_fixtures::truth_commit_fixture(format!("commit-bulk-{suffix}-b")),
        )),
    ]);
    let plan = runtime
        .plan_bulk_workload(workload, execution)
        .expect("bulk workload should plan");
    runtime.canonicalize_bulk_workload_plan(&plan)
}

fn bulk_planning_digest(record: &BridgeCanonicalBulkPlanRecord) -> String {
    crate::diagnostics::causal_envelope::retained_mapping::retained_artifact_digest::planning_checkpoint::bulk_planning_digest(record)
        .as_str()
        .to_string()
}
