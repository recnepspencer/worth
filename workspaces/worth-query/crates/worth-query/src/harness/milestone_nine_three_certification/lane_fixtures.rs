use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::subscription::{
    ActiveAllocationScopeWidth, ActiveFanoutWidth, ActiveRegistryLookupWidth,
    ActiveSubscriptionAllocationPosture, ActiveSubscriptionWorkBudget, ConsumerDeliveryPacingWidth,
    DeliveryBackpressurePolicy, DeliveryWindowWidth, MaintenanceDeltaWidth, PatchGroupWidth,
    QueryDeliveryWindowBudget, QuerySubscriptionBridgeLoweringBudget, QuerySubscriptionSliceBudget,
    QuerySubscriptionWorkBudget, SubscriptionConsumerAttachmentBudget,
};

pub(super) fn continuation_harness_identity(
    label: &str,
) -> Result<crate::WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    crate::WorthQueryEvidenceIdentity::compose(
        crate::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("identity_family"),
        "subscription_continuation_harness_identity_v1",
    )
    .field_shape(crate::WorthQueryEvidenceTag::new("label"), label)
    .seal()
}

pub(super) fn roomy_budget() -> QuerySubscriptionWorkBudget {
    QuerySubscriptionWorkBudget::scratch_buffer_only(8, 8, 8, 32, 1)
}

pub(super) fn roomy_slice_budget() -> QuerySubscriptionSliceBudget {
    QuerySubscriptionSliceBudget::scratch_buffer_only(8, 8, 8, 8, 8, 8, 8, 8)
}

pub(super) fn roomy_lowering_budget() -> QuerySubscriptionBridgeLoweringBudget {
    QuerySubscriptionBridgeLoweringBudget::admitted(1, 8, 8, 1, 1)
}

pub(super) fn active_budget() -> ActiveSubscriptionWorkBudget {
    ActiveSubscriptionWorkBudget::admitted(
        ActiveRegistryLookupWidth::measured(1),
        ActiveFanoutWidth::measured(1),
        ActiveAllocationScopeWidth::measured(1),
        ActiveSubscriptionAllocationPosture::LifecycleArena,
    )
}

pub(super) fn attachment_budget() -> SubscriptionConsumerAttachmentBudget {
    SubscriptionConsumerAttachmentBudget::admitted(
        ActiveFanoutWidth::measured(1),
        ConsumerDeliveryPacingWidth::measured(1),
        ActiveAllocationScopeWidth::measured(1),
        DeliveryBackpressurePolicy::RetainWithinWindow,
    )
}

pub(super) fn delivery_budget() -> QueryDeliveryWindowBudget {
    QueryDeliveryWindowBudget::admitted(
        DeliveryWindowWidth::measured(3),
        PatchGroupWidth::measured(1),
        MaintenanceDeltaWidth::measured(1),
        ActiveAllocationScopeWidth::measured(1),
        ActiveSubscriptionAllocationPosture::DeliveryWindowArena,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    )
}
