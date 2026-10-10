use super::*;

#[test]
fn runtime_admits_registered_structural_declaration() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = registered_structural(
        "structural:analysis-snapshot",
        StructuralFingerprintFamily::TopologyFingerprint,
        StructuralTruthViewBasis::explicit_snapshot(BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        )),
    );

    let contract = runtime
        .admit_structural_comparison(declaration)
        .expect("registered structural declaration should be admitted");
    assert_eq!(
        contract
            .validated_declaration()
            .declaration()
            .comparison_mode(),
        StructuralComparisonMode::AdvisoryRemap
    );
}

#[test]
fn runtime_plans_and_reduces_structural_candidates_for_advisory_remap() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = registered_structural(
        "structural:analysis-snapshot",
        StructuralFingerprintFamily::TopologyFingerprint,
        StructuralTruthViewBasis::explicit_snapshot(BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        )),
    );
    let contract = runtime
        .admit_structural_comparison(declaration)
        .expect("registered structural declaration should be admitted");

    let planned = runtime
        .plan_structural_match_packet_set(
            &contract,
            vec![StructuralMatchCandidate::new(
                StructuralCandidateIdentity::admit_bridge_owned("candidate:geometry-a"),
                StructuralMatchCandidateKind::ExactAdvisoryMatch,
            )],
        )
        .expect("structural candidates should plan");
    let reduced = runtime
        .reduce_structural_match_set(&planned)
        .expect("planned structural packet set should reduce");
    let artifact = runtime
        .publish_structural_remap_artifact(&reduced)
        .expect("advisory remap outcome should publish");

    assert_eq!(
        reduced.outcome_class(),
        StructuralMatchOutcomeClass::ExactAdvisoryMatch
    );
    assert_eq!(artifact.reduced_match_set(), &reduced);
    assert_eq!(
        artifact.reduced_match_set().outcome_class(),
        StructuralMatchOutcomeClass::ExactAdvisoryMatch
    );
}

#[test]
fn runtime_rejects_branch_diff_candidate_for_advisory_remap_contract() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = registered_structural(
        "structural:analysis-snapshot",
        StructuralFingerprintFamily::TopologyFingerprint,
        StructuralTruthViewBasis::explicit_snapshot(BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        )),
    );
    let contract = runtime
        .admit_structural_comparison(declaration)
        .expect("registered structural declaration should be admitted");

    let error = runtime
        .plan_structural_match_packet_set(
            &contract,
            vec![StructuralMatchCandidate::new(
                StructuralCandidateIdentity::admit_bridge_owned("candidate:branch-diff"),
                StructuralMatchCandidateKind::BranchDiff,
            )],
        )
        .expect_err("branch diff candidate should be rejected for advisory remap");

    assert_eq!(
        error.kind(),
        crate::error::BridgeDeliveryErrorKind::StructuralPlanRejected
    );
}

#[test]
fn runtime_reduces_branch_comparison_candidates_to_branch_artifact() {
    let declaration = StructuralIdentityDeclaration::branch_comparison(
        StructuralIdentityDeclarationIdentity::admit_bridge_owned("structural:branch-compare"),
        StructuralSchemaIdentity::admit_bridge_owned("schema:geometry"),
        StructuralFingerprintEquivalenceContract::new(
            StructuralSchemaIdentity::admit_bridge_owned("schema:geometry"),
            StructuralFingerprintFamily::BranchComparisonFingerprint,
            "geometry-branch-v1",
            StructuralFingerprintNormalizationRule::SchemaDeclaredCanonicalForm,
            StructuralFingerprintOrderingRule::SchemaDeclaredCanonicalOrder,
            StructuralFingerprintOmissionPolicy::SchemaDeclaredOmissionPolicy,
        ),
        StructuralTruthViewBasis::explicit_branch_pair(
            BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("left"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("right"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
        ),
    );

    let runtime = RuntimeBridgeBuilder::new()
        .with_policy(BridgeRuntimePolicy::default())
        .with_relational_source(StaticSource)
        .with_source_adapter(StaticSourceAdapter)
        .with_truth_branch_head_source(StaticSource)
        .with_signal_sink(StaticSink)
        .register_source(registered_source(
            "source:analysis-snapshot",
            BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::BranchRead,
            ],
        ))
        .register_structural(declaration.clone())
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::admit_bridge_owned("mapping"),
            TruthPatchScope::for_entity_field(
                MappingSelector::exact("entity-1"),
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid native field key"),
            ),
            crate::snapshot::SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                worth_foundational::facade::ScalarAspectType::String,
            ),
            SignalInvalidationScope::admit_bridge_owned("signal:profile"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .expect("runtime should build with structural declaration");

    let contract = runtime
        .admit_structural_comparison(declaration)
        .expect("branch comparison declaration should be admitted");
    let planned = runtime
        .plan_structural_match_packet_set(
            &contract,
            vec![StructuralMatchCandidate::new(
                StructuralCandidateIdentity::admit_bridge_owned("diff:one"),
                StructuralMatchCandidateKind::BranchDiff,
            )],
        )
        .expect("branch diff candidate should plan");
    let reduced = runtime
        .reduce_structural_match_set(&planned)
        .expect("branch comparison should reduce");
    let artifact = runtime
        .publish_branch_comparison_artifact(&reduced)
        .expect("branch comparison outcome should publish");

    assert_eq!(
        reduced.outcome_class(),
        StructuralMatchOutcomeClass::BranchComparisonArtifact
    );
    assert_eq!(artifact.reduced_match_set(), &reduced);
    assert_eq!(artifact.reduced_match_set().branch_diff_count(), 1);
}

#[test]
fn runtime_rejects_remap_publication_for_ambiguous_reduced_match_set() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = registered_structural(
        "structural:analysis-snapshot",
        StructuralFingerprintFamily::TopologyFingerprint,
        StructuralTruthViewBasis::explicit_snapshot(BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        )),
    );
    let contract = runtime
        .admit_structural_comparison(declaration)
        .expect("registered structural declaration should be admitted");
    let planned = runtime
        .plan_structural_match_packet_set(
            &contract,
            vec![
                StructuralMatchCandidate::new(
                    StructuralCandidateIdentity::admit_bridge_owned("candidate:a"),
                    StructuralMatchCandidateKind::ExactAdvisoryMatch,
                ),
                StructuralMatchCandidate::new(
                    StructuralCandidateIdentity::admit_bridge_owned("candidate:b"),
                    StructuralMatchCandidateKind::AdvisoryReuseCandidate,
                ),
            ],
        )
        .expect("ambiguous structural candidates should still plan");
    let reduced = runtime
        .reduce_structural_match_set(&planned)
        .expect("ambiguous planned packet set should reduce");

    let error = runtime
        .publish_structural_remap_artifact(&reduced)
        .expect_err("ambiguous reduced set should not publish a remap artifact");
    assert_eq!(
        error.kind(),
        crate::error::BridgeDeliveryErrorKind::StructuralPlanRejected
    );
}

#[test]
fn runtime_reduces_lineage_structural_divergence_to_typed_rejection() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = registered_structural(
        "structural:analysis-snapshot",
        StructuralFingerprintFamily::TopologyFingerprint,
        StructuralTruthViewBasis::explicit_snapshot(BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        )),
    );
    let contract = runtime
        .admit_structural_comparison(declaration)
        .expect("registered structural declaration should be admitted");
    let planned = runtime
        .plan_structural_match_packet_set(
            &contract,
            vec![StructuralMatchCandidate::new(
                StructuralCandidateIdentity::admit_bridge_owned("candidate:lineage-divergence"),
                StructuralMatchCandidateKind::LineageStructuralDivergence,
            )],
        )
        .expect("lineage divergence candidate should plan");
    let reduced = runtime
        .reduce_structural_match_set(&planned)
        .expect("lineage divergence should reduce");

    assert_eq!(
        reduced.outcome_class(),
        StructuralMatchOutcomeClass::RejectedLineageStructuralDivergence
    );
    assert!(
        runtime.publish_structural_remap_artifact(&reduced).is_err(),
        "lineage divergence must not publish an advisory remap artifact"
    );
    assert_eq!(reduced.branch_diff_count(), 0);
}

#[test]
fn runtime_canonicalizes_and_replays_structural_remap_record() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = registered_structural(
        "structural:analysis-snapshot",
        StructuralFingerprintFamily::TopologyFingerprint,
        StructuralTruthViewBasis::explicit_snapshot(BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        )),
    );
    let contract = runtime
        .admit_structural_comparison(declaration)
        .expect("registered structural declaration should be admitted");
    let planned = runtime
        .plan_structural_match_packet_set_from_read_packets(
            &contract,
            SnapshotReadPacket::new(vec![crate::snapshot::SnapshotReadRequest::for_coarse(
                "entity-1",
                crate::snapshot::SnapshotReadContract::scalar(
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid snapshot aspect key"),
                    worth_foundational::facade::ScalarAspectType::String,
                ),
            )]),
            vec![SnapshotReadPacket::new(vec![
                crate::snapshot::SnapshotReadRequest::for_coarse(
                    "entity-1",
                    crate::snapshot::SnapshotReadContract::scalar(
                        worth_foundational::facade::AspectKey::new("profile")
                            .expect("valid snapshot aspect key"),
                        worth_foundational::facade::ScalarAspectType::String,
                    ),
                ),
            ])],
            execution,
        )
        .expect("structural candidates should plan");
    let reduced = runtime
        .reduce_structural_match_set(&planned)
        .expect("planned structural packet set should reduce");
    let artifact = runtime
        .publish_structural_remap_artifact(&reduced)
        .expect("advisory remap outcome should publish");
    let record =
        runtime.canonicalize_structural_remap_record(&contract, &planned, &reduced, &artifact);

    let replay = runtime
        .replay_canonical_structural_remap_record(&record, execution)
        .expect("structural remap replay should succeed");
    let explanation = runtime
        .diagnostics()
        .explain_last_structural_remap_record()
        .expect("structural remap record should be retained");

    assert_eq!(replay.digest(), artifact.digest());
    assert_eq!(
        runtime
            .diagnostics()
            .last_structural_remap_record()
            .expect("structural remap record should be present")
            .record_identity(),
        record.record_identity()
    );
    assert_eq!(explanation.candidate_count(), 1);
    assert_eq!(
        explanation.outcome_class(),
        StructuralMatchOutcomeClass::ExactAdvisoryMatch
    );
    assert_eq!(explanation.counters().structural_declaration_count(), 1);
    assert_eq!(explanation.counters().structural_contract_count(), 1);
    assert_eq!(explanation.counters().structural_match_packet_count(), 1);
    assert_eq!(explanation.counters().structural_candidate_count(), 1);
    assert_eq!(explanation.counters().structural_exact_match_count(), 1);
    assert_eq!(explanation.counters().structural_fingerprint_count(), 2);
    assert_eq!(explanation.counters().structural_replay_request_count(), 0);
    assert_eq!(explanation.counters().structural_replay_mismatch_count(), 0);
    assert_eq!(
        explanation
            .counters()
            .branch_comparison_drift_rejection_count(),
        0
    );
}
