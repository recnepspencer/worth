use super::*;

#[test]
fn source_builder_order_does_not_change_materialized_source_truth() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let first = RuntimeBridgeBuilder::new()
        .with_policy(BridgeRuntimePolicy::default())
        .with_relational_source(StaticSource)
        .with_source_adapter(StaticSourceAdapter)
        .with_truth_branch_head_source(StaticSource)
        .with_signal_sink(StaticSink)
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
        .expect("first builder order should succeed");

    let second = RuntimeBridgeBuilder::new()
        .with_policy(BridgeRuntimePolicy::default())
        .with_relational_source(StaticSource)
        .with_truth_branch_head_source(StaticSource)
        .with_signal_sink(StaticSink)
        .with_source_adapter(StaticSourceAdapter)
        .register_source(registered_source(
            "source:analysis-history",
            BridgeTruthViewSelector::historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            ),
            vec![
                BridgeSourceCapability::ReplayContinuityRead,
                BridgeSourceCapability::BranchRead,
                BridgeSourceCapability::HistoricalRead,
                BridgeSourceCapability::SnapshotRead,
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
        .expect("second builder order should succeed");

    let first_contract = first
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
        .expect("first runtime source should admit");
    let second_contract = second
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
        .expect("second runtime source should admit");

    let first_record = first
        .canonicalize_source_materialization_record(
            &first_contract,
            &first
                .materialize_source_packet(
                    &first_contract,
                    SnapshotReadPacket::new(vec![]),
                    execution,
                )
                .expect("first source packet should materialize"),
        )
        .expect("first source record should canonicalize");
    let second_record = second
        .canonicalize_source_materialization_record(
            &second_contract,
            &second
                .materialize_source_packet(
                    &second_contract,
                    SnapshotReadPacket::new(vec![]),
                    execution,
                )
                .expect("second source packet should materialize"),
        )
        .expect("second source record should canonicalize");

    assert_eq!(
        first.source_registry().digest(),
        second.source_registry().digest()
    );
    assert_eq!(first_record, second_record);
}

#[test]
fn source_diagnostics_richness_preserves_source_truth() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let minimal = runtime(BridgeRuntimePolicy::operational());
    let exhaustive = runtime(BridgeRuntimePolicy::forensic());
    let declaration = registered_source(
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
    );

    let minimal_contract = minimal
        .admit_source(declaration.clone())
        .expect("minimal diagnostics runtime should admit source");
    let exhaustive_contract = exhaustive
        .admit_source(declaration)
        .expect("exhaustive diagnostics runtime should admit source");

    let minimal_record = minimal
        .canonicalize_source_materialization_record(
            &minimal_contract,
            &minimal
                .materialize_source_packet(
                    &minimal_contract,
                    SnapshotReadPacket::new(vec![]),
                    execution,
                )
                .expect("minimal diagnostics source should materialize"),
        )
        .expect("minimal diagnostics source record should canonicalize");
    let exhaustive_record = exhaustive
        .canonicalize_source_materialization_record(
            &exhaustive_contract,
            &exhaustive
                .materialize_source_packet(
                    &exhaustive_contract,
                    SnapshotReadPacket::new(vec![]),
                    execution,
                )
                .expect("exhaustive diagnostics source should materialize"),
        )
        .expect("exhaustive diagnostics source record should canonicalize");

    assert_eq!(minimal_record, exhaustive_record);
    assert_eq!(
        minimal
            .diagnostics()
            .explain_source_materialization_record(&minimal_record)
            .snapshot_identities(),
        exhaustive
            .diagnostics()
            .explain_source_materialization_record(&exhaustive_record)
            .snapshot_identities()
    );
}
