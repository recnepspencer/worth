use super::retained_mapping_digest_support::{
    expected_retained_causal_digest, ExpectedRetainedCausalDigestArtifact,
};
use super::retained_mapping_support::{
    binding_for, bridge_continuity_reference, bridge_route_reference,
    bridge_source_materialization_reference, query_observation_reference, retained_runtime,
};
use super::{
    registered_source, BridgeSourceCapability, BridgeTruthViewSelector, SnapshotReadPacket,
};
use crate::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEvidenceBindingClass,
    BridgeCausalEvidenceFamily, BridgeContinuityAuthorityBasis, BridgeLineageContext,
    BridgeMappingContext, BridgeRouteRequest,
};

#[test]
fn causal_envelope_maps_retained_source_and_continuity_records() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = retained_runtime();
    let planned_route = runtime
        .plan_committed_patch_with_mapping_context(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-causal-retained",
            )),
            BridgeMappingContext::default().with_lineage_context(BridgeLineageContext::new(
                BridgeContinuityAuthorityBasis::new(
                    crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                ),
            )),
            execution,
        )
        .expect("route should plan");
    let route_result = runtime
        .deliver_invalidation(planned_route, execution)
        .expect("route should deliver");
    let route_record = runtime
        .diagnostics()
        .last_route_record()
        .expect("route record should be retained");
    let continuity = runtime
        .deliver_continuity(&route_record, execution)
        .expect("continuity should deliver")
        .canonical_record()
        .clone();

    let source_contract = runtime
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
        .expect("historical source should admit");
    let source_observation = runtime
        .materialize_source_packet(&source_contract, SnapshotReadPacket::new(vec![]), execution)
        .expect("source should materialize");
    let source_record = runtime
        .canonicalize_source_materialization_record(&source_contract, &source_observation)
        .expect("source record should canonicalize");

    let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:retained-expansion",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:retained-expansion",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            query_observation_reference(
                crate::facade::BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:retained-expansion",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_route_reference(route_result.result_summary()),
            bridge_source_materialization_reference(&source_record),
            bridge_continuity_reference(&continuity),
        ],
    )
    .expect("request should be valid");

    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect("retained mappings should assemble");

    assert_eq!(envelope.bindings().len(), 4);
    assert_eq!(envelope.counters().bridge_retained_lookup_count(), 3);
    assert_eq!(envelope.counters().retained_bridge_binding_count(), 3);
    assert_eq!(envelope.counters().bridge_record_unindexed_scan_count(), 0);
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceFamily::BridgeSourceMaterialization,
            source_record.record_identity().as_str()
        )
        .binding_class(),
        BridgeCausalEvidenceBindingClass::RetainedBridgeRecord
    );
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceFamily::BridgeSourceMaterialization,
            source_record.record_identity().as_str()
        )
        .retained_record_digest_for_reporting(),
        Some(
            expected_retained_causal_digest(
                ExpectedRetainedCausalDigestArtifact::SourceMaterializationRecord,
                &[source_record.record_identity().as_str()],
            )
            .as_str()
        )
    );
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceFamily::BridgeContinuity,
            continuity.route_identity().as_str()
        )
        .retained_record_digest_for_reporting(),
        Some(
            expected_retained_causal_digest(
                ExpectedRetainedCausalDigestArtifact::ContinuityRecord,
                &[
                    continuity.route_identity().as_str(),
                    continuity.schema_version(),
                    continuity.continuity_artifact_identity().as_str(),
                    continuity.remapped_subscription_slice_identity().as_str(),
                ],
            )
            .as_str()
        )
    );
}
