use super::support::*;

#[test]
fn runtime_admits_detail_exact_subscription_against_current_snapshot_basis() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (_runtime, ready) = activation_ready_detail_subscription(resource_request);
    let admitted = ready.admitted();

    assert_eq!(
        admitted.declaration().requested_family_kind(),
        BridgeSubscriptionDeclarationFamilyKind::DetailExact
    );
    assert_eq!(
        admitted.basis_binding().basis_kind(),
        BridgeSubscriptionBasisKind::Snapshot
    );
    assert_eq!(admitted.counters().admitted_subscription_count(), 1);
}

#[test]
fn runtime_admits_collection_membership_subscription_against_snapshot_basis() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (_runtime, ready) = activation_ready_collection_subscription(resource_request);
    let admitted = ready.admitted();

    assert_eq!(
        admitted.declaration().requested_family_kind(),
        BridgeSubscriptionDeclarationFamilyKind::CollectionMembership
    );
    assert_eq!(
        admitted.basis_binding().basis_kind(),
        BridgeSubscriptionBasisKind::Snapshot
    );
    assert_eq!(ready.counters().lifecycle_record_count(), 1);
    assert_eq!(
        ready.lifecycle_record().admitted_subscription_identity(),
        admitted.admitted_subscription_identity()
    );
}

#[test]
fn signal_strategy_identity_is_derived_from_validated_basis_evidence() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let declaration = runtime
        .declare_subscription(
            BridgeSubscriptionDeclarationFamilyKind::DetailExact,
            vec![NormalizedSubscriptionSliceIntent::try_new_entity_field(
                "entity-1",
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native subscription aspect key"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid native subscription field key"),
                SubscriptionSliceKind::SignalField,
            )
            .expect("slice intent should validate")],
            BridgeSubscriptionDeliveryIntentClass::None,
        )
        .expect("declaration should succeed");

    let snapshot_admission = runtime
        .admit_subscription(
            &declaration,
            BridgeSubscriptionBasisRequest::snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            execution,
        )
        .expect("snapshot basis should admit");
    let branch_head_admission = runtime
        .admit_subscription(
            &declaration,
            BridgeSubscriptionBasisRequest::branch_head(
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
            ),
            execution,
        )
        .expect("branch-head basis should admit");

    assert_ne!(
        snapshot_admission.basis_binding().digest(),
        branch_head_admission.basis_binding().digest()
    );
    assert_ne!(
        snapshot_admission.signal_strategy().digest(),
        branch_head_admission.signal_strategy().digest()
    );
    assert!(snapshot_admission
        .signal_strategy()
        .canonical_basis()
        .contains(snapshot_admission.basis_binding().digest()));
    assert!(branch_head_admission
        .signal_strategy()
        .canonical_basis()
        .contains(branch_head_admission.basis_binding().digest()));
    assert!(snapshot_admission
        .canonical_basis()
        .contains(snapshot_admission.signal_strategy().digest()));
    assert!(branch_head_admission
        .canonical_basis()
        .contains(branch_head_admission.signal_strategy().digest()));
}

#[test]
fn runtime_rejects_subscription_admission_when_snapshot_basis_cannot_bind() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let declaration = runtime
        .declare_subscription(
            BridgeSubscriptionDeclarationFamilyKind::DetailExact,
            vec![NormalizedSubscriptionSliceIntent::try_new_entity_field(
                "entity-1",
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native subscription aspect key"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid native subscription field key"),
                SubscriptionSliceKind::SignalField,
            )
            .expect("slice intent should validate")],
            BridgeSubscriptionDeliveryIntentClass::None,
        )
        .expect("declaration should succeed");

    let rejection = runtime
        .admit_subscription(
            &declaration,
            BridgeSubscriptionBasisRequest::snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("missing"),
            ),
            execution,
        )
        .expect_err("unknown snapshot should reject admission");

    assert_eq!(
        rejection.rejection_kind(),
        BridgeSubscriptionAdmissionRejectionKind::BasisResolutionFailure
    );
    assert_eq!(rejection.counters().basis_rejection_count(), 1);
}

#[test]
fn runtime_prepares_and_inspects_activation_ready_subscription() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, ready) = activation_ready_detail_subscription(resource_request);

    let explanation = runtime.inspect_activation_ready_subscription(&ready);

    assert_eq!(
        explanation.admitted_subscription_identity(),
        Some(ready.admitted().admitted_subscription_identity())
    );
    assert_eq!(ready.counters().lifecycle_record_count(), 1);
    assert_eq!(explanation.counters().diagnostics_bundle_count(), 1);
}

#[test]
fn runtime_deactivates_and_replays_retained_subscription_bundle() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, ready) = activation_ready_detail_subscription(resource_request);
    let registry_identity = runtime.subscription_family_registry_identity().clone();

    let deactivated = runtime.deactivate_subscription(ready);
    let replay = runtime
        .replay_subscription(deactivated.retained_bundle())
        .expect("replay should succeed");

    assert_eq!(
        replay.retained_bundle().registry_identity(),
        &registry_identity
    );
    assert_eq!(replay.counters().replay_reconstruction_count(), 1);
}
use crate::subscription::BridgeSubscriptionDeclarationFamilyKind;
