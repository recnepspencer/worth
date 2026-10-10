use super::retained_mapping_digest_support::{
    expected_retained_causal_digest, ExpectedRetainedCausalDigestArtifact,
};
use super::retained_mapping_support::{
    binding_for, branch_comparison_declaration, bridge_continuity_reference,
    bridge_merge_reference, bridge_route_reference, bridge_source_materialization_reference,
    bridge_stream_replay_reference, bridge_structural_branch_comparison_reference,
    bridge_structural_remap_reference, query_observation_reference, registered_causal_merge,
    retained_runtime,
};
use super::{
    canonical_envelope, registered_source, registered_structural, BridgeSourceCapability,
    BridgeTruthViewSelector, SnapshotReadPacket, StructuralFingerprintFamily,
    StructuralTruthViewBasis,
};
use crate::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEvidenceBindingClass,
    BridgeCausalEvidenceFamily, BridgeContinuityAuthorityBasis, BridgeLineageContext,
    BridgeMappingContext, BridgeRouteRequest, MergeHistoryDeclarationIdentity,
    StructuralCandidateIdentity, StructuralComparisonMode, StructuralMatchCandidate,
    StructuralMatchCandidateKind,
};

#[test]
fn causal_envelope_maps_retained_source_structural_stream_continuity_and_merge_records() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let merge_declaration = registered_causal_merge(
        MergeHistoryDeclarationIdentity::admit_bridge_owned("merge:causal-retained"),
    );
    let branch_declaration = branch_comparison_declaration(
        crate::facade::StructuralIdentityDeclarationIdentity::admit_bridge_owned(
            "structural:branch-causal",
        ),
    );
    let runtime = retained_runtime(merge_declaration.clone(), branch_declaration.clone());
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
        .deliver_continuity(&route_record)
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

    let structural_contract = runtime
        .admit_structural_comparison(registered_structural(
            "structural:analysis-snapshot",
            StructuralFingerprintFamily::TopologyFingerprint,
            StructuralTruthViewBasis::explicit_snapshot(BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            )),
        ))
        .expect("structural declaration should admit");
    assert_eq!(
        structural_contract
            .validated_declaration()
            .declaration()
            .comparison_mode(),
        StructuralComparisonMode::AdvisoryRemap
    );
    let structural_read =
        SnapshotReadPacket::new(vec![crate::snapshot::SnapshotReadRequest::for_coarse(
            "entity-1",
            crate::snapshot::SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid snapshot aspect key"),
                worth_foundational::facade::ScalarAspectType::String,
            ),
        )]);
    let structural_planned = runtime
        .plan_structural_match_packet_set_from_read_packets(
            &structural_contract,
            structural_read.clone(),
            vec![structural_read],
            execution,
        )
        .expect("structural packets should plan");
    let structural_reduced = runtime
        .reduce_structural_match_set(&structural_planned)
        .expect("structural set should reduce");
    let structural_artifact = runtime
        .publish_structural_remap_artifact(&structural_reduced)
        .expect("remap artifact should publish");
    let structural_record = runtime.canonicalize_structural_remap_record(
        &structural_contract,
        &structural_planned,
        &structural_reduced,
        &structural_artifact,
    );

    let branch_contract = runtime
        .admit_structural_comparison(branch_declaration)
        .expect("branch comparison should admit");
    let branch_planned = runtime
        .plan_structural_match_packet_set(
            &branch_contract,
            vec![StructuralMatchCandidate::new(
                StructuralCandidateIdentity::admit_bridge_owned("diff:causal"),
                StructuralMatchCandidateKind::BranchDiff,
            )],
        )
        .expect("branch diff should plan");
    let branch_reduced = runtime
        .reduce_structural_match_set(&branch_planned)
        .expect("branch comparison should reduce");
    let branch_artifact = runtime
        .publish_branch_comparison_artifact(&branch_reduced)
        .expect("branch artifact should publish");
    let branch_record = runtime.canonicalize_structural_branch_comparison_record(
        &branch_contract,
        &branch_planned,
        &branch_reduced,
        &branch_artifact,
    );

    let stream_declaration = crate::stream::ChangeStreamDeclaration::new(
        crate::stream::StreamConsumerShape::RoutingConsumer,
        crate::stream::StreamResumeMode::FromCheckpointOnly,
        crate::stream::StreamCheckpointPublicationMode::PublishEveryWindow,
        crate::stream::StreamCoalescingIntent::Prefer(
            crate::stream::StreamCoalescingFamily::RoutingWindowCoalescing,
        ),
        crate::stream::StreamReplayMode::Enabled,
        crate::stream::StreamDeliveryIntent::RouteInvalidations,
        crate::stream::StreamDiagnosticsPolicyClass::Standard,
    );
    let stream_protocol = runtime
        .validate_change_stream_declaration(stream_declaration)
        .expect("stream declaration should validate");
    let stream_contract = runtime
        .resolve_change_stream_consumer_contract(&stream_protocol)
        .expect("stream contract should resolve");
    let stream_window = runtime
        .plan_change_stream_window(
            &stream_contract,
            vec![canonical_envelope(
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            )],
        )
        .expect("stream window should plan");
    let checkpoint = runtime.publish_consumer_checkpoint(
        &stream_contract,
        &stream_window,
        crate::stream::StreamCheckpointFrontierKind::ContiguousFrontier,
    );
    let stream_record = runtime
        .canonicalize_stream_replay_record(&stream_contract, &stream_window, &checkpoint)
        .expect("stream replay should canonicalize");

    let merge_contract = runtime
        .admit_merge_history(merge_declaration)
        .expect("merge should admit");
    let merge_bundle = runtime
        .replay_merge_history(&merge_contract)
        .expect("merge should replay");
    let merge_record = runtime.canonicalize_merge_record(&merge_bundle);

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
            bridge_structural_remap_reference(&structural_record),
            bridge_structural_branch_comparison_reference(&branch_record),
            bridge_stream_replay_reference(&stream_record),
            bridge_continuity_reference(&continuity),
            bridge_merge_reference(&merge_record),
        ],
    )
    .expect("request should be valid");

    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect("retained mappings should assemble");

    assert_eq!(envelope.bindings().len(), 8);
    assert_eq!(envelope.counters().bridge_retained_lookup_count(), 7);
    assert_eq!(envelope.counters().retained_bridge_binding_count(), 7);
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
            BridgeCausalEvidenceFamily::BridgeStructuralRemap,
            structural_record.record_identity().as_str()
        )
        .retained_record_digest_for_reporting(),
        Some(
            expected_retained_causal_digest(
                ExpectedRetainedCausalDigestArtifact::StructuralRemapRecord,
                &[
                    structural_record.record_identity().as_str(),
                    structural_record.schema_version(),
                    structural_record.contract().contract_identity().as_str(),
                ],
            )
            .as_str()
        )
    );
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceFamily::BridgeStructuralBranchComparison,
            branch_record.record_identity().as_str()
        )
        .retained_record_digest_for_reporting(),
        Some(
            expected_retained_causal_digest(
                ExpectedRetainedCausalDigestArtifact::StructuralBranchComparisonRecord,
                &[
                    branch_record.record_identity().as_str(),
                    branch_record.schema_version(),
                    branch_record.contract().contract_identity().as_str(),
                ],
            )
            .as_str()
        )
    );
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceFamily::BridgeStreamReplay,
            stream_record.replay_record_identity().as_str()
        )
        .retained_record_digest_for_reporting(),
        Some(
            expected_retained_causal_digest(
                ExpectedRetainedCausalDigestArtifact::StreamReplayRecord,
                &[
                    stream_record.replay_record_identity().as_str(),
                    stream_record.consumer_contract_identity().as_str(),
                    stream_record.stream_window_identity().as_str(),
                    stream_record.protocol_semantics_version(),
                ],
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
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceFamily::BridgeMerge,
            merge_record.record_identity().as_str()
        )
        .retained_record_digest_for_reporting(),
        Some(
            expected_retained_causal_digest(
                ExpectedRetainedCausalDigestArtifact::MergeRecord,
                &[
                    merge_record.record_identity().as_str(),
                    merge_record.schema_version(),
                    merge_record.contract().contract_identity().as_str(),
                ],
            )
            .as_str()
        )
    );
}
