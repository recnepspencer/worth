use super::*;

pub(super) fn work_budget() -> QuerySubscriptionWorkBudget {
    QuerySubscriptionWorkBudget::scratch_buffer_only(64, 64, 64, 512, 1)
}

pub(super) fn slice_budget() -> QuerySubscriptionSliceBudget {
    QuerySubscriptionSliceBudget::scratch_buffer_only(64, 64, 64, 64, 64, 64, 64, 64)
}

pub(super) fn lowering_budget() -> QuerySubscriptionBridgeLoweringBudget {
    QuerySubscriptionBridgeLoweringBudget::admitted(1, 64, 64, 64, 64)
}

pub(super) fn admission_budget() -> QuerySubscriptionAdmissionBudget {
    QuerySubscriptionAdmissionBudget::admitted(64, 64, 64, 64, 64)
}

pub(super) fn active_lifecycle_budget() -> ActiveSubscriptionWorkBudget {
    ActiveSubscriptionWorkBudget::admitted(
        ActiveRegistryLookupWidth::measured(1),
        ActiveFanoutWidth::measured(1),
        ActiveAllocationScopeWidth::measured(1),
        ActiveSubscriptionAllocationPosture::LifecycleArena,
    )
}

pub(super) fn consumer_attachment_budget() -> SubscriptionConsumerAttachmentBudget {
    SubscriptionConsumerAttachmentBudget::admitted(
        ActiveFanoutWidth::measured(1),
        ConsumerDeliveryPacingWidth::measured(1),
        ActiveAllocationScopeWidth::measured(1),
        DeliveryBackpressurePolicy::RetainWithinWindow,
    )
}

pub(super) fn runtime_subscription_budget_policy() -> Result<
    crate::runtime::WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity,
    IdentityDerivationDenial,
> {
    Ok(
        crate::runtime::WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity::subscription_policy(
            [
                RUNTIME_SUBSCRIPTION_FAMILY_BUDGET_POLICY,
                RUNTIME_SUBSCRIPTION_SLICE_BUDGET_POLICY,
                RUNTIME_SUBSCRIPTION_BRIDGE_BUDGET_POLICY,
                RUNTIME_SUBSCRIPTION_ADMISSION_BUDGET_POLICY,
            ]
            .join("|"),
        )?,
    )
}
