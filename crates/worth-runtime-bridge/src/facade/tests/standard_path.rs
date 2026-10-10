use super::{
    registered_source, runtime, BridgeMappingId, BridgeMappingRegistration, BridgeRuntimePolicy,
    BridgeTruthViewSelector, CoarseRoutingMode, MappingSelector, RuntimeBridge,
    SignalInvalidationScope, StaticSink, StaticSource, StaticSourceAdapter, TruthPatchScope,
};
use crate::facade::BridgeSpeculativeSessionRequest;
use crate::speculation::{
    BridgePreviewResidueClass, BridgePreviewRetainedArtifactSchema, BridgePreviewSessionBasis,
    BridgePreviewSessionDeclaration, BridgePreviewSessionDeclarationIdentity,
    BridgePreviewSessionIdentity, BridgeRequestKind, BridgeSignalBranchIdentity,
    BridgeSpeculativeBranchBinding, BridgeSpeculativeBranchBindingIdentity,
};

fn preview_declaration() -> BridgePreviewSessionDeclaration {
    BridgePreviewSessionDeclaration::new(
        BridgePreviewSessionDeclarationIdentity::admit_bridge_owned("preview:analysis"),
        BridgeRequestKind::Preview,
        BridgeSpeculativeBranchBinding::new(
            BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned("binding:analysis"),
            crate::truth_identity_fixtures::truth_branch_fixture("truth:analysis"),
            BridgeSignalBranchIdentity::admit_bridge_owned("signal:analysis"),
        ),
        BridgePreviewSessionBasis::new(
            BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("truth:analysis"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            crate::facade::BridgeSourceCapabilitySet::new(vec![
                crate::facade::BridgeSourceCapability::SnapshotRead,
                crate::facade::BridgeSourceCapability::BranchRead,
            ]),
            BridgePreviewRetainedArtifactSchema::PreviewLifecycleArtifactsV1,
        ),
    )
}

#[test]
fn standard_builder_aliases_build_runtime() {
    let runtime = RuntimeBridge::builder()
        .with_policy(BridgeRuntimePolicy::default())
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

    assert_eq!(runtime.policy(), &BridgeRuntimePolicy::default());
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

#[test]
fn standard_speculation_flow_activates_discards_and_promotes() {
    let runtime = runtime(BridgeRuntimePolicy::default());

    let comparison = runtime
        .speculate(BridgeSpeculativeSessionRequest::new(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:std-compare"),
            preview_declaration(),
            3,
            1,
            2,
        ))
        .expect("standard speculation should activate")
        .compare_to_main();

    assert_eq!(
        comparison
            .main_evaluation_request(crate::truth_identity_fixtures::truth_branch_fixture("main"))
            .selector(),
        &crate::facade::BridgeTruthViewSelector::branch_head(
            crate::truth_identity_fixtures::truth_branch_fixture("main")
        )
    );
    assert_eq!(
        comparison.speculative_evaluation_request().selector(),
        &crate::facade::BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("truth:analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        )
    );

    let discarded = runtime
        .speculate(BridgeSpeculativeSessionRequest::new(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:std-discard"),
            preview_declaration(),
            3,
            1,
            2,
        ))
        .expect("standard speculation should activate")
        .discard(vec![
            BridgePreviewResidueClass::PreviewExecutionRetained,
            BridgePreviewResidueClass::ReplayRetainedNonAuthoritative,
            BridgePreviewResidueClass::TemporaryDiagnosticsResidue,
        ])
        .expect("standard speculation discard should succeed");

    assert_eq!(
        discarded.session().lifecycle_state_kind(),
        crate::facade::BridgePreviewLifecycleStateKind::Discarded
    );

    let promoted = runtime
        .speculate(BridgeSpeculativeSessionRequest::new(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:std-promote"),
            preview_declaration(),
            3,
            1,
            2,
        ))
        .expect("standard speculation should activate")
        .promote()
        .expect("standard promotion should succeed");

    assert_eq!(
        promoted.session().lifecycle_state_kind(),
        crate::facade::BridgePreviewLifecycleStateKind::Promoted
    );
    assert_eq!(runtime.diagnostics().preview_execution_records().len(), 3);
    assert_eq!(runtime.diagnostics().preview_discard_records().len(), 1);
    assert_eq!(runtime.diagnostics().preview_promotion_records().len(), 1);
    let promoted_session_identity =
        BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:std-promote");
    assert!(matches!(
        runtime
            .diagnostics()
            .explain_session(&promoted_session_identity),
        Some(crate::facade::BridgeStandardSessionExplanation::PreviewPromotion(_))
    ));
    assert!(matches!(
        runtime.diagnostics().explain_last(),
        Some(crate::facade::BridgeStandardDiagnosticsExplanation::PreviewPromotion(_))
    ));
}

#[test]
fn preview_liveness_observer_tracks_handle_disposal_without_retaining_session() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let handle = runtime
        .speculate(BridgeSpeculativeSessionRequest::new(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:liveness-discard"),
            preview_declaration(),
            1,
            0,
            0,
        ))
        .expect("standard speculation should activate");
    let observer = handle.liveness_observer();
    let guard = observer
        .admit_active_session()
        .expect("the live handle must admit a lifecycle guard");
    drop(guard);

    handle
        .discard(Vec::new())
        .expect("zero-residue preview discard should succeed");
    assert!(observer.admit_active_session().is_none());

    let abandoned = runtime
        .speculate(BridgeSpeculativeSessionRequest::new(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:liveness-drop"),
            preview_declaration(),
            1,
            0,
            0,
        ))
        .expect("standard speculation should activate");
    let abandoned_observer = abandoned.liveness_observer();
    drop(abandoned);

    assert!(abandoned_observer.admit_active_session().is_none());
}
