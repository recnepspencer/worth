use super::super::causal_inspection::certification::runtime_backed_causal_certification_bundle;
use super::super::intent_admission::intent_runtime_with_authority;
use super::super::support::*;
use crate::continuation_pipeline::runtime_backed_continuation_closure_summary;
use crate::harness::MilestoneFivePointTwoPreviewCertificationAdapter;
use crate::recovery_boundary::{WorthQueryRecoveryAction, WorthQueryRecoveryStopKind};
use crate::runtime::runtime_subscription_support_evidence_identity;
use crate::subscription::runtime_backed_subscription_certification_summary;
use crate::subscription::CoverageResolutionPosture;
use crate::subscription::QuerySubscriptionDeliveryCauseKind;
use crate::WorthQueryEvidenceIdentity;
use worth_runtime_bridge::facade::{
    BridgeAsyncCompletionClass, BridgeAsyncCompletionState, BridgeAsyncRequestTruthViewBasis,
    BridgeMixedCauseOrderingInput, BridgeMixedCauseOrderingLaneKind,
    BridgeMixedCauseOrderingRequest, BridgeSubscriptionDeliveryFamilyKind,
};

struct RemaskingSubscriptionActivation {
    projection: WorthQueryRuntimeRemaskProjection,
}

impl WorthQueryRuntimeSubscriptionActivationAdapter for RemaskingSubscriptionActivation {
    fn support_evidence_identity(&self) -> WorthQueryEvidenceIdentity {
        runtime_subscription_support_evidence_identity("test-subscription-activation")
    }

    fn remask_projection(
        &self,
        _view_name: &str,
        _activation: &crate::subscription::SubscriptionActivationInput,
    ) -> Option<WorthQueryRuntimeRemaskProjection> {
        Some(self.projection.clone())
    }

    fn admit_activation(
        &mut self,
        view_name: &str,
        activation: &crate::subscription::SubscriptionActivationInput,
    ) -> Result<SubscriptionActivationBoundaryReceipt, WorthQueryWorkspaceError> {
        let receipt = self.build_subscription_activation_receipt(view_name, activation);
        Ok(self.build_subscription_activation_boundary_receipt(view_name, activation, receipt))
    }
}

fn remasked_runtime(projection: WorthQueryRuntimeRemaskProjection) -> WorthQueryRuntime {
    test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(RemaskingSubscriptionActivation { projection })
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .build_backend_from_parts()
        .build()
        .expect("bridge-backed runtime should build")
}

#[test]
fn runtime_backed_reference_workload_exercises_temporal_async_preview_causal_and_follow_on_lanes() {
    let mut time_only_runtime = stateful_bridge_task_runtime();
    let time_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = time_only_runtime
        .declare_live_view(
            "tasks.phase26-time-only",
            task_live_request(),
            task_schema(),
        )
        .expect("time-only live view should declare");
    time_only_runtime
        .emit_time_only_delivery(
            time_view.name(),
            QuerySubscriptionDeliveryCauseKind::WindowEntry,
            "tick:phase26-time-only",
            false,
            true,
        )
        .expect("time-only delivery should emit");
    let _ = time_only_runtime.drain_patches(&time_view);
    let time_delivery = time_only_runtime
        .downstream_delivery(&time_view)
        .expect("time-only downstream delivery should project")
        .expect("time-only retained delivery should exist");

    let bridge = test_bridge();
    let truth_patch = canonical_truth_patch("truth-main", "snapshot-a", "commit-a", "patch-a");
    let truth_plus_time = authoritative_truth_plus_time_cause(&bridge, &truth_patch);
    let async_completion = admitted_async_completion(
        &bridge,
        worth_signal::facade::NodeId::new(301, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            TruthBranchIdentity::from_bridge_harness_label("truth-main"),
            TruthCommitIdentity::from_bridge_harness_label("commit-a"),
            TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
        ),
        64,
    );
    let ordering = bridge.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![
            BridgeMixedCauseOrderingInput::TruthPatch(truth_patch),
            BridgeMixedCauseOrderingInput::Temporal(truth_plus_time),
            BridgeMixedCauseOrderingInput::AsyncCompletion(async_completion),
        ],
    ));
    let window = bridge
        .plan_mixed_cause_delivery_window(
            &ordering,
            BridgeSubscriptionDeliveryFamilyKind::AdmittedCoalesced,
        )
        .expect("mixed-cause delivery should plan");
    let mut mixed_runtime = stateful_bridge_task_runtime();
    let mixed_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = mixed_runtime
        .declare_live_view(
            "tasks.phase26-mixed-cause",
            task_live_request(),
            task_schema(),
        )
        .expect("mixed-cause live view should declare");
    let (basis_digest, generation_digest) =
        live_subscription_async_identity(&mixed_runtime, mixed_view.name());
    mixed_runtime
        .project_async_result_state(
            mixed_view.name(),
            &WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Admitted(BridgeAsyncCompletionClass::Fulfilled),
                "async:phase26-current",
            ),
            &basis_digest,
            &generation_digest,
        )
        .expect("async current state should project");
    mixed_runtime
        .emit_mixed_cause_delivery(mixed_view.name(), &ordering, &window)
        .expect("mixed-cause delivery should emit");
    let _ = mixed_runtime.drain_patches(&mixed_view);
    let mixed_delivery = mixed_runtime
        .downstream_delivery(&mixed_view)
        .expect("mixed downstream delivery should project")
        .expect("mixed retained delivery should exist");

    let mut remask_runtime = remasked_runtime(WorthQueryRuntimeRemaskProjection::denied(
        WorthQueryRuntimeRemaskReasonKind::SchemaContextDrift,
        "policy:stable",
        "tenant-truth:stable",
        "tenant-schema:stable",
        "relationship-proof:stable",
        "schema-context:drifted",
    ));
    let remask_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = remask_runtime
        .declare_live_view(
            "tasks.phase26-remask-denied",
            task_live_request(),
            task_schema(),
        )
        .expect("remask live view should declare");
    remask_runtime
        .emit_time_only_delivery(
            remask_view.name(),
            QuerySubscriptionDeliveryCauseKind::Deadline,
            "tick:phase26-remask-denied",
            false,
            true,
        )
        .expect("remask time-only delivery should emit");
    let _ = remask_runtime.drain_patches(&remask_view);
    let remask_delivery = remask_runtime
        .downstream_delivery(&remask_view)
        .expect("remask downstream delivery should project")
        .expect("remask retained delivery should exist");

    let mut follow_on_runtime = intent_runtime_with_authority(TestIntentAuthority);
    let follow_on_origin_identity = test_write_adjacent_origin_identity(
        WorthQueryEffectWriteAdjacentTriggerClass::AsyncCompletion,
        "async-completion:cause:phase26-title",
    );
    let follow_on_view = follow_on_runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            "tasks.phase26-follow-on",
            task_live_request(),
            task_schema(),
        )
        .expect("follow-on live view should declare");
    let follow_on_effect = follow_on_runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(
            WorthQueryEffectDeclaration::write_intent(
                "effects.phase26-follow-on",
                WorthQueryEffectTrigger::live_view(
                    &follow_on_view,
                    test_aspect_touches(["title.value"]),
                ),
                "strategy.intent.reconcile",
            )
            .with_write_adjacent_trigger(
                WorthQueryEffectWriteAdjacentTriggerClass::AsyncCompletion,
                follow_on_origin_identity.clone(),
            ),
        )
        .expect("follow-on effect should declare");
    follow_on_runtime
        .write(test_update_string_aspect_command(
            crate::memory_workspace::admit_authored_entity_label("task-1"),
            "title.value",
            "title from phase26 async completion",
        ))
        .expect("follow-on write should queue pending intent");
    let follow_on_receipt = follow_on_runtime
        .next_effect_write_intent(&follow_on_effect, "1.0", "effect.intent.input.v1")
        .execute()
        .expect("follow-on intent should execute");

    let preview_artifact = MilestoneFivePointTwoPreviewCertificationAdapter::
        preview_session_basis_and_promotion_parity_artifact();
    let causal_bundle = runtime_backed_causal_certification_bundle();
    let subscription_summary = runtime_backed_subscription_certification_summary();
    let continuation_summary = runtime_backed_continuation_closure_summary();

    assert_eq!(
        time_delivery.delivery_class(),
        WorthQueryRuntimeDownstreamDeliveryClass::TimeOnly
    );
    assert_eq!(
        time_delivery
            .negotiate_runtime_resume(Some(time_delivery.basis_identity()))
            .kind(),
        WorthQueryRuntimeDownstreamResumePostureKind::RuntimeBackedAdmitted
    );
    assert_eq!(
        mixed_delivery.delivery_class(),
        WorthQueryRuntimeDownstreamDeliveryClass::MixedCause
    );
    assert!(mixed_delivery.mixed_cause_for_reporting().is_some());
    assert!(mixed_delivery.async_result_state_for_reporting().is_some());
    assert_eq!(
        remask_delivery.support_posture(),
        WorthQueryRuntimeDownstreamSupportPosture::Denied
    );
    assert!(remask_delivery.remask_for_reporting().is_some());
    assert_eq!(
        remask_delivery.durable_resume_posture().kind(),
        WorthQueryRuntimeDownstreamResumePostureKind::DurableDeferredDebt
    );
    assert_eq!(
        follow_on_receipt.write_adjacent_trigger_class(),
        WorthQueryEffectWriteAdjacentTriggerClass::AsyncCompletion
    );
    assert_eq!(
        follow_on_receipt
            .write_adjacent_trigger()
            .origin_evidence_identity(),
        &follow_on_origin_identity
    );
    assert!(
        preview_artifact
            .bundle_completeness_report
            .offline_analysis_ready
    );
    assert!(!preview_artifact.certification_bundle_digest.is_empty());
    assert_eq!(causal_bundle.hostile_row_count(), 10);
    assert_eq!(causal_bundle.scale_fixture_row_count(), 3);
    assert!(!causal_bundle.certification_bundle_digest().is_empty());
    assert_eq!(subscription_summary.certified_family_count, 1);
    assert_eq!(subscription_summary.hostile_row_coverage_count, 1);
    assert_eq!(
        subscription_summary.coverage_resolution_posture,
        CoverageResolutionPosture::IndexedCoverageSet
    );
    assert!(!subscription_summary.support_report_digest.is_empty());
    assert!(!subscription_summary.bridge_parity_digest.is_empty());
    assert!(!subscription_summary.diagnostic_bundle_digest.is_empty());
    assert!(!subscription_summary
        .lifecycle_certification_digest
        .is_empty());
    assert_eq!(
        continuation_summary.runtime_basis_identity_digest,
        continuation_summary.observed_basis_identity_digest
    );
    assert_eq!(
        continuation_summary.replay_recovery_stop_kind,
        WorthQueryRecoveryStopKind::ReplayDrift
    );
    assert_eq!(
        continuation_summary.replay_recovery_action,
        WorthQueryRecoveryAction::RefreshBasis
    );
    assert_eq!(
        continuation_summary.preview_recovery_stop_kind,
        WorthQueryRecoveryStopKind::PreviewCrossedResidue
    );
    assert_eq!(
        continuation_summary.preview_recovery_action,
        WorthQueryRecoveryAction::UseExplicitHandoff
    );
    assert!(continuation_summary.stale_completion_stop_is_typed);
}

#[test]
fn runtime_backed_closure_matrix_preserves_equivalent_and_distinct_public_meaning() {
    let runtime = stateful_bridge_task_runtime();
    let runtime_contract = runtime.public_downstream_delivery_contract();
    let workspace = stateful_bridge_task_runtime()
        .workspace("phase26.runtime-backed-closure")
        .expect("workspace should open");
    let workspace_contract = workspace.public_downstream_delivery_contract();
    let support_matrix = workspace.public_support_matrix();
    let support_row = support_matrix
        .row("downstream-delivery-contract")
        .expect("downstream delivery contract row should stay explicit");
    let causal_bundle = runtime_backed_causal_certification_bundle();
    let preview_artifact = MilestoneFivePointTwoPreviewCertificationAdapter::
        preview_session_basis_and_promotion_parity_artifact();

    assert_eq!(
        runtime_contract.contract_for_reporting(),
        workspace_contract.contract_for_reporting()
    );
    assert_eq!(
        support_row.support_contract_digest(),
        Some(workspace_contract.contract_for_reporting())
    );
    assert_eq!(
        runtime_contract.runtime_resume_support_posture(),
        WorthQueryLowerRuntimeSupportPosture::Admitted
    );
    assert_eq!(
        runtime_contract.durable_resume_support_posture(),
        WorthQueryLowerRuntimeSupportPosture::Deferred
    );
    assert_ne!(
        causal_bundle.certification_bundle_digest(),
        preview_artifact.certification_bundle_digest
    );
    assert_ne!(
        causal_bundle.representative_matrix_digest(),
        preview_artifact.coverage_matrix_digest
    );
}
