use super::retained_mapping_support::{
    binding_for, bridge_route_reference, bridge_source_failure_reference,
    bridge_source_materialization_reference, missing_bridge_reference, query_observation_reference,
};
use super::{
    registered_source, runtime, runtime_with_source_adapter, BridgeRuntimePolicy,
    BridgeSourceCapability, BridgeTruthViewSelector, RejectingSourceAdapter, SnapshotReadPacket,
};
use crate::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEnvelopeDenialKind, BridgeCausalEvidenceFamily,
};

#[test]
fn causal_envelope_maps_source_failure_by_exact_failure_identity() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime =
        runtime_with_source_adapter(BridgeRuntimePolicy::default(), RejectingSourceAdapter);
    let routed = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit-causal-source-failure"),
            execution,
        )
        .expect("route should succeed");
    let contract = runtime
        .admit_source(registered_source(
            "source:analysis-history",
            BridgeTruthViewSelector::historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            ),
            vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::HistoricalRead,
                BridgeSourceCapability::BranchRead,
                BridgeSourceCapability::ReplayContinuityRead,
            ],
        ))
        .expect("source should admit");
    assert!(
        runtime
            .materialize_source_packet(&contract, SnapshotReadPacket::new(vec![]), execution)
            .is_err(),
        "rejecting adapter should produce a retained source failure"
    );
    let failure = runtime
        .diagnostics()
        .last_source_failure_record()
        .expect("source failure should be retained");

    let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:source-failure",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:source-failure",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            query_observation_reference(
                crate::facade::BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:source-failure",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_route_reference(routed.result().result_summary()),
            bridge_source_failure_reference(&failure),
        ],
    )
    .expect("request should be valid");
    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect("source failure should bind by exact identity");
    let expected_failure_digest =
        crate::diagnostics::causal_envelope::retained_mapping::retained_artifact_digest::source_structural_stream::source_failure_digest(&failure);

    assert_eq!(envelope.counters().bridge_retained_lookup_count(), 2);
    assert_eq!(envelope.counters().retained_bridge_binding_count(), 2);
    assert_eq!(envelope.counters().bridge_record_unindexed_scan_count(), 0);
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceFamily::BridgeSourceFailure,
            failure.failure_identity().as_str()
        )
        .retained_record_digest_for_reporting(),
        Some(expected_failure_digest.as_str())
    );
}

#[test]
fn causal_envelope_denies_missing_retained_expansion_record_without_unindexed_scan() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let routed = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-causal-missing-retained-expansion",
            ),
            execution,
        )
        .expect("route should succeed");
    let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:missing-retained-expansion",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:missing-retained-expansion",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            query_observation_reference(
                crate::facade::BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:missing-retained-expansion",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_route_reference(routed.result().result_summary()),
            missing_bridge_reference(
                BridgeCausalEvidenceFamily::BridgeStructuralBranchComparison,
                "missing-branch-comparison-record",
            ),
        ],
    )
    .expect("request should be valid");

    let denial = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect_err("missing retained record should deny");

    assert_eq!(
        denial.kind(),
        BridgeCausalEnvelopeDenialKind::MissingRetainedBridgeRecord
    );
    assert_eq!(
        denial.family(),
        BridgeCausalEvidenceFamily::BridgeStructuralBranchComparison
    );
    assert_eq!(denial.counters().bridge_retained_lookup_count(), 2);
    assert_eq!(denial.counters().retained_bridge_binding_count(), 1);
    assert_eq!(denial.counters().missing_bridge_record_count(), 1);
    assert_eq!(denial.counters().bridge_record_unindexed_scan_count(), 0);
}

#[test]
fn causal_envelope_source_materialization_lookup_cost_ignores_unrelated_records() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut envelope_identities = Vec::new();

    for unrelated_records in [0, 3, 8] {
        let runtime = runtime(BridgeRuntimePolicy::default());
        let contract = runtime
            .admit_source(registered_source(
                "source:analysis-history",
                BridgeTruthViewSelector::historical_commit(
                    crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                ),
                vec![
                    BridgeSourceCapability::SnapshotRead,
                    BridgeSourceCapability::HistoricalRead,
                    BridgeSourceCapability::BranchRead,
                    BridgeSourceCapability::ReplayContinuityRead,
                ],
            ))
            .expect("source should admit");
        for index in 0..unrelated_records {
            let packet =
                SnapshotReadPacket::new(vec![crate::snapshot::SnapshotReadRequest::for_coarse(
                    format!("entity-noise-{index}"),
                    crate::snapshot::SnapshotReadContract::scalar(
                        worth_foundational::facade::AspectKey::new("profile")
                            .expect("valid snapshot aspect key"),
                        worth_foundational::facade::ScalarAspectType::String,
                    ),
                )]);
            let observation = runtime
                .materialize_source_packet(&contract, packet, execution)
                .expect("noise source should materialize");
            runtime
                .canonicalize_source_materialization_record(&contract, &observation)
                .expect("noise source should canonicalize");
        }
        let routed = runtime
            .route(
                crate::truth_identity_fixtures::truth_commit_fixture("commit-causal-source-scale"),
                execution,
            )
            .expect("route should succeed");
        let target_observation = runtime
            .materialize_source_packet(&contract, SnapshotReadPacket::new(vec![]), execution)
            .expect("target source should materialize");
        let target_record = runtime
            .canonicalize_source_materialization_record(&contract, &target_observation)
            .expect("target source should canonicalize");
        let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
            crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
                crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                    "query-admission:source-scale",
                ),
                crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                    "causal-anchor:source-scale",
                ),
            )
            .expect("query admission summary should be valid"),
            vec![
                query_observation_reference(
                    crate::facade::BridgeCausalEvidenceReferenceIdentity::query_observation(
                        crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                            "query-observation:source-scale",
                        ),
                    )
                    .expect("query observation reference identity should be valid"),
                ),
                bridge_route_reference(routed.result().result_summary()),
                bridge_source_materialization_reference(&target_record),
            ],
        )
        .expect("request should be valid");

        let envelope = runtime
            .diagnostics()
            .assemble_causal_explanation_envelope(request)
            .expect("target source materialization should bind");

        assert_eq!(
            runtime.diagnostics().source_materialization_records().len(),
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
