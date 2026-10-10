use super::super::support::*;
use crate::facade::{
    BridgePreviewSessionDeclarationIdentity, BridgeRuntimePolicy, BridgeSignalBranchIdentity,
    BridgeSpeculativeBranchBindingIdentity, BridgeSubscriptionDeliveryDensityPosture,
    BridgeSubscriptionPreviewPromotionOutcomeClass, BridgeSubscriptionPreviewWorkInput,
    BridgeSubscriptionPreviewWorkKind, BridgeSubscriptionPreviewWorkTraceRejectionKind,
};
use crate::speculation::BridgePreviewSessionIdentity;

#[test]
fn bridge_harness_subscription_suite_34_preview_zero_residue_and_promotion_are_explicit() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
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
                declaration_identity: BridgePreviewSessionDeclarationIdentity::admit_bridge_owned(
                    format!("preview:subscription:{suffix}"),
                ),
                binding_identity: BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned(
                    format!("preview-binding:subscription:{suffix}"),
                ),
                truth_branch_identity: crate::truth_identity_fixtures::truth_branch_fixture(
                    "analysis",
                ),
                signal_branch_identity: BridgeSignalBranchIdentity::admit_bridge_owned(format!(
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
        let residue_index = runtime.build_subscription_preview_residue_scope_index(
            &preview_active,
            work_trace.zero_residue_inputs(),
        );
        let discard = runtime
            .prove_preview_scope_discard_residue(preview_active, residue_index)
            .expect("zero preview residue should discard");

        assert_eq!(discard.total_residue_count(), 0);
        assert_eq!(discard.artifact_records().len(), 7);
        assert_eq!(discard.counters().subscription_preview_discard_count(), 1);
        assert_eq!(
            discard
                .counters()
                .subscription_preview_residue_nonzero_count(),
            0
        );
        assert_eq!(
            discard
                .counters()
                .subscription_preview_non_scope_registry_scan_count(),
            0
        );
        assert_eq!(
            discard.artifact_records()[2].evidence_digest(),
            format!(
                "preview-work-zero-residue|trace={}|scope={}|record={}|category={}",
                work_trace.digest(),
                work_trace.preview_residue_scope_identity().as_str(),
                work_trace.record_digest_for(BridgeSubscriptionPreviewWorkKind::Delivery),
                discard.artifact_records()[2].category().as_str(),
            )
        );
    }

    let malformed_preview = preview_active_subscription_for(
        &runtime,
        BridgePreviewSessionIdentity::admit_bridge_owned(
            "preview-session:subscription:malformed-work",
        ),
        SubscriptionPreviewSessionIdentities {
            declaration_identity: BridgePreviewSessionDeclarationIdentity::admit_bridge_owned(
                "preview:subscription:malformed-work",
            ),
            binding_identity: BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned(
                "preview-binding:subscription:malformed-work",
            ),
            truth_branch_identity: crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            signal_branch_identity: BridgeSignalBranchIdentity::admit_bridge_owned(
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
            declaration_identity: BridgePreviewSessionDeclarationIdentity::admit_bridge_owned(
                "preview:subscription:other-work",
            ),
            binding_identity: BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned(
                "preview-binding:subscription:other-work",
            ),
            truth_branch_identity: crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            signal_branch_identity: BridgeSignalBranchIdentity::admit_bridge_owned(
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
    let admitted_preview = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned(
                "preview-session:subscription:promotion",
            ),
            preview_declaration(&SubscriptionPreviewSessionIdentities {
                declaration_identity: BridgePreviewSessionDeclarationIdentity::admit_bridge_owned(
                    "preview:subscription:promotion",
                ),
                binding_identity: BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned(
                    "preview-binding:subscription:promotion",
                ),
                truth_branch_identity: crate::truth_identity_fixtures::truth_branch_fixture(
                    "analysis",
                ),
                signal_branch_identity: BridgeSignalBranchIdentity::admit_bridge_owned(
                    "signal:subscription:promotion",
                ),
                snapshot_identity: crate::truth_identity_fixtures::truth_snapshot_fixture(
                    "snapshot:subscription:promotion",
                ),
            }),
        )
        .expect("preview session should admit for promotion");
    let (active_preview_session, execution_record) =
        runtime.activate_preview_session(admitted_preview, 3, 1, 2);
    let preview_basis = runtime
        .admit_subscription_preview_basis(&active_preview_session, &execution_record)
        .expect("preview basis should admit");
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
    let proof = active_preview_session.promotion_admissibility_proof();
    let (_promoted_session, speculation_promotion) = runtime
        .promote_preview_session(active_preview_session, &execution_record, &proof)
        .expect("speculation promotion should succeed");
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
    let promotion = runtime
        .record_preview_authoritative_boundary(
            preview_active,
            &promotion_work_trace,
            &speculation_promotion,
            &promoted_ready,
        )
        .expect("subscription promotion boundary should admit");
    let explanation = runtime.inspect_subscription_preview_promotion_record(&promotion);

    assert_eq!(
        promotion.outcome_class(),
        BridgeSubscriptionPreviewPromotionOutcomeClass::PromotedAuthoritativeBoundary
    );
    assert_eq!(
        promotion.preview_active_subscription_identity(),
        &preview_identity
    );
    assert_eq!(
        promotion.promoted_admitted_subscription_identity(),
        &promoted_identity
    );
    assert_eq!(
        promotion.preview_work_trace_identity(),
        promotion_work_trace.preview_work_trace_identity()
    );
    assert_eq!(
        explanation.preview_work_trace_digest(),
        promotion_work_trace.digest()
    );
    assert_ne!(
        promotion.preview_active_subscription_identity().as_str(),
        promotion.promoted_admitted_subscription_identity().as_str()
    );
    assert_eq!(
        explanation.promotion_record_identity(),
        promotion.promotion_record_identity()
    );
    assert_eq!(
        explanation.speculation_promotion_record_digest(),
        speculation_promotion.digest()
    );
    assert_eq!(explanation.counters().diagnostics_bundle_count(), 1);
}
