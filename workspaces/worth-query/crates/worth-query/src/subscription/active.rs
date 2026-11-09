use super::activation::SubscriptionActivationInput;
use super::active_budget::ActiveSubscriptionWorkBudget;
use super::active_counters::ActiveSubscriptionCounters;
use super::active_digest::ActiveSubscriptionLaneDigest;
use super::active_error::{
    ActiveSubscriptionLifecycleDenialKind, ActiveSubscriptionLifecycleError,
};
use super::active_lane::ActiveSubscriptionLaneAdmission;
use super::active_posture::{
    ActiveLaneLookupClass, ActiveSubscriptionDeliveryPosture, ActiveSubscriptionLifecyclePosture,
};
use super::delivery_density::ActiveDeliveryDensityPosture;
use super::evidence_identities::active_lane_identity;
use super::performance_receipt::SubscriptionPerformanceReceipt;

pub fn admit_active_subscription_lane(
    activation: SubscriptionActivationInput,
    budget: ActiveSubscriptionWorkBudget,
) -> Result<ActiveSubscriptionLaneAdmission, ActiveSubscriptionLifecycleError> {
    let mut counters = ActiveSubscriptionCounters::default();

    if budget.exceeds_phase_one_budget() {
        counters.active_lane_allocation_denial_count = 1;
        return Err(ActiveSubscriptionLifecycleError::new(
            ActiveSubscriptionLifecycleDenialKind::WorkBudgetExceeded,
            "active subscription lane admission exceeds its explicit Phase 1 budget",
            activation.evidence_identity().clone(),
            counters,
        ));
    }

    if budget.durable_checkpoint_requested() {
        counters.durable_checkpoint_overclaim_denial_count = 1;
        return Err(ActiveSubscriptionLifecycleError::new(
            ActiveSubscriptionLifecycleDenialKind::DurableCheckpointOverclaim,
            "durable active subscription checkpoints remain later-milestone debt",
            activation.evidence_identity().clone(),
            counters,
        ));
    }

    if budget.store_backed_restart_requested() {
        counters.store_backed_restart_overclaim_denial_count = 1;
        return Err(ActiveSubscriptionLifecycleError::new(
            ActiveSubscriptionLifecycleDenialKind::StoreBackedRestartOverclaim,
            "store-backed restart-stable active subscription handles remain later-milestone debt",
            activation.evidence_identity().clone(),
            counters,
        ));
    }

    if budget.allocation_posture().is_heap_denied()
        || !budget.allocation_posture().admits_lifecycle_phase()
    {
        counters.heap_allocation_denial_count = 1;
        counters.active_lane_allocation_denial_count = 1;
        return Err(ActiveSubscriptionLifecycleError::new(
            ActiveSubscriptionLifecycleDenialKind::HeapAllocationForbidden,
            "active lane allocation must use an admitted lifecycle allocation posture",
            activation.evidence_identity().clone(),
            counters,
        ));
    }

    if budget.allocation_posture().is_heap_debt() {
        counters.heap_allocation_debt_count = 1;
    }

    if budget.lookup_class() == &ActiveLaneLookupClass::LinearScanDebtExplicit {
        counters.active_lane_linear_scan_debt_count = 1;
    }

    if budget.lookup_class() == &ActiveLaneLookupClass::LinearScanDenied {
        counters.active_lane_linear_scan_denial_count = 1;
        return Err(ActiveSubscriptionLifecycleError::new(
            ActiveSubscriptionLifecycleDenialKind::LinearScanLookupForbidden,
            "active lane admission must use an indexed lookup class",
            activation.evidence_identity().clone(),
            counters,
        ));
    }

    counters.active_lane_admission_count = 1;
    counters.active_lane_lookup_class_count = 1;
    let performance_receipt = SubscriptionPerformanceReceipt::new(
        3,
        budget.registry_lookup_width() + budget.fanout_width() + budget.allocation_scope_width(),
        ActiveDeliveryDensityPosture::SparseDelta,
        budget.allocation_posture(),
        activation.evidence_identity(),
    );
    counters.subscription_performance_receipt_count = 1;
    counters.subscription_budget_consumption_width = performance_receipt.consumed_width();
    counters.subscription_budget_remaining_width = performance_receipt.remaining_width();
    let lifecycle_posture = ActiveSubscriptionLifecyclePosture::SingleConsumer;
    let delivery_posture = ActiveSubscriptionDeliveryPosture::QueryShapedPatch;
    let lane_identity = active_lane_identity(
        activation.evidence_identity(),
        activation.admission_identity(),
        activation.query_declaration_identity(),
        activation.bridge_declaration_identity(),
        activation.future_selection().projection_identity(),
        activation.basis_binding_identity(),
        activation.scoped_declaration_basis().scoped_basis_digest(),
        activation.scoped_activation_basis().scoped_basis_digest(),
        activation.checkpoint_identity(),
        activation.signal_strategy_identity(),
        lifecycle_posture.as_str(),
        delivery_posture.as_str(),
        budget.lookup_class().as_str(),
        budget.allocation_policy().as_str(),
        budget.registry_lookup_width() as usize,
        budget.fanout_width() as usize,
        budget.allocation_scope_width() as usize,
        performance_receipt.performance_receipt_identity(),
        &counters.evidence_identity(),
    )
    .map_err(|cause| {
        ActiveSubscriptionLifecycleError::identity_refused(
            cause,
            activation.evidence_identity().clone(),
            counters.clone(),
        )
    })?;
    let lane_digest = ActiveSubscriptionLaneDigest::new(lane_identity);

    Ok(ActiveSubscriptionLaneAdmission {
        lane_digest,
        activation_identity: activation.evidence_identity().clone(),
        admission_identity: activation.admission_identity().clone(),
        query_declaration_identity: activation.query_declaration_identity().clone(),
        bridge_declaration_identity: activation.bridge_declaration_identity().clone(),
        future_selection: activation.future_selection().clone(),
        basis_binding_identity: activation.basis_binding_identity().clone(),
        scoped_declaration_basis: activation.scoped_declaration_basis().clone(),
        scoped_activation_basis: activation.scoped_activation_basis().clone(),
        checkpoint_identity: activation.checkpoint_identity().clone(),
        signal_strategy_identity: activation.signal_strategy_identity().clone(),
        lifecycle_posture,
        delivery_posture,
        lookup_class: *budget.lookup_class(),
        allocation_policy: *budget.allocation_policy(),
        budget,
        performance_receipt,
        counters,
    })
}
