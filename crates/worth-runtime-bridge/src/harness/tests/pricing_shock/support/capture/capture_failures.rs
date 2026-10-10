use super::*;
use crate::facade::{
    BridgeMergeDenialClass, BridgeMergePrecedenceStage, BridgePolicyFieldKind,
    BridgePolicyRejectionKind, BridgeRouteErrorKind,
};

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_restart_replay_bundle(
    policy: BridgeRuntimePolicy,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingRestartReplayBundle {
    let original_runtime = build_pricing_runtime_with_policy(
        pricing_reference_source(),
        RecordingSignalBridgeSink::default(),
        policy,
    );
    original_runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main"),
            execution,
        )
        .expect("pricing restart control route should succeed");
    let canonical_record = original_runtime
        .diagnostics()
        .last_canonical_route_record()
        .expect("pricing restart replay should retain a canonical route record");

    let restarted_runtime = build_pricing_runtime_with_policy(
        pricing_reference_source(),
        RecordingSignalBridgeSink::default(),
        policy,
    );
    let replay = restarted_runtime
        .replay_canonical_record(&canonical_record, execution)
        .expect("pricing restart replay should preserve canonical truth across rebuild");

    PricingRestartReplayBundle {
        source_commit: replay.source_commit().clone(),
        source_snapshot: replay.source_snapshot().clone(),
        route_identity: replay.route_identity().clone(),
        invalidation_identity: replay.invalidation_identity().clone(),
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_restart_failure_bundle(
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingRestartFailureBundle {
    let original_runtime = build_pricing_runtime(
        pricing_reference_source(),
        RecordingSignalBridgeSink::default(),
    );
    original_runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main"),
            execution,
        )
        .expect("pricing restart mismatch control route should succeed");
    let canonical_record = original_runtime
        .diagnostics()
        .last_canonical_route_record()
        .expect("pricing restart mismatch should retain a canonical route record");

    let drifted_source = InMemoryRelationalBridgeSource::default();
    drifted_source.insert_committed_patch(pricing_patch_items(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:steel-main"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main"),
        ),
        vec![
            BridgeCommittedPatchItem::with_target(
                "component:steel",
                crate::facade::BridgeCommittedPatchTarget::entity_field_path(
                    worth_foundational::facade::AspectLocator::new(
                        worth_foundational::facade::LocatorAuthority::Authoritative,
                        worth_foundational::facade::AspectKey::new("cost")
                            .expect("valid bridge patch aspect key"),
                    ),
                    worth_foundational::facade::CanonicalFieldPath::single(
                        worth_foundational::facade::FieldKey::new("usd".to_owned())
                            .expect("valid foundational field key"),
                    ),
                ),
            ),
            BridgeCommittedPatchItem::with_target(
                "component:steel",
                crate::facade::BridgeCommittedPatchTarget::entity_field_path(
                    worth_foundational::facade::AspectLocator::new(
                        worth_foundational::facade::LocatorAuthority::Authoritative,
                        worth_foundational::facade::AspectKey::new("tariff")
                            .expect("valid bridge patch aspect key"),
                    ),
                    worth_foundational::facade::CanonicalFieldPath::single(
                        worth_foundational::facade::FieldKey::new("usd".to_owned())
                            .expect("valid foundational field key"),
                    ),
                ),
            ),
        ],
    ));
    drifted_source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-main"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:rubber-main"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main"),
        ),
        "rubber",
    ));
    drifted_source.insert_snapshot(pricing_snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main"),
        "100",
        "40",
    ));
    let restarted_runtime =
        build_pricing_runtime(drifted_source, RecordingSignalBridgeSink::default());

    let error = restarted_runtime
        .replay_canonical_record(&canonical_record, execution)
        .expect_err("pricing restart replay should reject route drift after truth change");
    let failure_record = restarted_runtime
        .diagnostics()
        .last_failure_record()
        .expect("pricing restart replay mismatch should retain a failure record");

    PricingRestartFailureBundle {
        error_kind: error.kind(),
        replay_mismatch_count: failure_record.counters().route_replay_mismatch_count(),
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_replay_policy_failure_bundle(
) -> (BridgePolicyRejectionKind, BridgePolicyFieldKind) {
    let runtime = build_pricing_runtime_with_policy(
        pricing_reference_source(),
        RecordingSignalBridgeSink::default(),
        BridgeRuntimePolicy::operational().with_replay_artifacts(false),
    );
    let declaration = BridgePolicyDeclaration::new(
        BridgePolicyDeclarationIdentity::admit_bridge_owned(
            "policy:pricing-showcase-replay-conflict",
        ),
        BridgeRequestKind::Preview,
        BridgeExecutionPolicyClass::Optimized,
        BridgeDiagnosticsTier::Minimal,
        true,
        false,
    );
    let rejection = runtime
        .admit_policy_declaration(declaration)
        .expect_err("replay requirement should fail when runtime policy disables replay");

    (rejection.kind(), rejection.field_kind())
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_route_policy_conflict_bundle(
) -> BridgeRouteErrorKind {
    let permissive = build_pricing_runtime_with_policy(
        pricing_reference_source(),
        RecordingSignalBridgeSink::default(),
        BridgeRuntimePolicy::development(),
    );
    let restrictive = build_pricing_runtime_with_policy(
        pricing_reference_source(),
        RecordingSignalBridgeSink::default(),
        BridgeRuntimePolicy::operational().with_replay_artifacts(false),
    );
    let declaration = BridgePolicyDeclaration::new(
        BridgePolicyDeclarationIdentity::admit_bridge_owned("policy:pricing-route-policy-conflict"),
        BridgeRequestKind::Authoritative,
        BridgeExecutionPolicyClass::DeterministicCanonical,
        BridgeDiagnosticsTier::Standard,
        true,
        true,
    );
    let admitted = permissive
        .admit_policy_declaration(declaration)
        .expect("permissive runtime should admit replay-capable route policy");
    let lowered = permissive.lower_admitted_policy(&admitted);
    let error = restrictive
        .project_route_planning_policy(&lowered)
        .expect_err("restrictive runtime should reject divergent route policy");

    error.kind()
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_merge_denial_bundle(
) -> (BridgeMergePrecedenceStage, BridgeMergeDenialClass) {
    let runtime = build_pricing_runtime_with_merge(
        pricing_reference_source(),
        RecordingSignalBridgeSink::default(),
        BridgeRuntimePolicy::development(),
    );
    let contract = runtime
        .admit_merge_history(pricing_topology_denial_merge_declaration())
        .expect("registered topology-denial merge should admit");
    let bundle = runtime
        .replay_merge_history(&contract)
        .expect("registered topology-denial merge should replay as a denied merge bundle");
    (
        bundle
            .lowered_packet_set()
            .blocked_stage()
            .expect("topology-denial merge should retain a blocked stage"),
        bundle
            .lowered_packet_set()
            .denial_class()
            .expect("topology-denial merge should retain a typed denial class"),
    )
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_trust_attack_bundle(
) -> PricingTrustAttackBundle {
    let (replay_policy_error_kind, replay_policy_failure_class) =
        capture_pricing_replay_policy_failure_bundle();
    let route_policy_error_kind = capture_pricing_route_policy_conflict_bundle();
    let (merge_denial_blocked_stage, merge_denial_class) = capture_pricing_merge_denial_bundle();

    PricingTrustAttackBundle {
        replay_policy_error_kind,
        replay_policy_failure_class,
        route_policy_error_kind,
        merge_denial_blocked_stage,
        merge_denial_class,
    }
}
