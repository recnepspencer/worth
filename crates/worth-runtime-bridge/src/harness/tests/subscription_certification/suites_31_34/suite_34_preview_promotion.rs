use super::super::support::*;
use crate::facade::{
    BridgePreviewBranchBinding, BridgeRuntimePolicy, BridgeSignalBranchIdentity,
    BridgeSubscriptionAuthoritativePreviewReadmissionClass,
    BridgeSubscriptionDeliveryDensityPosture, BridgeSubscriptionPreviewLifecycleResidueInput,
    BridgeSubscriptionPreviewLifecycleResidueKind, BridgeSubscriptionPreviewWorkInput,
    BridgeSubscriptionPreviewWorkKind, BridgeSubscriptionPreviewWorkTraceRejectionKind,
};
use crate::preview::BridgePreviewSessionIdentity;

#[test]
fn bridge_harness_subscription_suite_34_preview_zero_residue_and_promotion_are_explicit() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let capture_zero_residue =
        |preview: &crate::facade::BridgePreviewActiveSubscription,
         trace: &crate::facade::BridgeSubscriptionPreviewWorkTrace| {
            runtime
                .capture_preview_lifecycle_residue_envelope(
                    preview,
                    trace,
                    BridgeSubscriptionPreviewLifecycleResidueKind::all()
                        .into_iter()
                        .map(|kind| {
                            BridgeSubscriptionPreviewLifecycleResidueInput::from_preview_work_trace(
                                kind, 0, trace,
                            )
                        })
                        .collect(),
                )
                .expect("preview work should bind the zero residue envelope")
        };
    let detail = detail_subscription(&runtime);
    let collection = collection_subscription(&runtime);

    for (suffix, declaration) in [
        ("detail-discard", &detail),
        ("collection-discard", &collection),
    ] {
        let preview_active = preview_active_subscription_for(
            &runtime,
            BridgePreviewSessionIdentity::admit_bridge_owned(format!(
                "preview-session:subscription:{suffix}"
            )),
            SubscriptionPreviewSessionIdentities {
                truth_branch_identity: crate::truth_identity_fixtures::truth_branch_fixture(
                    "analysis",
                ),
                signal_branch_identity: BridgeSignalBranchIdentity::from_stable_name(format!(
                    "signal:subscription:{suffix}"
                )),
                snapshot_identity: crate::truth_identity_fixtures::truth_snapshot_fixture(format!(
                    "snapshot:subscription:{suffix}"
                )),
            },
            declaration,
            resource_request,
        );
        let preview_identity = preview_active
            .preview_active_subscription_identity()
            .clone();
        let preview_scope_identity = preview_active.preview_scope_identity().clone();
        let work_trace = runtime
            .record_preview_subscription_work(
                &preview_active,
                vec![
                    BridgeSubscriptionPreviewWorkInput::routing(&preview_active),
                    BridgeSubscriptionPreviewWorkInput::delivery(&preview_active),
                    BridgeSubscriptionPreviewWorkInput::diagnostics(&preview_active),
                    BridgeSubscriptionPreviewWorkInput::continuation(&preview_active),
                ],
            )
            .expect("preview residue must be backed by all preview work descriptors");
        assert_eq!(work_trace.records().len(), 4);
        assert_eq!(
            work_trace.preview_active_subscription_identity(),
            &preview_identity
        );
        assert_eq!(work_trace.preview_scope_identity(), &preview_scope_identity);
        assert_eq!(
            work_trace.records()[0].kind(),
            BridgeSubscriptionPreviewWorkKind::Routing
        );
        assert_eq!(
            work_trace.records()[1].kind(),
            BridgeSubscriptionPreviewWorkKind::Delivery
        );
        assert_eq!(
            work_trace.records()[2].kind(),
            BridgeSubscriptionPreviewWorkKind::Diagnostics
        );
        assert_eq!(
            work_trace.records()[3].kind(),
            BridgeSubscriptionPreviewWorkKind::Continuation
        );
        assert_eq!(
            work_trace.records()[0]
                .evidence()
                .preview_active_subscription_identity(),
            &preview_identity
        );
        assert_eq!(
            work_trace.records()[0].evidence().preview_scope_identity(),
            &preview_scope_identity
        );
        let residue = capture_zero_residue(&preview_active, &work_trace);
        let discard = runtime
            .admit_preview_lifecycle_discard(preview_active, residue)
            .expect("zero preview residue should discard");

        assert_eq!(discard.total_residue_count(), 0);
        assert_eq!(discard.residue_records().len(), 4);
        assert_eq!(
            discard.preview_active_subscription_identity(),
            &preview_identity
        );
        assert_eq!(discard.counters().subscription_preview_discard_count(), 1);
    }

    let malformed_preview = preview_active_subscription_for(
        &runtime,
        BridgePreviewSessionIdentity::admit_bridge_owned(
            "preview-session:subscription:malformed-work",
        ),
        SubscriptionPreviewSessionIdentities {
            truth_branch_identity: crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            signal_branch_identity: BridgeSignalBranchIdentity::from_stable_name(
                "signal:subscription:malformed-work",
            ),
            snapshot_identity: crate::truth_identity_fixtures::truth_snapshot_fixture(
                "snapshot:subscription:malformed-work",
            ),
        },
        &detail,
        resource_request,
    );
    let duplicate_rejection = runtime
        .record_preview_subscription_work(
            &malformed_preview,
            vec![
                BridgeSubscriptionPreviewWorkInput::routing(&malformed_preview),
                BridgeSubscriptionPreviewWorkInput::routing(&malformed_preview),
                BridgeSubscriptionPreviewWorkInput::delivery(&malformed_preview),
                BridgeSubscriptionPreviewWorkInput::diagnostics(&malformed_preview),
                BridgeSubscriptionPreviewWorkInput::continuation(&malformed_preview),
            ],
        )
        .expect_err("duplicate preview work kind must reject before residue indexing");
    assert_eq!(
        duplicate_rejection.rejection_kind(),
        BridgeSubscriptionPreviewWorkTraceRejectionKind::DuplicateWorkKind
    );
    let missing_rejection = runtime
        .record_preview_subscription_work(
            &malformed_preview,
            vec![
                BridgeSubscriptionPreviewWorkInput::routing(&malformed_preview),
                BridgeSubscriptionPreviewWorkInput::delivery(&malformed_preview),
                BridgeSubscriptionPreviewWorkInput::diagnostics(&malformed_preview),
            ],
        )
        .expect_err("missing continuation preview work must reject before residue indexing");
    assert_eq!(
        missing_rejection.rejection_kind(),
        BridgeSubscriptionPreviewWorkTraceRejectionKind::MissingWorkKind
    );
    let other_preview = preview_active_subscription_for(
        &runtime,
        BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:subscription:other-work"),
        SubscriptionPreviewSessionIdentities {
            truth_branch_identity: crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            signal_branch_identity: BridgeSignalBranchIdentity::from_stable_name(
                "signal:subscription:other-work",
            ),
            snapshot_identity: crate::truth_identity_fixtures::truth_snapshot_fixture(
                "snapshot:subscription:other-work",
            ),
        },
        &detail,
        resource_request,
    );
    let mismatch_rejection = runtime
        .record_preview_subscription_work(
            &malformed_preview,
            vec![
                BridgeSubscriptionPreviewWorkInput::routing(&other_preview),
                BridgeSubscriptionPreviewWorkInput::delivery(&malformed_preview),
                BridgeSubscriptionPreviewWorkInput::diagnostics(&malformed_preview),
                BridgeSubscriptionPreviewWorkInput::continuation(&malformed_preview),
            ],
        )
        .expect_err("work evidence from a different preview scope must reject");
    assert_eq!(
        mismatch_rejection.rejection_kind(),
        BridgeSubscriptionPreviewWorkTraceRejectionKind::PreviewWorkEvidenceMismatch
    );

    let promotion_ready = activation_ready_for(&runtime, &detail, resource_request);
    let session = runtime.begin_preview(
        BridgePreviewSessionIdentity::from_stable_name("preview-session:subscription:promotion"),
        BridgePreviewBranchBinding::new(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            BridgeSignalBranchIdentity::from_stable_name("signal:subscription:promotion"),
        ),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:subscription:promotion"),
    );
    let preview_basis = runtime.admit_subscription_preview_basis(&session);
    let cost_profile = runtime
        .admit_subscription_delivery_cost_profile(
            BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
            4,
            4,
            1,
        )
        .expect("cost profile should admit");
    let preview_active = runtime.activate_preview_subscription_delivery(
        promotion_ready,
        preview_basis,
        cost_profile,
        canonical_consumer(&runtime),
    );
    let promoted_ready = activation_ready_for(&runtime, &detail, resource_request);
    let preview_identity = preview_active
        .preview_active_subscription_identity()
        .clone();
    let promotion_work_trace = runtime
        .record_preview_subscription_work(
            &preview_active,
            vec![
                BridgeSubscriptionPreviewWorkInput::routing(&preview_active),
                BridgeSubscriptionPreviewWorkInput::delivery(&preview_active),
                BridgeSubscriptionPreviewWorkInput::diagnostics(&preview_active),
                BridgeSubscriptionPreviewWorkInput::continuation(&preview_active),
            ],
        )
        .expect("promotion must be backed by scope-local preview work");
    let promoted_identity = promoted_ready
        .admitted()
        .admitted_subscription_identity()
        .clone();
    let residue = capture_zero_residue(&preview_active, &promotion_work_trace);
    let unrelated = runtime.begin_preview(
        session.session_identity().clone(),
        session.branches().clone(),
        session.snapshot_identity().clone(),
    );
    let refused = runtime
        .admit_preview_lifecycle_promotion(
            &preview_active,
            &promotion_work_trace,
            &residue,
            &unrelated.finish(),
        )
        .expect_err("a different occurrence of the same session label has no custody");
    assert_eq!(
        refused.rejection_kind(),
        crate::facade::BridgeSubscriptionPreviewLifecyclePromotionRejectionKind::PromotionSessionMismatch,
    );
    let promotion = runtime
        .admit_preview_lifecycle_promotion(
            &preview_active,
            &promotion_work_trace,
            &residue,
            &session.finish(),
        )
        .expect("subscription lifecycle promotion should admit");

    assert_eq!(
        promotion.preview_work_trace_identity(),
        promotion_work_trace.preview_work_trace_identity()
    );

    let readmission = runtime
        .prepare_authoritative_preview_readmission(promotion, &promoted_ready)
        .expect("authoritative readmission should admit");
    assert_eq!(
        readmission.readmission_class(),
        BridgeSubscriptionAuthoritativePreviewReadmissionClass::ReAdmittedAuthoritativeBoundary
    );
    assert_eq!(
        readmission.preview_active_subscription_identity(),
        preview_identity.as_str()
    );
    assert_eq!(
        readmission.authoritative_admitted_subscription_identity(),
        &promoted_identity
    );
}
