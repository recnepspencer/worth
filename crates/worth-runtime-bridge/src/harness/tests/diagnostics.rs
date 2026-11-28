use super::diagnostics_continuity_source::DiagnosticsContinuityLineageSource;
use super::support::build_runtime_with_aspects;
use crate::facade::{
    BridgeContinuityAuthorityBasis, BridgeDeliveryErrorKind, BridgeDeliveryIntent,
    BridgeLineageContext, BridgeReplayMode, BridgeRouteRequest, BridgeTruthViewSelector,
    HistoricalEvaluationDeclaration, SnapshotReadPacket,
};
use crate::harness::fixtures::ProfileTruth;
use crate::harness::fixtures::{NamedTruth, RecordingSignalBridgeSink};

#[test]
fn bridge_diagnostics_respect_route_record_retention_budget() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (first, _first) = truth.commit_name(execution, "alice");
    let (second, _second) = truth.commit_name(execution, "bob");
    let (third, _third) = truth.commit_name(execution, "carol");
    let runtime = crate::facade::RuntimeBridge::builder()
        .with_relational_source(truth.source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_policy(
            crate::facade::BridgeRuntimePolicy::development()
                .with_route_record_limit(2)
                .with_failure_record_limit(2),
        )
        .register_mapping(NamedTruth::mapping())
        .build()
        .expect("bridge runtime with bounded diagnostics retention");

    for commit in [first, second.clone(), third.clone()] {
        let route = runtime
            .plan_committed_patch(BridgeRouteRequest::for_commit(commit), execution)
            .expect("bridge should plan route for retention test");
        runtime
            .deliver_invalidation(route, execution)
            .expect("bridge should deliver route for retention test");
    }

    let route_records = runtime.diagnostics().route_records();
    assert_eq!(runtime.diagnostics().route_record_limit(), 2);
    assert_eq!(route_records.len(), 2);
    assert_eq!(route_records[0].source_commit().as_str(), second.as_str());
    assert_eq!(
        route_records
            .last()
            .expect("retained route record")
            .source_commit()
            .as_str(),
        third.as_str()
    );
}
#[test]
fn bridge_diagnostics_respect_failure_record_retention_budget() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (first, first_observation) = truth.commit_name(execution, "alice");
    let (second, second_observation) = truth.commit_name(execution, "bob");
    let (third, third_observation) = truth.commit_name(execution, "carol");
    let runtime = crate::facade::RuntimeBridge::builder()
        .with_relational_source(truth.source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_policy(
            crate::facade::BridgeRuntimePolicy::development()
                .with_route_record_limit(2)
                .with_failure_record_limit(2),
        )
        .register_mapping(NamedTruth::mapping())
        .build()
        .expect("bridge runtime with bounded diagnostics retention");

    for (commit, observation) in [
        (first, first_observation),
        (second.clone(), second_observation),
        (third.clone(), third_observation),
    ] {
        let route = runtime
            .plan_committed_patch(BridgeRouteRequest::for_commit(commit), execution)
            .expect("bridge should plan route for failure retention test");
        drop(observation);
        let error = runtime
            .deliver_invalidation(route, execution)
            .expect_err("bridge should fail delivery when the planned snapshot is absent");
        assert!(
            matches!(error.kind(), BridgeDeliveryErrorKind::SnapshotAcquisitionFailure(ref cause) if cause.kind() == crate::adapter::RelationalBridgeSourceErrorTag::Binding(crate::adapter::BridgeSourceBindingDenial::SnapshotNotBound))
        );
    }

    let failure_records = runtime.diagnostics().failure_records();
    assert_eq!(runtime.diagnostics().failure_record_limit(), 2);
    assert_eq!(failure_records.len(), 2);
    assert_eq!(failure_records[0].source_commit().as_str(), second.as_str());
    assert_eq!(
        failure_records
            .last()
            .expect("retained failure record")
            .source_commit()
            .as_str(),
        third.as_str()
    );
}
#[test]
fn bridge_route_record_captures_slice_counters_and_slice_entries() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit, _observation) = truth.commit_name(execution, "alice");
    let runtime = build_runtime_with_aspects(
        truth.source,
        RecordingSignalBridgeSink::default(),
        vec![NamedTruth::mapping()],
        vec![NamedTruth::aspect_mapping()],
    );

    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit), execution)
        .expect("bridge should plan route with fine-grained aspect registration");
    runtime
        .deliver_invalidation(route, execution)
        .expect("bridge should deliver route before diagnostics capture");

    let record = runtime
        .diagnostics()
        .last_route_record()
        .expect("bridge should capture a route record");

    assert_eq!(record.subscription_slices().len(), 1);
    assert_eq!(record.counters().planned_slice_match_count(), 1);
    assert_eq!(record.counters().slice_widening_count(), 0);
    assert_eq!(record.counters().slice_suppression_count(), 0);
}
#[test]
fn bridge_diagnostics_retain_canonical_continuity_records() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = ProfileTruth::new(execution);
    let source = truth.native.source.clone();
    let runtime = crate::facade::RuntimeBridge::builder()
        .with_relational_source(source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_continuity_lineage_source(DiagnosticsContinuityLineageSource)
        .with_policy(
            crate::facade::BridgeRuntimePolicy::development()
                .with_route_record_limit(2)
                .with_failure_record_limit(2),
        )
        .register_mapping(ProfileTruth::mapping())
        .register_aspect_mapping(ProfileTruth::aspect_mapping())
        .build()
        .expect("bridge runtime with bounded diagnostics retention");

    let route = runtime
        .plan_committed_patch_with_mapping_context(
            BridgeRouteRequest::for_commit(truth.commit.clone()),
            crate::facade::BridgeMappingContext::default().with_lineage_context(
                BridgeLineageContext::new(BridgeContinuityAuthorityBasis::new(
                    crate::truth_identity_fixtures::truth_branch_fixture("main"),
                    truth.observation.snapshot_identity().clone(),
                )),
            ),
            execution,
        )
        .expect("route should plan");
    let result = runtime
        .deliver_invalidation(route, execution)
        .expect("delivery should succeed");
    let route_record = runtime
        .diagnostics()
        .route_record_for_route_identity(result.result_summary().route_identity())
        .expect("route record should be retained");
    let requests = runtime
        .plan_continuity_requests(&route_record)
        .expect("continuity requests should plan");
    let packet = runtime
        .plan_historical_lineage_packet(&requests, execution)
        .expect("historical lineage packet should plan");
    let resolved = runtime
        .resolve_lineage_continuity(&packet)
        .expect("continuity should resolve");
    let artifact = runtime.lower_continuity_artifact(&resolved);
    let canonical = runtime.canonicalize_continuity_record(&route_record, &requests, &artifact);

    let retained = runtime.diagnostics().continuity_records();
    assert_eq!(retained.len(), 1);
    assert_eq!(
        retained[0].continuity_artifact_identity(),
        canonical.continuity_artifact_identity()
    );
    assert_eq!(
        runtime
            .diagnostics()
            .last_canonical_continuity_record()
            .expect("last continuity record")
            .continuity_resolution_digest(),
        canonical.continuity_resolution_digest()
    );
}

#[test]
fn bridge_diagnostics_retain_queryable_historical_records_by_record_and_decision_log_identity() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = NamedTruth::new();
    let (commit, _observation) = truth.commit_name(execution, "alice");
    let _head = truth.bind_main_head();
    let runtime = crate::facade::RuntimeBridge::builder()
        .with_relational_source(truth.source.clone())
        .with_truth_branch_head_source(truth.source.clone())
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_policy(
            crate::facade::BridgeRuntimePolicy::development()
                .with_route_record_limit(2)
                .with_failure_record_limit(2),
        )
        .register_mapping(NamedTruth::mapping())
        .build()
        .expect("bridge runtime with bounded diagnostics retention");

    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::historical_commit(
            crate::facade::TruthBranchIdentity::from_relational_branch_id("main"),
            commit,
        ),
        BridgeReplayMode::Enabled,
        runtime.policy().diagnostics_tier(),
        BridgeDeliveryIntent::PrepareSignalEvaluation,
    );
    let observation = runtime
        .materialize_truth_view_observation(
            runtime
                .plan_truth_view_packet(declaration, SnapshotReadPacket::new(vec![]), execution)
                .expect("historical declaration should plan"),
            execution,
        )
        .expect("historical declaration should materialize");
    let record = runtime.canonicalize_historical_evaluation_record(&observation);

    let diagnostics = runtime.diagnostics();
    let retained_by_record = diagnostics
        .historical_record_for_record_identity(record.record_identity())
        .expect("historical record should be queryable by record identity");
    let retained_by_decision_log = diagnostics
        .historical_record_for_decision_log_identity(record.decision_log().decision_log_identity())
        .expect("historical record should be queryable by decision-log identity");
    let handle = diagnostics.handle();

    assert_eq!(
        retained_by_record.record_identity(),
        record.record_identity()
    );
    assert_eq!(
        retained_by_decision_log
            .decision_log()
            .decision_log_identity(),
        record.decision_log().decision_log_identity()
    );
    assert_eq!(handle.historical_evaluation_records().len(), 1);
    assert_eq!(
        handle
            .historical_record_for_decision_log_identity(
                record.decision_log().decision_log_identity()
            )
            .expect("handle should query historical record by decision-log identity")
            .record_identity(),
        record.record_identity()
    );
}
