use super::*;

#[test]
fn runtime_plans_registered_source_packet_set() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

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
        .expect("registered historical source should be admitted");

    let planned = runtime
        .plan_source_packet_set(&contract, SnapshotReadPacket::new(vec![]), execution)
        .expect("registered source packet set should plan");

    assert_eq!(planned.contract(), &contract);
    assert_eq!(planned.packets().len(), 1);
    assert_eq!(
        planned
            .validated_declaration()
            .declaration()
            .declaration_identity()
            .as_str(),
        "source:analysis-history"
    );
}

#[test]
fn runtime_materializes_registered_source_packet_set() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

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
        .expect("registered historical source should be admitted");
    let planned = runtime
        .plan_source_packet_set(&contract, SnapshotReadPacket::new(vec![]), execution)
        .expect("registered source packet set should plan");

    let materialized = runtime
        .materialize_source(&planned, execution)
        .expect("registered source packet set should materialize");

    assert_eq!(materialized.planned_packet_set().digest(), planned.digest());
    assert_eq!(materialized.observations().len(), 1);
    assert!(
        crate::truth_identity_fixtures::truth_snapshot_fixture_matches(
            materialized.first().snapshot_identity(),
            "snapshot-a"
        )
    );
    assert_eq!(materialized.planned_packet_set().packet_count(), 1);
    assert_eq!(materialized.planned_packet_set().packet_member_count(), 0);
    assert_eq!(materialized.materialization_count(), 1);
}

#[test]
fn runtime_canonicalizes_registered_source_materialization_record() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

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
        .expect("registered historical source should be admitted");

    let observation = runtime
        .materialize_source_packet(&contract, SnapshotReadPacket::new(vec![]), execution)
        .expect("registered historical source should materialize");
    let record = runtime
        .canonicalize_source_materialization_record(&contract, &observation)
        .expect("registered source materialization should canonicalize");

    assert_eq!(
        record.source_declaration_identity(),
        "source:analysis-history"
    );
    assert_eq!(record.snapshot_identities().len(), 1);
    assert!(
        crate::truth_identity_fixtures::truth_snapshot_fixture_matches(
            &record.snapshot_identities()[0],
            "snapshot-a"
        )
    );
    assert_eq!(
        record.source_capability_digest(),
        contract.required_capabilities().digest()
    );
    assert_eq!(record.counters().source_packet_count(), 1);
    assert_eq!(record.counters().source_materialization_count(), 1);
    assert_eq!(
        record.planned_packet_set_digest(),
        runtime
            .plan_source_packet_set(&contract, SnapshotReadPacket::new(vec![]), execution)
            .expect("source packet set should plan")
            .digest()
    );
}

#[test]
fn runtime_canonicalizes_registered_source_packet_set_record() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

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
        .expect("registered historical source should be admitted");
    let planned = runtime
        .plan_source_packet_set(&contract, SnapshotReadPacket::new(vec![]), execution)
        .expect("registered source packet set should plan");
    let materialized = runtime
        .materialize_source(&planned, execution)
        .expect("registered source packet set should materialize");

    let record = runtime
        .canonicalize_source_materialization_packet_set_record(&materialized)
        .expect("registered source packet set should canonicalize");

    assert_eq!(record.planned_packet_set_digest(), planned.digest());
    assert_eq!(
        record.materialized_packet_set_digest(),
        materialized.digest()
    );
    assert_eq!(
        record.counters().source_packet_count(),
        planned.packet_count()
    );
    assert_eq!(
        record.counters().source_packet_member_count(),
        planned.packet_member_count()
    );
    assert_eq!(
        record.counters().source_materialization_count(),
        materialized.materialization_count()
    );
}

#[test]
fn runtime_replays_multi_packet_source_materialization_record() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

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
        .expect("registered historical source should be admitted");

    let planned_packet_set = runtime
        .plan_source_packet_batch(
            &contract,
            vec![
                SnapshotReadPacket::new(vec![crate::snapshot::SnapshotReadRequest::for_coarse(
                    "entity-1",
                    crate::snapshot::SnapshotReadContract::scalar(
                        worth_foundational::facade::AspectKey::new("profile")
                            .expect("valid snapshot aspect key"),
                        worth_foundational::facade::ScalarAspectType::String,
                    ),
                )]),
                SnapshotReadPacket::new(vec![crate::snapshot::SnapshotReadRequest::for_coarse(
                    "entity-2",
                    crate::snapshot::SnapshotReadContract::scalar(
                        worth_foundational::facade::AspectKey::new("profile")
                            .expect("valid snapshot aspect key"),
                        worth_foundational::facade::ScalarAspectType::String,
                    ),
                )]),
            ],
            execution,
        )
        .expect("multi-packet source set should plan");
    let materialized = runtime
        .materialize_source_packet_batch(
            &contract,
            planned_packet_set
                .packets()
                .iter()
                .map(|packet| packet.read_packet().clone())
                .collect(),
            execution,
        )
        .expect("multi-packet source set should materialize");
    let record = runtime
        .canonicalize_source_materialization_packet_set_record(&materialized)
        .expect("multi-packet source set should canonicalize");

    assert_eq!(record.counters().source_packet_count(), 2);
    assert_eq!(record.read_packets().len(), 2);
    assert_eq!(record.planned_packet_digests().len(), 2);
    assert_eq!(record.truth_view_digest(), materialized.digest());

    let replayed = runtime
        .replay_source_materialization_record(&record, execution)
        .expect("multi-packet source materialization should replay");

    assert_eq!(replayed, record);
}

#[test]
fn runtime_lowers_identical_registered_source_requests_to_identical_records() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

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
        .expect("registered historical source should be admitted");

    let left = runtime
        .canonicalize_source_materialization_record(
            &contract,
            &runtime
                .materialize_source_packet(&contract, SnapshotReadPacket::new(vec![]), execution)
                .expect("left source packet should materialize"),
        )
        .expect("left source record should canonicalize");
    let right = runtime
        .canonicalize_source_materialization_record(
            &contract,
            &runtime
                .materialize_source_packet(&contract, SnapshotReadPacket::new(vec![]), execution)
                .expect("right source packet should materialize"),
        )
        .expect("right source record should canonicalize");

    assert_eq!(left, right);
    assert_eq!(left.digest(), right.digest());
}

#[test]
fn runtime_retains_source_materialization_record_in_diagnostics() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

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
        .expect("registered historical source should be admitted");

    let observation = runtime
        .materialize_source_packet(&contract, SnapshotReadPacket::new(vec![]), execution)
        .expect("registered historical source should materialize");
    let record = runtime
        .canonicalize_source_materialization_record(&contract, &observation)
        .expect("registered source materialization should canonicalize");

    let retained = runtime
        .diagnostics()
        .last_source_materialization_record()
        .expect("source materialization record should be retained");

    assert_eq!(retained, record);
    assert_eq!(
        runtime
            .diagnostics()
            .source_materialization_record_for_identity(record.record_identity())
            .expect("record should be queryable by identity"),
        record
    );
}

#[test]
fn runtime_replays_registered_source_materialization_record() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

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
        .expect("registered historical source should be admitted");

    let observation = runtime
        .materialize_source_packet(&contract, SnapshotReadPacket::new(vec![]), execution)
        .expect("registered historical source should materialize");
    let record = runtime
        .canonicalize_source_materialization_record(&contract, &observation)
        .expect("registered source materialization should canonicalize");

    let replayed = runtime
        .replay_source_materialization_record(&record, execution)
        .expect("registered source materialization should replay");

    assert_eq!(replayed, record);
}
