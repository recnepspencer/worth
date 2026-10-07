use super::*;

#[test]
fn runtime_canonicalizes_and_replays_structural_branch_comparison_record() {
    #[derive(Clone)]
    struct BranchDiffSource;

    #[derive(Clone)]
    struct SnapshotBReader;

    impl crate::snapshot::TruthSnapshotReader for SnapshotBReader {
        fn snapshot_identity(&self) -> TruthSnapshotIdentity {
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b")
        }

        fn read_packet(
            &self,
            request: &SnapshotReadPacket,
            _execution: worth_execution::ExecutionRequest<'_, '_>,
        ) -> Result<
            crate::snapshot::SnapshotReadPacketResult,
            crate::snapshot::BridgeSnapshotReadError,
        > {
            Ok(crate::snapshot::SnapshotReadPacketResult::new(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
                request
                    .reads()
                    .iter()
                    .map(|read| {
                        crate::snapshot::SnapshotReadRecord::for_request(
                            read,
                            worth_foundational::facade::AspectValue::String(
                                "fixture-value-b".into(),
                            ),
                        )
                    })
                    .collect(),
            ))
        }
    }

    impl crate::adapter::CommittedPatchSource for BranchDiffSource {
        fn load_committed_patch(
            &self,
            request: crate::adapter::RelationalCommittedPatchRequest,
        ) -> Result<
            crate::input::envelope::BridgeCommittedPatchEnvelope,
            crate::adapter::RelationalBridgeSourceError,
        > {
            crate::input::envelope::BridgeCommittedPatchEnvelope::new(
                crate::input::envelope::BridgeCommittedPatchEnvelopeIdentity::new(
                    request.commit_identity().clone(),
                    crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                    crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                ),
                vec![
                    crate::input::envelope::BridgeCommittedPatchItem::with_target(
                        "entity-1",
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
                    ),
                ],
            )
            .map_err(|error| crate::adapter::RelationalBridgeSourceError::new(error.to_string()))
        }
    }

    impl crate::adapter::SnapshotReadSource for BranchDiffSource {
        fn open_snapshot(
            &self,
            identity: &TruthSnapshotIdentity,
        ) -> Result<
            Box<dyn crate::snapshot::TruthSnapshotReader>,
            crate::adapter::RelationalBridgeSourceError,
        > {
            if crate::truth_identity_fixtures::truth_snapshot_fixture_matches(
                identity,
                "snapshot-a",
            ) {
                Ok(Box::new(StaticSnapshotReader))
            } else if crate::truth_identity_fixtures::truth_snapshot_fixture_matches(
                identity,
                "snapshot-b",
            ) {
                Ok(Box::new(SnapshotBReader))
            } else {
                Err(crate::adapter::RelationalBridgeSourceError::new(format!(
                    "unknown snapshot `{}`",
                    identity.as_str()
                )))
            }
        }
    }

    impl crate::adapter::TruthBranchHeadSource for BranchDiffSource {
        fn load_branch_head_patch(
            &self,
            branch_identity: &TruthBranchIdentity,
        ) -> Result<
            crate::input::envelope::BridgeCommittedPatchEnvelope,
            crate::adapter::RelationalBridgeSourceError,
        > {
            let branch_label = branch_identity.relational_branch_id().unwrap_or("unknown");
            let snapshot = if branch_label == "right" {
                "snapshot-b"
            } else {
                "snapshot-a"
            };
            crate::input::envelope::BridgeCommittedPatchEnvelope::new(
                crate::input::envelope::BridgeCommittedPatchEnvelopeIdentity::new(
                    crate::truth_identity_fixtures::truth_commit_fixture(format!(
                        "head-{}",
                        branch_label
                    )),
                    crate::truth_identity_fixtures::truth_patch_fixture("patch-head"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture(snapshot),
                    branch_identity.clone(),
                ),
                vec![
                    crate::input::envelope::BridgeCommittedPatchItem::with_target(
                        "entity-1",
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
                    ),
                ],
            )
            .map_err(|error| crate::adapter::RelationalBridgeSourceError::new(error.to_string()))
        }
    }

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
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
            ),
        ),
    );

    let runtime = RuntimeBridgeBuilder::new()
        .with_policy(BridgeRuntimePolicy::default())
        .with_relational_source(BranchDiffSource)
        .with_source_adapter(StaticSourceAdapter)
        .with_truth_branch_head_source(BranchDiffSource)
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
        .plan_structural_branch_comparison_from_read_packet(
            &contract,
            SnapshotReadPacket::new(vec![crate::snapshot::SnapshotReadRequest::for_coarse(
                "entity-1",
                crate::snapshot::SnapshotReadContract::scalar(
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid snapshot aspect key"),
                    worth_foundational::facade::ScalarAspectType::String,
                ),
            )]),
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

    let replay = runtime
        .replay_canonical_structural_branch_comparison_record(&record)
        .expect("structural branch comparison replay should succeed");
    let explanation = runtime
        .diagnostics()
        .explain_last_structural_branch_comparison_record()
        .expect("structural branch comparison record should be retained");

    assert_eq!(replay.digest(), artifact.digest());
    assert_eq!(explanation.branch_diff_count(), 1);
    assert_eq!(explanation.candidate_count(), 1);
    assert_eq!(explanation.counters().branch_comparison_count(), 1);
    assert_eq!(explanation.counters().branch_comparison_diff_count(), 1);
    assert_eq!(explanation.counters().structural_candidate_count(), 1);
    assert_eq!(explanation.counters().structural_fingerprint_count(), 3);
    assert_eq!(explanation.counters().structural_widened_scan_count(), 0);
    assert_eq!(
        explanation
            .counters()
            .branch_comparison_drift_rejection_count(),
        0
    );
    assert_eq!(explanation.counters().structural_replay_request_count(), 0);
    assert_eq!(explanation.counters().structural_replay_mismatch_count(), 0);
}
