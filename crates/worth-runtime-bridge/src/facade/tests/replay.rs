#[test]
fn runtime_replays_canonical_historical_evaluation_record() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::branch_head(crate::truth_identity_fixtures::truth_branch_fixture(
            "analysis",
        )),
        BridgeReplayMode::Enabled,
        BridgeDiagnosticsTier::Standard,
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );
    let observation = runtime
        .materialize_truth_view_observation(
            runtime
                .plan_truth_view_packet(declaration, SnapshotReadPacket::new(vec![]), execution)
                .expect("branch-head declaration should plan"),
            execution,
        )
        .expect("branch-head declaration should materialize");
    let record = runtime.canonicalize_historical_evaluation_record(&observation);

    let replay = runtime
        .replay_canonical_historical_evaluation_record(&record, execution)
        .expect("historical record replay should succeed");

    assert_eq!(replay.record_identity(), record.record_identity());
    assert!(
        crate::truth_identity_fixtures::truth_snapshot_fixture_matches(
            replay.snapshot_identity(),
            "snapshot-a"
        )
    );
}

#[test]
fn runtime_replay_rejects_historical_authority_drift_as_authority_mismatch() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    #[derive(Clone)]
    struct DriftSource;

    impl crate::adapter::CommittedPatchSource for DriftSource {
        fn load_committed_patch(
            &self,
            request: crate::adapter::RelationalCommittedPatchRequest,
            _execution: worth_execution::ExecutionRequest<'_, '_>,
        ) -> Result<
            crate::input::envelope::BridgeCommittedPatchEnvelope,
            crate::adapter::RelationalBridgeSourceError,
        > {
            let snapshot = if request.commit_identity().as_str() == "commit-a" {
                "snapshot-b"
            } else {
                "snapshot-a"
            };
            crate::input::envelope::BridgeCommittedPatchEnvelope::new(
                crate::input::envelope::BridgeCommittedPatchEnvelopeIdentity::new(
                    request.commit_identity().clone(),
                    crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture(snapshot),
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

    impl crate::adapter::SnapshotReadSource for DriftSource {
        fn open_snapshot(
            &self,
            identity: &TruthSnapshotIdentity,
            _execution: worth_execution::ExecutionRequest<'_, '_>,
        ) -> Result<
            Box<dyn crate::snapshot::TruthSnapshotReader>,
            crate::adapter::RelationalBridgeSourceError,
        > {
            if identity.as_str() == "snapshot-b" {
                Ok(Box::new(StaticSnapshotReader))
            } else {
                Err(crate::adapter::RelationalBridgeSourceError::new(
                    crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                    "missing snapshot",
                ))
            }
        }
    }

    impl crate::adapter::TruthBranchHeadSource for DriftSource {
        fn load_branch_head_patch(
            &self,
            branch_identity: &TruthBranchIdentity,
            _execution: worth_execution::ExecutionRequest<'_, '_>,
        ) -> Result<
            crate::input::envelope::BridgeCommittedPatchEnvelope,
            crate::adapter::RelationalBridgeSourceError,
        > {
            crate::input::envelope::BridgeCommittedPatchEnvelope::new(
                crate::input::envelope::BridgeCommittedPatchEnvelopeIdentity::new(
                    crate::truth_identity_fixtures::truth_commit_fixture("head-analysis"),
                    crate::truth_identity_fixtures::truth_patch_fixture("patch-head"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
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

    let original = runtime(BridgeRuntimePolicy::default());
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::historical_commit(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        ),
        BridgeReplayMode::Enabled,
        BridgeDiagnosticsTier::Standard,
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );
    let record = original.canonicalize_historical_evaluation_record(
        &original
            .materialize_truth_view_observation(
                original
                    .plan_truth_view_packet(
                        declaration,
                        SnapshotReadPacket::new(vec![]),
                        resource_request,
                    )
                    .expect("original historical declaration should plan"),
                resource_request,
            )
            .expect("original historical declaration should materialize"),
    );
    let drifted = RuntimeBridgeBuilder::new()
        .with_policy(BridgeRuntimePolicy::default())
        .with_relational_source(DriftSource)
        .with_source_adapter(StaticSourceAdapter)
        .with_truth_branch_head_source(DriftSource)
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
        .register_source(registered_source(
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
        .expect("drifted runtime should build");

    let error = drifted
        .replay_canonical_historical_evaluation_record(&record, resource_request)
        .expect_err("historical replay should reject authority drift");

    assert_eq!(
        error.kind(),
        crate::error::BridgeReplayErrorKind::HistoricalEvaluationAuthorityMismatch
    );
    assert_eq!(
        drifted
            .diagnostics()
            .last_historical_evaluation_failure()
            .expect("historical replay mismatch should be recorded")
            .failure_class(),
        crate::facade::BridgeHistoricalEvaluationFailureClass::HistoricalReplayMismatch
    );
}

use super::*;
