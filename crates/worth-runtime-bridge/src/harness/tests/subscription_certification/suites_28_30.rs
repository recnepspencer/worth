use super::support::*;
use crate::facade::{
    BridgeRuntimePolicy, BridgeSignalStrategyKind, BridgeSubscriptionAdmissionRejectionKind,
    BridgeSubscriptionBasisKind, BridgeSubscriptionBasisRequest,
    BridgeSubscriptionBasisResolutionFailureKind, BridgeSubscriptionDeliveryIntentClass,
    BridgeSubscriptionReplayMismatchKind, NormalizedSubscriptionSliceIntent,
};
use crate::mapping::SubscriptionSliceKind;
use crate::subscription::BridgeSubscriptionDeclarationFamilyKind;

#[test]
fn bridge_harness_subscription_suite_28_declaration_equivalence_is_canonical_and_policy_invariant()
{
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let development = runtime(BridgeRuntimePolicy::development());
    let forensic = runtime(BridgeRuntimePolicy::forensic());

    let left = development
        .declare_subscription(
            BridgeSubscriptionDeclarationFamilyKind::CollectionMembership,
            vec![
                NormalizedSubscriptionSliceIntent::try_new_entity_partition(
                    "entity-1",
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid native subscription aspect key"),
                    SubscriptionSliceKind::SignalPartition,
                )
                .expect("slice intent should validate"),
                NormalizedSubscriptionSliceIntent::try_new_entity_region(
                    "entity-1",
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid native subscription aspect key"),
                    SubscriptionSliceKind::SignalRegion,
                )
                .expect("slice intent should validate"),
                NormalizedSubscriptionSliceIntent::try_new_entity_region(
                    "entity-1",
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid native subscription aspect key"),
                    SubscriptionSliceKind::SignalRegion,
                )
                .expect("slice intent should validate"),
            ],
            BridgeSubscriptionDeliveryIntentClass::CanonicalMeaningfulChange,
        )
        .expect("left declaration should succeed");
    let right = forensic
        .declare_subscription(
            BridgeSubscriptionDeclarationFamilyKind::CollectionMembership,
            vec![
                NormalizedSubscriptionSliceIntent::try_new_entity_region(
                    "entity-1",
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid native subscription aspect key"),
                    SubscriptionSliceKind::SignalRegion,
                )
                .expect("slice intent should validate"),
                NormalizedSubscriptionSliceIntent::try_new_entity_partition(
                    "entity-1",
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid native subscription aspect key"),
                    SubscriptionSliceKind::SignalPartition,
                )
                .expect("slice intent should validate"),
            ],
            BridgeSubscriptionDeliveryIntentClass::None,
        )
        .expect("right declaration should succeed");

    assert_eq!(left.digest(), right.digest());
    assert_eq!(left.declaration_identity(), right.declaration_identity());
    assert_eq!(
        development.subscription_family_registry_identity(),
        forensic.subscription_family_registry_identity()
    );
    assert_eq!(left.normalized_slice_intent_count(), 2);
    assert_eq!(left.counters().family_lookup_count(), 1);
    assert_eq!(left.counters().declaration_count(), 1);
    assert_eq!(left.counters().declaration_rejection_count(), 0);
    assert_eq!(left.counters().basis_request_count(), 0);
    assert_eq!(left.counters().signal_strategy_selection_count(), 0);

    let left_admitted = development
        .admit_subscription(
            &left,
            BridgeSubscriptionBasisRequest::snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            execution,
        )
        .expect("left admission should succeed");
    let right_admitted = forensic
        .admit_subscription(
            &right,
            BridgeSubscriptionBasisRequest::snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            execution,
        )
        .expect("right admission should succeed");

    assert_eq!(left_admitted.digest(), right_admitted.digest());
    assert_eq!(
        left_admitted.basis_binding().digest(),
        right_admitted.basis_binding().digest()
    );
    assert_eq!(
        left_admitted.signal_strategy().digest(),
        right_admitted.signal_strategy().digest()
    );

    let different_family = detail_subscription(&development);
    assert_ne!(left.digest(), different_family.digest());
}

#[test]
fn bridge_harness_subscription_suite_29_basis_binding_is_explicit_and_fail_closed() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let baseline = runtime(BridgeRuntimePolicy::development());
    let declaration = detail_subscription(&baseline);

    let snapshot_admitted = baseline
        .admit_subscription(
            &declaration,
            BridgeSubscriptionBasisRequest::snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            execution,
        )
        .expect("snapshot admission should succeed");
    let branch_admitted = baseline
        .admit_subscription(
            &declaration,
            BridgeSubscriptionBasisRequest::branch_head(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            ),
            execution,
        )
        .expect("branch-head admission should succeed");

    assert_eq!(
        snapshot_admitted.basis_binding().basis_kind(),
        BridgeSubscriptionBasisKind::Snapshot
    );
    assert_eq!(
        branch_admitted.basis_binding().basis_kind(),
        BridgeSubscriptionBasisKind::BranchHead
    );
    assert_ne!(
        snapshot_admitted.basis_binding().digest(),
        branch_admitted.basis_binding().digest()
    );
    assert_ne!(snapshot_admitted.digest(), branch_admitted.digest());
    assert_eq!(
        snapshot_admitted.signal_strategy().strategy_kind(),
        BridgeSignalStrategyKind::ExactFieldLensObservation
    );
    assert_eq!(snapshot_admitted.counters().basis_request_count(), 1);
    assert_eq!(snapshot_admitted.counters().basis_binding_count(), 1);
    assert_eq!(snapshot_admitted.counters().basis_rejection_count(), 0);
    assert_eq!(
        snapshot_admitted
            .counters()
            .signal_strategy_selection_count(),
        1
    );

    let missing_snapshot = baseline
        .admit_subscription(
            &declaration,
            BridgeSubscriptionBasisRequest::snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-missing"),
            ),
            execution,
        )
        .expect_err("missing snapshot should reject");
    assert_eq!(
        missing_snapshot.rejection_kind(),
        BridgeSubscriptionAdmissionRejectionKind::BasisResolutionFailure
    );
    assert_eq!(
        missing_snapshot.basis_resolution_failure_kind(),
        Some(BridgeSubscriptionBasisResolutionFailureKind::SnapshotAcquisitionFailure)
    );

    let misbound_runtime =
        runtime_with_sources(BridgeRuntimePolicy::development(), MisbindingSource);
    let misbound_declaration = detail_subscription(&misbound_runtime);
    let snapshot_mismatch = misbound_runtime
        .admit_subscription(
            &misbound_declaration,
            BridgeSubscriptionBasisRequest::snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            execution,
        )
        .expect_err("snapshot identity mismatch should reject");
    assert_eq!(
        snapshot_mismatch.basis_resolution_failure_kind(),
        Some(BridgeSubscriptionBasisResolutionFailureKind::SnapshotIdentityMismatch)
    );

    let wrong_branch_runtime =
        runtime_with_sources(BridgeRuntimePolicy::development(), WrongBranchHeadSource);
    let wrong_branch_declaration = detail_subscription(&wrong_branch_runtime);
    let branch_mismatch = wrong_branch_runtime
        .admit_subscription(
            &wrong_branch_declaration,
            BridgeSubscriptionBasisRequest::branch_head(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            ),
            execution,
        )
        .expect_err("branch mismatch should reject");
    assert_eq!(
        branch_mismatch.basis_resolution_failure_kind(),
        Some(BridgeSubscriptionBasisResolutionFailureKind::BranchHeadMismatch)
    );
    assert_eq!(branch_mismatch.counters().basis_request_count(), 1);
    assert_eq!(branch_mismatch.counters().basis_binding_count(), 0);
    assert_eq!(branch_mismatch.counters().basis_rejection_count(), 1);
    assert_eq!(
        branch_mismatch.counters().signal_strategy_selection_count(),
        0
    );
}

#[test]
fn bridge_harness_subscription_suite_30_lifecycle_replay_parity_is_canonical() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let development = runtime(BridgeRuntimePolicy::development());
    let forensic = runtime(BridgeRuntimePolicy::forensic());

    let development_declaration = detail_subscription(&development);
    let forensic_declaration = detail_subscription(&forensic);

    let development_admitted = development
        .admit_subscription(
            &development_declaration,
            BridgeSubscriptionBasisRequest::snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            execution,
        )
        .expect("development admission should succeed");
    let forensic_admitted = forensic
        .admit_subscription(
            &forensic_declaration,
            BridgeSubscriptionBasisRequest::snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            execution,
        )
        .expect("forensic admission should succeed");

    let development_ready = development.prepare_subscription_activation(&development_admitted);
    let forensic_ready = forensic.prepare_subscription_activation(&forensic_admitted);

    assert_eq!(
        development_ready.lifecycle_record().digest(),
        forensic_ready.lifecycle_record().digest()
    );
    assert_eq!(
        development_ready.retained_bundle().digest(),
        forensic_ready.retained_bundle().digest()
    );

    let development_replay = development
        .replay_subscription(development_ready.retained_bundle())
        .expect("development replay should succeed");
    let forensic_replay = forensic
        .replay_subscription(forensic_ready.retained_bundle())
        .expect("forensic replay should succeed");

    assert_eq!(development_replay.digest(), forensic_replay.digest());
    assert_eq!(
        development_replay.counters().replay_reconstruction_count(),
        1
    );
    assert_eq!(development_replay.counters().basis_request_count(), 0);
    assert_eq!(
        development_replay
            .counters()
            .signal_strategy_selection_count(),
        0
    );

    let deactivated = development.deactivate_subscription(development_ready);
    assert_ne!(
        deactivated.lifecycle_record().digest(),
        forensic_ready.lifecycle_record().digest()
    );
    assert_eq!(
        deactivated.lifecycle_record().state_kind(),
        crate::facade::BridgeSubscriptionLifecycleStateKind::Deactivated
    );

    let mismatch = crate::facade::BridgeSubscriptionReplaySummary::replay(
        &crate::facade::BridgeSubscriptionFamilyRegistryIdentity::admit_bridge_owned(
            "registry-drift",
        ),
        forensic_ready.retained_bundle(),
    )
    .expect_err("registry drift should reject");
    assert_eq!(
        mismatch.mismatch_kind(),
        BridgeSubscriptionReplayMismatchKind::RegistryIdentityMismatch
    );
    assert_eq!(mismatch.counters().replay_mismatch_count(), 1);
}
