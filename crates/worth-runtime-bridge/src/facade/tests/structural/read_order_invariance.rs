use super::*;

#[test]
fn runtime_branch_comparison_ignores_read_result_order_when_structure_is_equal() {
    #[derive(Clone)]
    struct ReorderedSource;

    #[derive(Clone)]
    struct ForwardReader;

    #[derive(Clone)]
    struct ReverseReader;

    impl crate::snapshot::TruthSnapshotReader for ForwardReader {
        fn snapshot_identity(&self) -> TruthSnapshotIdentity {
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a")
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
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                request
                    .reads()
                    .iter()
                    .enumerate()
                    .map(|(index, read)| {
                        crate::snapshot::SnapshotReadRecord::for_request(
                            read,
                            worth_foundational::facade::AspectValue::String(
                                format!("aspect-text-{index}").into(),
                            ),
                        )
                    })
                    .collect(),
            ))
        }
    }

    impl crate::snapshot::TruthSnapshotReader for ReverseReader {
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
                    .enumerate()
                    .rev()
                    .map(|(index, read)| {
                        crate::snapshot::SnapshotReadRecord::for_request(
                            read,
                            worth_foundational::facade::AspectValue::String(
                                format!("aspect-text-{index}").into(),
                            ),
                        )
                    })
                    .collect(),
            ))
        }
    }

    impl crate::adapter::CommittedPatchSource for ReorderedSource {
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

    impl crate::adapter::SnapshotReadSource for ReorderedSource {
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
                Ok(Box::new(ForwardReader))
            } else if crate::truth_identity_fixtures::truth_snapshot_fixture_matches(
                identity,
                "snapshot-b",
            ) {
                Ok(Box::new(ReverseReader))
            } else {
                Err(crate::adapter::RelationalBridgeSourceError::new(format!(
                    "unknown snapshot `{}`",
                    identity.as_str()
                )))
            }
        }
    }

    impl crate::adapter::TruthBranchHeadSource for ReorderedSource {
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
        .with_relational_source(ReorderedSource)
        .with_source_adapter(StaticSourceAdapter)
        .with_truth_branch_head_source(ReorderedSource)
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
            SnapshotReadPacket::new(vec![
                crate::snapshot::SnapshotReadRequest::for_coarse(
                    "entity-1",
                    crate::snapshot::SnapshotReadContract::scalar(
                        worth_foundational::facade::AspectKey::new("profile")
                            .expect("valid snapshot aspect key"),
                        worth_foundational::facade::ScalarAspectType::String,
                    ),
                ),
                crate::snapshot::SnapshotReadRequest::for_coarse(
                    "entity-2",
                    crate::snapshot::SnapshotReadContract::scalar(
                        worth_foundational::facade::AspectKey::new("profile")
                            .expect("valid snapshot aspect key"),
                        worth_foundational::facade::ScalarAspectType::String,
                    ),
                ),
            ]),
        )
        .expect("branch comparison should plan from reordered equal reads");
    let reduced = runtime
        .reduce_structural_match_set(&planned)
        .expect("reordered equal reads should still reduce");

    assert_eq!(planned.candidate_count(), 0);
    assert_eq!(reduced.branch_diff_count(), 0);
    assert_eq!(
        reduced.outcome_class(),
        StructuralMatchOutcomeClass::BranchComparisonArtifact
    );
}
