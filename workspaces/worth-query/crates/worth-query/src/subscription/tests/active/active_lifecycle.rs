use super::runtime_harness::{activation_for, active_budget};
use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn active_lane_admission_preserves_activation_digests_and_posture() {
    let activation = activation_for(LiveQueryFamily::Detail, None);
    let activation_digest = activation.activation_projection().label().to_string();
    let admission_digest = activation.admission_projection().label().to_string();
    let query_declaration_digest = activation
        .query_declaration_projection()
        .label()
        .to_string();
    let bridge_declaration_digest = activation
        .bridge_declaration_projection()
        .label()
        .to_string();
    let basis_binding_digest = activation.basis_binding_projection().label().to_string();
    let scoped_declaration_basis_digest = activation
        .scoped_declaration_basis()
        .scoped_basis_digest()
        .to_string();
    let scoped_activation_basis_digest = activation
        .scoped_activation_basis()
        .scoped_basis_digest()
        .to_string();
    let signal_strategy_digest = activation.signal_strategy_projection().label().to_string();
    let signal_strategy_identity = activation.signal_strategy_identity().clone();

    let admission = admit_active_subscription_lane(
        activation,
        active_budget(1, 1, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();

    assert_eq!(
        admission.activation_projection().label().as_str(),
        activation_digest.as_str()
    );
    assert_eq!(
        admission.admission_projection().label().as_str(),
        admission_digest.as_str()
    );
    assert_eq!(
        admission.query_declaration_projection().label().as_str(),
        query_declaration_digest.as_str()
    );
    assert_eq!(
        admission.bridge_declaration_projection().label().as_str(),
        bridge_declaration_digest.as_str()
    );
    assert_eq!(
        admission.basis_binding_projection().label().as_str(),
        basis_binding_digest.as_str()
    );
    assert_eq!(
        admission.scoped_declaration_basis().scoped_basis_digest(),
        scoped_declaration_basis_digest
    );
    assert_eq!(
        admission.scoped_activation_basis().scoped_basis_digest(),
        scoped_activation_basis_digest
    );
    assert_eq!(
        admission.signal_strategy_identity(),
        &signal_strategy_identity
    );
    assert_eq!(
        admission.signal_strategy_projection().label().as_str(),
        signal_strategy_digest.as_str()
    );
    assert_eq!(
        admission.lifecycle_posture(),
        &ActiveSubscriptionLifecyclePosture::SingleConsumer
    );
    assert_eq!(
        admission.delivery_posture(),
        &ActiveSubscriptionDeliveryPosture::QueryShapedPatch
    );
    assert_eq!(
        admission.lookup_class(),
        &ActiveLaneLookupClass::EquivalenceIndex
    );
    assert_eq!(
        admission.allocation_policy(),
        &ActiveSubscriptionAllocationPolicy::LifecycleArena
    );
    assert_eq!(admission.performance_receipt().consumed_width(), 3);
    assert_eq!(admission.performance_receipt().budgeted_width(), 3);
    assert_eq!(admission.performance_receipt().remaining_width(), 0);
    assert_eq!(admission.counters().active_lane_admission_count(), 1);
    assert_eq!(admission.counters().active_lane_lookup_class_count(), 1);
    assert_eq!(admission.counters().active_lane_creation_count(), 0);
    assert_eq!(admission.counters().active_lane_handle_issue_count(), 0);
}

#[test]
fn linear_scan_debt_is_explicit_when_lookup_posture_allows_it() {
    let activation = activation_for(LiveQueryFamily::Detail, None);
    let admission = admit_active_subscription_lane(
        activation,
        active_budget(1, 1, ActiveSubscriptionAllocationPolicy::LifecycleArena)
            .with_lookup_class(ActiveLaneLookupClass::LinearScanDebtExplicit),
    )
    .unwrap();

    assert_eq!(
        admission.lookup_class(),
        &ActiveLaneLookupClass::LinearScanDebtExplicit
    );
    assert_eq!(admission.counters().active_lane_linear_scan_debt_count(), 1);
}

#[test]
fn active_runtime_opens_registry_owned_lane_handle() {
    let activation = activation_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );
    let admission = admit_active_subscription_lane(
        activation,
        active_budget(1, 1, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let lane_digest = admission.lane_digest().clone();
    let mut runtime = ActiveSubscriptionRuntime::new();

    let handle = open_active_subscription_lane(&mut runtime, admission).unwrap();

    assert_eq!(runtime.lane_count(), 1);
    assert_eq!(handle.lane_digest(), &lane_digest);
    assert_eq!(handle.lane_index(), 0);
    assert_eq!(handle.registry_generation(), 1);
    assert_eq!(runtime.counters().active_lane_registry_lookup_count(), 1);
    assert_eq!(runtime.counters().active_lane_creation_count(), 1);
    assert_eq!(runtime.counters().active_lane_handle_issue_count(), 1);
    assert_eq!(runtime.counters().active_lane_join_denial_count(), 0);
}

#[test]
fn linear_scan_lookup_class_denies_before_lane_exists() {
    let activation = activation_for(LiveQueryFamily::Detail, None);
    let error = admit_active_subscription_lane(
        activation,
        active_budget(1, 1, ActiveSubscriptionAllocationPolicy::LifecycleArena)
            .with_lookup_class(ActiveLaneLookupClass::LinearScanDenied),
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &ActiveSubscriptionLifecycleDenialKind::LinearScanLookupForbidden
    );
    assert_eq!(error.counters().active_lane_admission_count(), 0);
    assert_eq!(error.counters().active_lane_linear_scan_denial_count(), 1);
    assert_eq!(error.counters().active_lane_allocation_denial_count(), 0);
}

#[test]
fn duplicate_active_lane_join_reuses_existing_lane() {
    let activation = activation_for(LiveQueryFamily::Detail, None);
    let first = admit_active_subscription_lane(
        activation.clone(),
        active_budget(1, 1, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let duplicate = admit_active_subscription_lane(
        activation,
        active_budget(1, 1, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let mut runtime = ActiveSubscriptionRuntime::new();

    let first_handle = open_active_subscription_lane(&mut runtime, first).unwrap();
    let second_handle = open_active_subscription_lane(&mut runtime, duplicate).unwrap();

    assert_eq!(first_handle.lane_digest(), second_handle.lane_digest());
    assert_eq!(first_handle.lane_index(), second_handle.lane_index());
    assert_ne!(
        first_handle.registry_generation(),
        second_handle.registry_generation()
    );
    assert_eq!(runtime.counters().active_lane_registry_lookup_count(), 1);
    assert_eq!(runtime.counters().active_lane_creation_count(), 0);
    assert_eq!(runtime.counters().active_lane_join_count(), 1);
    assert_eq!(runtime.counters().shared_lane_count(), 1);
    assert_eq!(runtime.counters().active_lane_handle_issue_count(), 1);
    assert_eq!(
        runtime.lane_lifecycle_posture(&first_handle),
        Some(&ActiveSubscriptionLifecyclePosture::SharedEquivalent)
    );
    assert_eq!(runtime.lane_count(), 1);
}

#[test]
fn meaning_mismatch_denies_explicit_shared_join() {
    let activation = activation_for(LiveQueryFamily::Detail, None);
    let mismatched_activation = activation_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );
    let first = admit_active_subscription_lane(
        activation,
        active_budget(1, 1, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let mismatch = admit_active_subscription_lane(
        mismatched_activation,
        active_budget(1, 1, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let mut runtime = ActiveSubscriptionRuntime::new();

    let handle = open_active_subscription_lane(&mut runtime, first).unwrap();
    let error = join_active_subscription_lane(&mut runtime, &handle, mismatch).unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &ActiveSubscriptionLifecycleDenialKind::RegistryEquivalenceMismatch
    );
    assert_eq!(error.counters().active_lane_registry_lookup_count(), 1);
    assert_eq!(error.counters().active_lane_join_denial_count(), 1);
    assert_eq!(runtime.lane_count(), 1);
}

#[test]
fn active_lane_budget_denies_before_admission() {
    let activation = activation_for(LiveQueryFamily::Detail, None);
    let error = admit_active_subscription_lane(
        activation,
        ActiveSubscriptionWorkBudget::admitted(
            ActiveRegistryLookupWidth::measured(0),
            ActiveFanoutWidth::measured(1),
            ActiveAllocationScopeWidth::measured(1),
            ActiveSubscriptionAllocationPolicy::LifecycleArena,
        ),
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &ActiveSubscriptionLifecycleDenialKind::WorkBudgetExceeded
    );
    assert_eq!(error.counters().active_lane_admission_count(), 0);
    assert_eq!(error.counters().active_lane_allocation_denial_count(), 1);
}

#[test]
fn durable_checkpoint_overclaim_denies_before_lane_exists() {
    let activation = activation_for(LiveQueryFamily::Detail, None);
    let error = admit_active_subscription_lane(
        activation,
        active_budget(1, 1, ActiveSubscriptionAllocationPolicy::LifecycleArena)
            .with_durable_checkpoint_request(),
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &ActiveSubscriptionLifecycleDenialKind::DurableCheckpointOverclaim
    );
    assert_eq!(error.counters().active_lane_admission_count(), 0);
    assert_eq!(
        error.counters().durable_checkpoint_overclaim_denial_count(),
        1
    );
}

#[test]
fn store_backed_restart_overclaim_denies_before_lane_exists() {
    let activation = activation_for(LiveQueryFamily::Detail, None);
    let error = admit_active_subscription_lane(
        activation,
        active_budget(1, 1, ActiveSubscriptionAllocationPolicy::LifecycleArena)
            .with_store_backed_restart_request(),
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &ActiveSubscriptionLifecycleDenialKind::StoreBackedRestartOverclaim
    );
    assert_eq!(error.counters().active_lane_admission_count(), 0);
    assert_eq!(
        error
            .counters()
            .store_backed_restart_overclaim_denial_count(),
        1
    );
}

#[test]
fn heap_allocation_policy_denies_before_lane_exists() {
    let activation = activation_for(LiveQueryFamily::Detail, None);
    let error = admit_active_subscription_lane(
        activation,
        ActiveSubscriptionWorkBudget::admitted(
            ActiveRegistryLookupWidth::measured(1),
            ActiveFanoutWidth::measured(1),
            ActiveAllocationScopeWidth::measured(1),
            ActiveSubscriptionAllocationPolicy::HeapAllocationDenied,
        ),
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &ActiveSubscriptionLifecycleDenialKind::HeapAllocationForbidden
    );
    assert_eq!(error.counters().active_lane_admission_count(), 0);
    assert_eq!(error.counters().active_lane_allocation_denial_count(), 1);
    assert_eq!(error.counters().heap_allocation_denial_count(), 1);
}

#[test]
fn lifecycle_admission_rejects_delivery_window_allocation_posture() {
    let activation = activation_for(LiveQueryFamily::Detail, None);
    let error = admit_active_subscription_lane(
        activation,
        ActiveSubscriptionWorkBudget::admitted(
            ActiveRegistryLookupWidth::measured(1),
            ActiveFanoutWidth::measured(1),
            ActiveAllocationScopeWidth::measured(1),
            ActiveSubscriptionAllocationPosture::DeliveryWindowArena,
        ),
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &ActiveSubscriptionLifecycleDenialKind::HeapAllocationForbidden
    );
    assert_eq!(error.counters().active_lane_admission_count(), 0);
    assert_eq!(error.counters().active_lane_allocation_denial_count(), 1);
    assert_eq!(error.counters().heap_allocation_denial_count(), 1);
}

#[test]
fn lifecycle_heap_allocation_debt_is_explicit_and_digest_bound() {
    let activation = activation_for(LiveQueryFamily::Detail, None);
    let normal = admit_active_subscription_lane(
        activation.clone(),
        active_budget(1, 1, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let debt = admit_active_subscription_lane(
        activation,
        ActiveSubscriptionWorkBudget::admitted(
            ActiveRegistryLookupWidth::measured(1),
            ActiveFanoutWidth::measured(1),
            ActiveAllocationScopeWidth::measured(1),
            ActiveSubscriptionAllocationPosture::HeapAllocationDebtExplicit,
        ),
    )
    .unwrap();

    assert_eq!(debt.counters().heap_allocation_debt_count(), 1);
    assert_ne!(normal.lane_digest(), debt.lane_digest());
    assert_ne!(
        normal
            .performance_receipt()
            .performance_receipt_projection()
            .label(),
        debt.performance_receipt()
            .performance_receipt_projection()
            .label()
    );
    assert_eq!(
        debt.allocation_posture(),
        ActiveSubscriptionAllocationPosture::HeapAllocationDebtExplicit
    );
}
