#[test]
fn bridge_bulk_planning_rejects_empty_workloads() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
    let runtime = build_runtime(
        source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
    );

    let error = runtime
        .plan_bulk_workload(BridgeBulkWorkloadRequest::new(vec![]), execution)
        .expect_err("empty bulk workload should be rejected");

    assert_eq!(
        error.kind(),
        crate::error::BridgeRouteErrorKind::EmptyBulkWorkloadRequest
    );
}

#[test]
fn bridge_bulk_planning_identity_is_stable_across_input_order() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let left_source = InMemoryRelationalBridgeSource::default();
    left_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    left_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-b"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    left_source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    left_source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
        "bob",
    ));
    let left_runtime = build_runtime(
        left_source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
    );

    let right_source = InMemoryRelationalBridgeSource::default();
    right_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    right_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-b"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    right_source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    right_source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
        "bob",
    ));
    let right_runtime = build_runtime(
        right_source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
    );

    let left = left_runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                )),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
                )),
            ]),
            execution,
        )
        .expect("left bulk workload should plan");
    let right = right_runtime
        .plan_bulk_workload(
            BridgeBulkWorkloadRequest::new(vec![
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
                )),
                BridgeBulkWorkloadSegment::new(BridgeRouteRequest::for_commit(
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                )),
            ]),
            execution,
        )
        .expect("right bulk workload should plan");

    assert_eq!(left.workload_identity(), right.workload_identity());
    assert_eq!(
        left.canonical_planning_identity(),
        right.canonical_planning_identity()
    );
    assert_eq!(
        left.admission_profile_identity(),
        right.admission_profile_identity()
    );
    assert_eq!(
        left.canonical_request().digest(),
        right.canonical_request().digest()
    );
    assert_eq!(
        left.normalized_summary().digest(),
        right.normalized_summary().digest()
    );
    assert_eq!(left.summary().digest(), right.summary().digest());
    assert_eq!(
        left.planned_routes()
            .iter()
            .map(|route| route.route_identity().as_str())
            .collect::<Vec<_>>(),
        vec![
            right.planned_routes()[0].route_identity().as_str(),
            right.planned_routes()[1].route_identity().as_str(),
        ]
    );
}

#[test]
fn bridge_bulk_planning_separates_canonical_plan_identity_from_admission_profile_identity() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let standard_source = InMemoryRelationalBridgeSource::default();
    standard_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    standard_source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let standard_runtime = crate::facade::RuntimeBridgeBuilder::new()
        .with_policy(
            BridgeRuntimePolicy::development()
                .with_route_record_limit(128)
                .with_failure_record_limit(128),
        )
        .with_relational_source(standard_source.clone())
        .with_truth_branch_head_source(standard_source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .register_mapping(registration())
        .build()
        .expect("standard runtime should build");

    let exhaustive_source = InMemoryRelationalBridgeSource::default();
    exhaustive_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    exhaustive_source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let exhaustive_runtime = crate::facade::RuntimeBridgeBuilder::new()
        .with_policy(
            BridgeRuntimePolicy::forensic()
                .with_route_record_limit(128)
                .with_failure_record_limit(128),
        )
        .with_relational_source(exhaustive_source.clone())
        .with_truth_branch_head_source(exhaustive_source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .register_mapping(registration())
        .build()
        .expect("exhaustive runtime should build");

    let request = BridgeBulkWorkloadRequest::new(vec![BridgeBulkWorkloadSegment::new(
        BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
            "commit-a",
        )),
    )]);
    let standard = standard_runtime
        .plan_bulk_workload(request.clone(), execution)
        .expect("standard workload should plan");
    let exhaustive = exhaustive_runtime
        .plan_bulk_workload(request, execution)
        .expect("exhaustive workload should plan");

    assert_eq!(
        standard.canonical_planning_identity(),
        exhaustive.canonical_planning_identity()
    );
    assert_eq!(standard.workload_identity(), exhaustive.workload_identity());
    assert_eq!(
        standard.canonical_request().digest(),
        exhaustive.canonical_request().digest()
    );
    assert_eq!(
        standard.normalized_summary().digest(),
        exhaustive.normalized_summary().digest()
    );
    assert_ne!(
        standard.admission_profile_identity(),
        exhaustive.admission_profile_identity()
    );
    assert_eq!(
        standard.planned_routes()[0].source_commit().as_str(),
        exhaustive.planned_routes()[0].source_commit().as_str()
    );
    assert_eq!(
        BridgeDiagnosticsTier::Standard,
        standard_runtime.policy().diagnostics_tier()
    );
    assert_eq!(
        BridgeDiagnosticsTier::Exhaustive,
        exhaustive_runtime.policy().diagnostics_tier()
    );
}

#[test]
fn bridge_bulk_planning_identity_uses_frozen_registration_identity_without_leaking_signal_scope() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let left_source = InMemoryRelationalBridgeSource::default();
    left_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    left_source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let left_runtime = build_runtime(
        left_source,
        RecordingSignalBridgeSink::default(),
        vec![registration_with_signal_scope(
            crate::facade::SignalInvalidationScope::admit_bridge_owned("signal.profile.left"),
        )],
    );

    let right_source = InMemoryRelationalBridgeSource::default();
    right_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    right_source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let right_runtime = build_runtime(
        right_source,
        RecordingSignalBridgeSink::default(),
        vec![registration_with_signal_scope(
            crate::facade::SignalInvalidationScope::admit_bridge_owned("signal.profile.right"),
        )],
    );

    let request = BridgeBulkWorkloadRequest::new(vec![BridgeBulkWorkloadSegment::new(
        BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
            "commit-a",
        )),
    )]);
    let left = left_runtime
        .plan_bulk_workload(request.clone(), execution)
        .expect("left workload should plan");
    let right = right_runtime
        .plan_bulk_workload(request, execution)
        .expect("right workload should plan");

    assert_ne!(
        left.canonical_planning_identity(),
        right.canonical_planning_identity()
    );
    assert!(left
        .canonical_planning_identity()
        .as_str()
        .starts_with("bulk-planning-identity:sha256:"));
    assert!(right
        .canonical_planning_identity()
        .as_str()
        .starts_with("bulk-planning-identity:sha256:"));
    assert!(!left
        .canonical_planning_identity()
        .as_str()
        .contains("signal.profile.left"));
    assert!(!right
        .canonical_planning_identity()
        .as_str()
        .contains("signal.profile.right"));
}

fn registration_with_signal_scope(
    signal_scope: crate::facade::SignalInvalidationScope,
) -> crate::facade::BridgeMappingRegistration {
    crate::facade::BridgeMappingRegistration::new(
        crate::facade::BridgeMappingId::admit_bridge_owned("profile-name"),
        crate::facade::TruthPatchScope::for_entity_field(
            crate::facade::MappingSelector::exact("user"),
            worth_foundational::facade::AspectKey::new("profile")
                .expect("valid bulk planning aspect key"),
            worth_foundational::facade::FieldKey::new("name".to_owned())
                .expect("valid bulk planning field key"),
        ),
        crate::snapshot::SnapshotReadContract::scalar(
            worth_foundational::facade::AspectKey::new("profile")
                .expect("valid bulk planning aspect key"),
            worth_foundational::facade::ScalarAspectType::String,
        ),
        signal_scope,
        crate::facade::CoarseRoutingMode::Direct,
    )
}

use super::*;
