use super::*;

#[test]
fn runtime_derives_branch_comparison_candidates_from_branch_pair_reads() {
    let policy = crate::policy::BridgeExecutionPolicyBaseline::operational().request_policy();
    let lease = crate::snapshot::test_execution_lease_for_policy(
        policy,
        worth_execution::CancellationToken::new(),
    );
    let resource_request = worth_execution::ExecutionRequest::leased(&lease);

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
            _execution: worth_execution::ExecutionRequest<'_, '_>,
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
            .map_err(|error| {
                crate::adapter::RelationalBridgeSourceError::new(
                    crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                    error.to_string(),
                )
            })
        }
    }

    impl crate::adapter::SnapshotReadSource for BranchDiffSource {
        fn open_snapshot(
            &self,
            identity: &TruthSnapshotIdentity,
            _execution: worth_execution::ExecutionRequest<'_, '_>,
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
                Err(crate::adapter::RelationalBridgeSourceError::new(
                    crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                    format!("unknown snapshot `{}`", identity.as_str()),
                ))
            }
        }
    }

    impl crate::adapter::TruthBranchHeadSource for BranchDiffSource {
        fn load_branch_head_patch(
            &self,
            branch_identity: &TruthBranchIdentity,
            _execution: worth_execution::ExecutionRequest<'_, '_>,
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
            .map_err(|error| {
                crate::adapter::RelationalBridgeSourceError::new(
                    crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                    error.to_string(),
                )
            })
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
    let planned = resource_request
        .run(
            worth_execution::ExecutionWorkCeiling::new(8_000_000),
            |_| {
                runtime.plan_structural_branch_comparison_from_read_packet(
                    &contract,
                    SnapshotReadPacket::new(vec![
                        crate::snapshot::SnapshotReadRequest::for_coarse(
                            "entity-1",
                            crate::snapshot::SnapshotReadContract::scalar(
                                worth_foundational::facade::AspectKey::new("profile")
                                    .expect("valid snapshot aspect key"),
                                worth_foundational::facade::ScalarAspectType::String,
                            ),
                        ),
                    ]),
                    resource_request,
                )
            },
        )
        .expect("active leased scope")
        .0
        .expect("branch comparison should derive candidates from paired reads");
    let reduced = runtime
        .reduce_structural_match_set(&planned)
        .expect("derived branch comparison packet set should reduce");

    assert_eq!(planned.candidate_count(), 1);
    assert_eq!(
        planned.candidates()[0].candidate_kind(),
        StructuralMatchCandidateKind::BranchDiff
    );
    assert_eq!(
        reduced.outcome_class(),
        StructuralMatchOutcomeClass::BranchComparisonArtifact
    );
}
