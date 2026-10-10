use worth_runtime_bridge::facade::{
    BridgePreviewResidueClass, BridgePreviewRetainedArtifactSchema, BridgePreviewSessionBasis,
    BridgePreviewSessionDeclaration, BridgePreviewSessionDeclarationIdentity,
    BridgePreviewSessionIdentity, BridgeRequestKind, BridgeRouteRequest,
    BridgeSignalBranchIdentity, BridgeSourceCapability, BridgeSourceCapabilitySet,
    BridgeSpeculativeBranchBinding, BridgeSpeculativeBranchBindingIdentity,
    BridgeTruthViewEvaluationRequest, BridgeTruthViewSelector, ChangeStreamDeclaration,
    RuntimeBridge, SnapshotReadContract, SnapshotReadPacket, SnapshotReadRequest,
    StreamCheckpointFrontierKind, StreamCheckpointPublicationMode, StreamCoalescingFamily,
    StreamCoalescingIntent, StreamConsumerShape, StreamDeliveryIntent,
    StreamDiagnosticsPolicyClass, StreamReplayMode, StreamResumeMode, StructuralFingerprintFamily,
    StructuralTruthViewBasis, TruthCommitIdentity,
};

use super::super::super::super::*;
use super::super::materialization::{
    bridge_runtime, causal_materialization_branch_identity, causal_materialization_commit_identity,
    causal_materialization_snapshot_identity, causal_truth_analysis_branch_identity,
    changed_reference_set, registered_source, registered_structural, request_for,
};
use super::writeback_support::retain_writeback_record_identities;
use lower_runtime_slot_references::bridge_request_with_lower_runtime_slot_references;

mod lower_runtime_slot_references;

pub(super) fn artifact_with_lower_runtime_slot_evidence(
    commit_identity: TruthCommitIdentity,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> QueryCausalInspectionArtifact {
    let runtime = bridge_runtime();
    let routed = runtime
        .route(commit_identity.clone(), resource_request)
        .unwrap();
    let retained_evidence =
        retain_lower_runtime_slot_evidence(&runtime, &commit_identity, resource_request);
    let reference_set = changed_reference_set(routed.route_identity());
    let flow = admit_causal_inspection(request_for(
        reference_set,
        CausalInspectionRichness::ReferenceOnly,
    ));
    let CausalInspectionProofFlow::Admitted(admitted) = flow else {
        panic!("slot evidence fixture should admit");
    };
    let bridge_request = bridge_request_with_lower_runtime_slot_references(
        &admitted,
        &routed,
        &retained_evidence,
        &commit_identity,
    );
    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(bridge_request)
        .expect("bridge envelope should assemble with lower runtime slots");

    materialize_admitted_causal_inspection(
        &admitted,
        &envelope,
        CausalInspectionRedactionPolicy::PreserveDetail,
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
    )
    .expect("slot evidence materialization should consume bridge envelope")
}

struct RetainedLowerRuntimeSlotEvidence {
    historical_evaluation_record_identity: String,
    preview_execution_record_identity: String,
    preview_discard_record_identity: String,
    source_materialization_record_identity: String,
    structural_remap_record_identity: String,
    stream_replay_record_identity: String,
    writeback_admission_record_identity: String,
    writeback_mapper_envelope_identity: String,
    writeback_mapped_family_input_identity: String,
    writeback_mapper_record_identity: String,
    writeback_execution_record_identity: String,
    writeback_replay_record_identity: String,
}

fn retain_lower_runtime_slot_evidence(
    runtime: &RuntimeBridge,
    commit_identity: &TruthCommitIdentity,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> RetainedLowerRuntimeSlotEvidence {
    let (preview_execution_record_identity, preview_discard_record_identity) =
        retain_preview_record_identities(runtime, commit_identity);
    let writeback = retain_writeback_record_identities(runtime, commit_identity);
    RetainedLowerRuntimeSlotEvidence {
        historical_evaluation_record_identity: retain_historical_evaluation_record_identity(
            runtime,
            resource_request,
        ),
        preview_execution_record_identity,
        preview_discard_record_identity,
        source_materialization_record_identity: retain_source_materialization_record_identity(
            runtime,
            resource_request,
        ),
        structural_remap_record_identity: retain_structural_remap_record_identity(
            runtime,
            resource_request,
        ),
        stream_replay_record_identity: retain_stream_replay_record_identity(
            runtime,
            commit_identity,
            resource_request,
        ),
        writeback_admission_record_identity: writeback.admission_record_identity,
        writeback_mapper_envelope_identity: writeback.mapper_envelope_identity,
        writeback_mapped_family_input_identity: writeback.mapped_family_input_identity,
        writeback_mapper_record_identity: writeback.mapper_record_identity,
        writeback_execution_record_identity: writeback.execution_record_identity,
        writeback_replay_record_identity: writeback.replay_record_identity,
    }
}

fn retain_historical_evaluation_record_identity(
    runtime: &RuntimeBridge,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> String {
    runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_branch_head(
                causal_materialization_branch_identity(),
            ),
            resource_request,
        )
        .expect("historical evaluation should retain evidence")
        .record()
        .record_identity()
        .bridge_admission_evidence()
        .terminal_projection_for_reporting()
        .to_string()
}

fn retain_preview_record_identities(
    runtime: &RuntimeBridge,
    commit_identity: &TruthCommitIdentity,
) -> (String, String) {
    let suffix = lower_runtime_slot_suffix(commit_identity);
    let preview_admitted = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::from_stable_name(format!("preview-session:{suffix}")),
            preview_declaration(&suffix),
        )
        .expect("preview declaration should admit");
    let (preview_active, preview_execution) =
        runtime.activate_preview_session(preview_admitted, 3, 1, 2);
    let (_, preview_discard) = runtime
        .discard_preview_session(
            preview_active,
            &preview_execution,
            vec![
                BridgePreviewResidueClass::PreviewExecutionRetained,
                BridgePreviewResidueClass::ReplayRetainedNonAuthoritative,
                BridgePreviewResidueClass::TemporaryDiagnosticsResidue,
            ],
        )
        .expect("preview discard should retain evidence");
    (
        preview_execution
            .record_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting()
            .to_string(),
        preview_discard
            .record_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting()
            .to_string(),
    )
}

fn retain_source_materialization_record_identity(
    runtime: &RuntimeBridge,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> String {
    let source_contract = runtime
        .admit_source(registered_source(
            "source:causal-materialization-history",
            BridgeTruthViewSelector::historical_commit(
                causal_materialization_branch_identity(),
                causal_materialization_commit_identity(),
            ),
            vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::HistoricalRead,
                BridgeSourceCapability::BranchRead,
                BridgeSourceCapability::ReplayContinuityRead,
            ],
        ))
        .expect("source declaration should admit");
    let source_observation = runtime
        .materialize_source_packet(
            &source_contract,
            SnapshotReadPacket::new(vec![]),
            resource_request,
        )
        .expect("source packet should materialize");
    runtime
        .canonicalize_source_materialization_record(&source_contract, &source_observation)
        .expect("source materialization should retain evidence")
        .record_identity()
        .bridge_admission_evidence()
        .terminal_projection_for_reporting()
        .to_string()
}

fn retain_structural_remap_record_identity(
    runtime: &RuntimeBridge,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> String {
    let structural_contract = runtime
        .admit_structural_comparison(registered_structural(
            "structural:causal-materialization-snapshot",
            StructuralFingerprintFamily::TopologyFingerprint,
            StructuralTruthViewBasis::explicit_snapshot(BridgeTruthViewSelector::branch_snapshot(
                causal_materialization_branch_identity(),
                causal_materialization_snapshot_identity(),
            )),
        ))
        .expect("structural declaration should admit");
    let structural_read = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_coarse(
        "entity-1",
        SnapshotReadContract::scalar(
            worth_foundational::facade::AspectKey::new("profile")
                .expect("valid snapshot aspect key"),
            worth_foundational::facade::ScalarAspectType::String,
        ),
    )]);
    let structural_planned = runtime
        .plan_structural_match_packet_set_from_read_packets(
            &structural_contract,
            structural_read.clone(),
            vec![structural_read],
            resource_request,
        )
        .expect("structural packets should plan");
    let structural_reduced = runtime
        .reduce_structural_match_set(&structural_planned)
        .expect("structural set should reduce");
    let structural_artifact = runtime
        .publish_structural_remap_artifact(&structural_reduced)
        .expect("structural artifact should publish");
    runtime
        .canonicalize_structural_remap_record(
            &structural_contract,
            &structural_planned,
            &structural_reduced,
            &structural_artifact,
        )
        .record_identity()
        .bridge_admission_evidence()
        .terminal_projection_for_reporting()
        .to_string()
}

fn retain_stream_replay_record_identity(
    runtime: &RuntimeBridge,
    commit_identity: &TruthCommitIdentity,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> String {
    let stream_protocol = runtime
        .validate_change_stream_declaration(ChangeStreamDeclaration::new(
            StreamConsumerShape::RoutingConsumer,
            StreamResumeMode::FromCheckpointOnly,
            StreamCheckpointPublicationMode::PublishEveryWindow,
            StreamCoalescingIntent::Prefer(StreamCoalescingFamily::RoutingWindowCoalescing),
            StreamReplayMode::Enabled,
            StreamDeliveryIntent::RouteInvalidations,
            StreamDiagnosticsPolicyClass::Standard,
        ))
        .expect("stream declaration should validate");
    let stream_contract = runtime
        .resolve_change_stream_consumer_contract(&stream_protocol)
        .expect("stream contract should resolve");
    let stream_envelope = runtime
        .ingest_committed_patch(
            BridgeRouteRequest::for_commit(stream_replay_commit_identity(commit_identity)),
            resource_request,
        )
        .expect("stream commit should ingest");
    let stream_window = runtime
        .plan_change_stream_window(&stream_contract, vec![stream_envelope])
        .expect("stream window should plan");
    let stream_checkpoint = runtime.publish_consumer_checkpoint(
        &stream_contract,
        &stream_window,
        StreamCheckpointFrontierKind::ContiguousFrontier,
    );
    runtime
        .canonicalize_stream_replay_record(&stream_contract, &stream_window, &stream_checkpoint)
        .expect("stream replay should retain evidence")
        .replay_record_identity()
        .bridge_admission_evidence()
        .terminal_projection_for_reporting()
        .to_string()
}

fn stream_replay_commit_identity(commit_identity: &TruthCommitIdentity) -> TruthCommitIdentity {
    let commit_id = commit_identity
        .relational_commit_id()
        .expect("causal stream replay fixture must carry relational commit authority");
    TruthCommitIdentity::from_relational_commit_id(commit_id + 10_000)
}

fn preview_declaration(suffix: &str) -> BridgePreviewSessionDeclaration {
    BridgePreviewSessionDeclaration::new(
        BridgePreviewSessionDeclarationIdentity::from_stable_name(format!("preview:slot:{suffix}")),
        BridgeRequestKind::Preview,
        BridgeSpeculativeBranchBinding::new(
            BridgeSpeculativeBranchBindingIdentity::from_stable_name(format!(
                "binding:slot:{suffix}"
            )),
            causal_truth_analysis_branch_identity(),
            BridgeSignalBranchIdentity::from_stable_name("signal:analysis"),
        ),
        BridgePreviewSessionBasis::new(
            BridgeTruthViewSelector::branch_snapshot(
                causal_truth_analysis_branch_identity(),
                causal_materialization_snapshot_identity(),
            ),
            BridgeSourceCapabilitySet::new(vec![BridgeSourceCapability::SnapshotRead]),
            BridgePreviewRetainedArtifactSchema::PreviewLifecycleArtifactsV1,
        ),
    )
}

fn lower_runtime_slot_suffix(commit_identity: &TruthCommitIdentity) -> String {
    let commit_id = commit_identity
        .relational_commit_id()
        .expect("causal lower-runtime slot fixture must carry relational commit authority");
    format!("commit-{commit_id}")
}
