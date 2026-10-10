use super::*;

#[test]
fn runtime_rejects_structural_declaration_with_different_semantics_version() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let registered = registered_structural(
        "structural:analysis-snapshot",
        StructuralFingerprintFamily::TopologyFingerprint,
        StructuralTruthViewBasis::explicit_snapshot(BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        )),
    );

    runtime
        .admit_structural_comparison(registered)
        .expect("registered structural declaration should be admitted");

    let mismatched = StructuralIdentityDeclaration::advisory_remap(
        StructuralIdentityDeclarationIdentity::admit_bridge_owned("structural:analysis-snapshot"),
        StructuralSchemaIdentity::admit_bridge_owned("schema:geometry"),
        StructuralFingerprintEquivalenceContract::new(
            StructuralSchemaIdentity::admit_bridge_owned("schema:geometry"),
            StructuralFingerprintFamily::TopologyFingerprint,
            "geometry-topology-v2",
            StructuralFingerprintNormalizationRule::SchemaDeclaredCanonicalForm,
            StructuralFingerprintOrderingRule::SchemaDeclaredCanonicalOrder,
            StructuralFingerprintOmissionPolicy::SchemaDeclaredOmissionPolicy,
        ),
        StructuralTruthViewBasis::explicit_snapshot(BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        )),
    );

    let error = runtime
        .admit_structural_comparison(mismatched)
        .expect_err("mismatched semantics version should fail explicitly");

    assert_eq!(
        error.kind(),
        crate::error::BridgeDeliveryErrorKind::StructuralContractMismatch
    );
}

#[test]
fn runtime_replay_rejects_truncated_structural_remap_basis() {
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
    let record =
        runtime.canonicalize_structural_remap_record(&contract, &planned, &reduced, &artifact);

    let error = runtime
        .replay_canonical_structural_remap_record(&record, execution)
        .expect_err("replay should reject a remap record without retained fingerprint basis");

    assert_eq!(
        error.kind(),
        crate::error::BridgeReplayErrorKind::StructuralReplayBasisTruncated
    );
}

#[test]
fn runtime_replay_rejects_truncated_structural_branch_basis() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

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
    let record = runtime
        .canonicalize_structural_branch_comparison_record(&contract, &planned, &reduced, &artifact);

    let error = runtime
        .replay_canonical_structural_branch_comparison_record(&record, execution)
        .expect_err("replay should reject a branch record without retained fingerprint basis");

    assert_eq!(
        error.kind(),
        crate::error::BridgeReplayErrorKind::StructuralReplayBasisTruncated
    );
}
