use crate::diagnostics::BridgeFailureLocalizationRequest;
use crate::facade::tests::source::support::{
    admit_request_response_completion, admitted_authoritative_request_response_completion,
    authoritative_writeback_request, denied_request_response_completion_with_displacing_identity,
    retry_lineage_after_cancellation,
};
use crate::facade::{
    BridgeAsyncCompletionSupersessionClassificationRequest, BridgeMixedCauseOrderingInput,
    BridgeMixedCauseOrderingLaneKind, BridgeMixedCauseOrderingRequest,
    BridgeTemporalAsyncCertificationAsyncSectionInput,
    BridgeTemporalAsyncCertificationBundleRequest,
    BridgeTemporalAsyncCertificationDiagnosticsRichness,
    BridgeTemporalAsyncOfflineDiagnosisBundleSealed, RuntimeBridge,
};
use worth_signal::facade::NodeId;

use super::super::*;

pub(crate) fn temporal_async_bundle_equivalent_comparison(
    seed: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::facade::BridgeTemporalAsyncCertificationBundleComparison {
    let left_runtime = crate::facade::tests::source::support::runtime_with_authority();
    let right_runtime = crate::facade::tests::source::support::runtime_with_authority();
    let left = left_runtime.seal_temporal_async_certification_bundle(temporal_async_bundle_draft(
        &left_runtime,
        BridgeTemporalAsyncCertificationDiagnosticsRichness::Minimal,
        &format!("temporal-commit-{seed}"),
        &format!("temporal-snapshot-{seed}"),
        resource_request,
    ));
    let right =
        right_runtime.seal_temporal_async_certification_bundle(temporal_async_bundle_draft(
            &right_runtime,
            BridgeTemporalAsyncCertificationDiagnosticsRichness::Minimal,
            &format!("temporal-commit-{seed}"),
            &format!("temporal-snapshot-{seed}"),
            resource_request,
        ));
    left_runtime.compare_temporal_async_certification_bundles(&left, &right)
}

pub(crate) fn temporal_async_bundle_diagnostics_delta_comparison(
    seed: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::facade::BridgeTemporalAsyncCertificationBundleComparison {
    let base_runtime = crate::facade::tests::source::support::runtime_with_authority();
    let rich_runtime = crate::facade::tests::source::support::runtime_with_authority();
    let base = base_runtime.seal_temporal_async_certification_bundle(temporal_async_bundle_draft(
        &base_runtime,
        BridgeTemporalAsyncCertificationDiagnosticsRichness::Minimal,
        &format!("temporal-commit-{seed}"),
        &format!("temporal-snapshot-{seed}"),
        resource_request,
    ));
    let rich = rich_runtime.seal_temporal_async_certification_bundle(temporal_async_bundle_draft(
        &rich_runtime,
        BridgeTemporalAsyncCertificationDiagnosticsRichness::Rich,
        &format!("temporal-commit-{seed}"),
        &format!("temporal-snapshot-{seed}"),
        resource_request,
    ));
    base_runtime.compare_temporal_async_certification_bundles(&base, &rich)
}

pub(crate) fn temporal_async_bundle_divergent_comparison(
    seed: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::facade::BridgeTemporalAsyncCertificationBundleComparison {
    let left_runtime = crate::facade::tests::source::support::runtime_with_authority();
    let right_runtime = crate::facade::tests::source::support::runtime_with_authority();
    let left = left_runtime.seal_temporal_async_certification_bundle(temporal_async_bundle_draft(
        &left_runtime,
        BridgeTemporalAsyncCertificationDiagnosticsRichness::Minimal,
        &format!("temporal-commit-{seed}-left"),
        &format!("temporal-snapshot-{seed}-left"),
        resource_request,
    ));
    let right =
        right_runtime.seal_temporal_async_certification_bundle(temporal_async_bundle_draft(
            &right_runtime,
            BridgeTemporalAsyncCertificationDiagnosticsRichness::Minimal,
            &format!("temporal-commit-{seed}-right"),
            &format!("temporal-snapshot-{seed}-right"),
            resource_request,
        ));
    left_runtime.compare_temporal_async_certification_bundles(&left, &right)
}

pub(crate) fn active_detail_subscription_in_runtime(
    runtime: &RuntimeBridge,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::facade::BridgeActiveSubscription {
    let ready = activation_ready_detail_subscription_in_runtime(runtime, resource_request);
    let cost_profile = runtime
        .admit_subscription_delivery_cost_profile(
            BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
            4,
            4,
            2,
        )
        .expect("cost profile should admit");
    runtime.activate_subscription_delivery(
        ready,
        cost_profile,
        canonical_consumer_contract(runtime),
    )
}

pub(crate) fn temporal_async_bundle_draft(
    runtime: &RuntimeBridge,
    diagnostics_richness: BridgeTemporalAsyncCertificationDiagnosticsRichness,
    temporal_commit: &str,
    temporal_snapshot: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::facade::BridgeTemporalAsyncCertificationBundleDraft {
    let active = active_detail_subscription_in_runtime(runtime, resource_request);
    let shared_bundle = shared_delivery_bundle(
        runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );
    temporal_async_bundle_draft_with_shared_bundle(
        runtime,
        active,
        shared_bundle,
        diagnostics_richness,
        temporal_commit,
        temporal_snapshot,
    )
}

pub(crate) fn temporal_async_bundle_draft_with_shared_bundle(
    runtime: &RuntimeBridge,
    active: crate::facade::BridgeActiveSubscription,
    shared_bundle: crate::facade::BridgeSharedConsumerDeliveryBundleSealed,
    diagnostics_richness: BridgeTemporalAsyncCertificationDiagnosticsRichness,
    temporal_commit: &str,
    temporal_snapshot: &str,
) -> crate::facade::BridgeTemporalAsyncCertificationBundleDraft {
    let temporal_basis = admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_commit_fixture(temporal_commit),
        crate::truth_identity_fixtures::truth_snapshot_fixture(temporal_snapshot),
    ));
    let completion_report = admit_request_response_completion(
        runtime,
        NodeId::new(701, 0),
        crate::facade::BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("async-commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("async-snapshot-a"),
        ),
        64,
    );
    let admitted_completion = completion_report
        .admitted_completion()
        .expect("async completion should admit")
        .clone();
    let (superseded_denial, displacing_request) =
        denied_request_response_completion_with_displacing_identity(
            runtime,
            NodeId::new(703, 0),
            crate::facade::BridgeAsyncRequestTruthViewBasis::authoritative(
                crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
                crate::truth_identity_fixtures::truth_commit_fixture("async-commit-c1"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("async-snapshot-c1"),
            ),
            crate::facade::BridgeAsyncRequestTruthViewBasis::authoritative(
                crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
                crate::truth_identity_fixtures::truth_commit_fixture("async-commit-c1"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("async-snapshot-c1"),
            ),
        );
    let supersession = runtime
        .classify_async_completion_supersession(
            BridgeAsyncCompletionSupersessionClassificationRequest::request_response(
                &superseded_denial,
                crate::facade::BridgeAsyncRequestTruthViewBasis::authoritative(
                    crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
                    crate::truth_identity_fixtures::truth_commit_fixture("async-commit-c1"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("async-snapshot-c1"),
                ),
            )
            .with_displacing_request_identity(&displacing_request),
        )
        .expect("supersession should classify");
    let retry_lineage = retry_lineage_after_cancellation(
        runtime,
        NodeId::new(704, 0),
        crate::facade::BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("async-commit-d"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("async-snapshot-d"),
        ),
    );
    let writeback_completion =
        admitted_authoritative_request_response_completion(runtime, NodeId::new(705, 0), "phase16");
    let admitted_writeback = runtime
        .admit_async_writeback(authoritative_writeback_request(
            &writeback_completion,
            "phase16",
        ))
        .expect("writeback should admit");
    let staged = runtime
        .stage_async_writeback_effect(&admitted_writeback)
        .expect("staged writeback should admit");
    let committed = runtime.commit_async_writeback(&staged);
    let writeback_receipt = committed
        .committed()
        .expect("writeback should commit")
        .causality_transfer()
        .clone();
    let async_section = BridgeTemporalAsyncCertificationAsyncSectionInput::new(
        vec![
            admitted_completion.request_identity().clone(),
            writeback_completion.request_identity().clone(),
        ],
        vec![admitted_completion.receipt().clone()],
        vec![],
        vec![supersession.receipt().clone()],
        vec![retry_lineage.receipt().clone()],
        vec![writeback_receipt],
    );
    let ordering = runtime.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![BridgeMixedCauseOrderingInput::TruthPatch(committed_patch(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        ))],
    ));
    let mixed_window = runtime
        .plan_mixed_cause_delivery_window(
            &ordering,
            BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        )
        .expect("mixed-cause delivery window should plan");
    let retained_temporal = retained_temporal_resume_basis(
        runtime,
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        crate::facade::BridgeRetainedTemporalWakePosture::Pending,
        true,
    );
    let resume_request_identity = admitted_async_request_identity(
        runtime,
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        710,
    );
    let retained_async =
        retained_inflight_async_resume_basis(runtime, &resume_request_identity, true);
    let (_, _, retained_delivery) = retained_shared_delivery_resume_basis(runtime, &shared_bundle);
    let sealed = sealed_window(
        runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );
    let checkpoint = checkpoint_from_sealed(
        runtime,
        &active,
        &sealed,
        0,
        crate::facade::BridgeSubscriptionDuplicateReplayPolicyKind::RejectDuplicateReplay,
    );
    let retained_resume = retained_subscription_resume_basis(
        runtime,
        &active,
        &checkpoint,
        Some(retained_temporal),
        Some(retained_async),
        Some(retained_delivery),
        true,
    );
    let admitted_resume = runtime
        .admit_subscription_resume_basis(&retained_resume)
        .expect("resume basis should admit");
    let failure_bundle = duplicate_suppression_failure_bundle(runtime);
    let request = BridgeTemporalAsyncCertificationBundleRequest::new(
        active,
        temporal_basis,
        async_section,
        mixed_window,
        shared_bundle,
        admitted_resume,
        failure_bundle,
        diagnostics_richness,
    );
    runtime
        .build_temporal_async_certification_bundle(request)
        .expect("temporal/async certification bundle should build")
}

pub(crate) fn duplicate_suppression_failure_bundle(
    runtime: &RuntimeBridge,
) -> BridgeTemporalAsyncOfflineDiagnosisBundleSealed {
    let duplicate_patch = committed_patch(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        crate::truth_identity_fixtures::truth_commit_fixture("duplicate-commit"),
        crate::truth_identity_fixtures::truth_patch_fixture("duplicate-patch"),
    );
    let ordering = runtime.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![
            BridgeMixedCauseOrderingInput::TruthPatch(duplicate_patch.clone()),
            BridgeMixedCauseOrderingInput::TruthPatch(duplicate_patch),
        ],
    ));
    let suppressed = ordering
        .suppressed()
        .first()
        .expect("duplicate patch should produce one suppressed cause")
        .clone();
    let localized = runtime
        .localize_temporal_async_failure(BridgeFailureLocalizationRequest::MixedCauseSuppressed(
            suppressed,
        ))
        .expect("suppressed mixed cause should localize");
    runtime
        .seal_temporal_async_offline_diagnosis_bundle(vec![localized])
        .expect("localized failure bundle should seal")
}
