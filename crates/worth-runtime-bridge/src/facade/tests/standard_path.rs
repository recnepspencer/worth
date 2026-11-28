use super::{
    registered_source, runtime, BridgeMappingId, BridgeMappingRegistration, BridgeRuntimePolicy,
    BridgeTruthViewSelector, CoarseRoutingMode, MappingSelector, RuntimeBridge,
    SignalInvalidationScope, StaticSink, StaticSource, StaticSourceAdapter, TruthPatchScope,
};
#[test]
fn standard_builder_aliases_build_runtime() {
    let retention = crate::facade::BridgeConditionalRetentionBudget {
        maximum_managed_clocks: Some(2),
        ..crate::facade::BridgeConditionalRetentionBudget::development()
    };
    let runtime = RuntimeBridge::builder()
        .with_policy(BridgeRuntimePolicy::default())
        .with_conditional_retention_budget(retention)
        .with_truth_source(StaticSource)
        .with_source_adapter(StaticSourceAdapter)
        .with_truth_branch_head_source(StaticSource)
        .with_compute_sink(StaticSink)
        .register_source(registered_source(
            "source:analysis-snapshot",
            BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            vec![
                crate::facade::BridgeSourceCapability::SnapshotRead,
                crate::facade::BridgeSourceCapability::BranchRead,
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
        .expect("builder aliases should produce a valid runtime");

    assert_eq!(runtime.policy().conditional_retention(), retention);
}

#[test]
fn standard_route_flows_from_commit_string_to_evaluation_target() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());

    let routed = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit-std"),
            execution,
        )
        .expect("standard route should succeed");
    let evaluation = runtime
        .evaluate_current(routed.target(), execution)
        .expect("evaluation target should prepare current evaluation");

    assert!(
        crate::truth_identity_fixtures::truth_commit_fixture_matches(
            routed.result().result_summary().source_commit(),
            "commit-std"
        )
    );
    assert!(
        crate::truth_identity_fixtures::truth_snapshot_fixture_matches(
            routed.result().receipt().snapshot_identity(),
            "snapshot-a"
        )
    );
    assert!(
        crate::truth_identity_fixtures::truth_snapshot_fixture_matches(
            evaluation.snapshot().snapshot_identity(),
            "snapshot-a"
        )
    );
    assert_eq!(runtime.diagnostics().route_records().len(), 1);
    assert!(matches!(
        runtime.diagnostics().explain_last(),
        Some(crate::facade::BridgeStandardDiagnosticsExplanation::Route(
            _
        ))
    ));
}

#[test]
fn standard_truth_view_evaluation_flows_from_branch_head_request() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());

    let evaluation = runtime
        .evaluate(
            crate::facade::BridgeTruthViewEvaluationRequest::for_branch_head(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            ),
            execution,
        )
        .expect("branch-head evaluation should succeed");

    assert!(
        crate::truth_identity_fixtures::truth_snapshot_fixture_matches(
            evaluation.snapshot_identity(),
            "snapshot-a"
        )
    );
    assert!(
        crate::truth_identity_fixtures::truth_snapshot_fixture_matches(
            evaluation.record().decision_log().snapshot_identity(),
            "snapshot-a"
        )
    );
    assert_eq!(
        runtime
            .diagnostics()
            .last_historical_evaluation_record()
            .expect("historical evaluation should be retained")
            .record_identity(),
        evaluation.record().record_identity()
    );
    assert_eq!(
        runtime
            .diagnostics()
            .explain_last_evaluation()
            .expect("standard diagnostics should explain the last evaluation")
            .record_identity(),
        evaluation.record().record_identity()
    );
    assert_eq!(
        runtime
            .diagnostics()
            .explain_evaluation(evaluation.record().record_identity())
            .expect("standard diagnostics should explain a named evaluation")
            .record_identity(),
        evaluation.record().record_identity()
    );
    assert!(matches!(
        runtime.diagnostics().explain_last(),
        Some(crate::facade::BridgeStandardDiagnosticsExplanation::Evaluation(_))
    ));
}
