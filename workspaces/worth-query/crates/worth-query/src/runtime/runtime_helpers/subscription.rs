use super::*;

pub(in crate::runtime) fn subscription_dimensions_for_request(
    request: &DeclarativeLiveQueryRequest,
    view_family: LiveViewShapeFamily,
) -> Result<QuerySubscriptionAdmissionDimensions, WorthQueryRuntimeError> {
    let projection_width = NonZeroUsize::new(request.projection().len().max(1))
        .expect("projection width is forced non-zero");
    let ordering_width = NonZeroUsize::new(1).expect("ordering width literal is non-zero");
    let metadata_width = NonZeroUsize::new(1).expect("metadata width literal is non-zero");

    match (request.view_shape(), view_family) {
        (DeclarativeLiveViewShape::ListSplice | DeclarativeLiveViewShape::Table, _) => {
            Ok(QuerySubscriptionAdmissionDimensions::collection_membership(
                projection_width,
                ordering_width,
            ))
        }
        (DeclarativeLiveViewShape::Detail, _) => Ok(
            QuerySubscriptionAdmissionDimensions::detail_exact(projection_width),
        ),
        (
            DeclarativeLiveViewShape::InspectorObserved
            | DeclarativeLiveViewShape::InspectorFocused { .. }
            | DeclarativeLiveViewShape::IdentityAwareInspectorFocused { .. },
            _,
        ) => Ok(
            QuerySubscriptionAdmissionDimensions::inspector_detail_exact(
                projection_width,
                metadata_width,
            ),
        ),
        (DeclarativeLiveViewShape::KanbanGrouped { .. }, _) => Ok(
            QuerySubscriptionAdmissionDimensions::grouped_collection_membership(
                projection_width,
                ordering_width,
                NonZeroUsize::new(1).expect("grouping width literal is non-zero"),
                metadata_width,
            ),
        ),
    }
}

pub(in crate::runtime) fn runtime_family_budget() -> QuerySubscriptionWorkBudget {
    QuerySubscriptionWorkBudget::scratch_buffer_only(64, 64, 64, 512, 1)
}

pub(in crate::runtime) fn runtime_slice_budget() -> QuerySubscriptionSliceBudget {
    QuerySubscriptionSliceBudget::scratch_buffer_only(64, 64, 64, 64, 64, 64, 64, 64)
}

pub(in crate::runtime) fn runtime_bridge_lowering_budget() -> QuerySubscriptionBridgeLoweringBudget
{
    QuerySubscriptionBridgeLoweringBudget::admitted(1, 64, 64, 64, 64)
}

pub(in crate::runtime) fn runtime_subscription_admission_budget() -> QuerySubscriptionAdmissionBudget
{
    QuerySubscriptionAdmissionBudget::admitted(64, 64, 64, 64, 64)
}

pub(in crate::runtime) fn runtime_active_lifecycle_budget() -> ActiveSubscriptionWorkBudget {
    ActiveSubscriptionWorkBudget::admitted(
        ActiveRegistryLookupWidth::measured(1),
        ActiveFanoutWidth::measured(1),
        ActiveAllocationScopeWidth::measured(1),
        ActiveSubscriptionAllocationPosture::LifecycleArena,
    )
}

pub(in crate::runtime) fn runtime_consumer_attachment_budget(
) -> SubscriptionConsumerAttachmentBudget {
    SubscriptionConsumerAttachmentBudget::admitted(
        ActiveFanoutWidth::measured(1),
        ConsumerDeliveryPacingWidth::measured(1),
        ActiveAllocationScopeWidth::measured(1),
        DeliveryBackpressurePolicy::RetainWithinWindow,
    )
}

pub(in crate::runtime) fn runtime_subscription_budget_policy() -> Result<
    WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
> {
    Ok(
        WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity::subscription_policy(
            [
                RUNTIME_SUBSCRIPTION_FAMILY_BUDGET_POLICY,
                RUNTIME_SUBSCRIPTION_SLICE_BUDGET_POLICY,
                RUNTIME_SUBSCRIPTION_BRIDGE_BUDGET_POLICY,
                RUNTIME_SUBSCRIPTION_ADMISSION_BUDGET_POLICY,
            ]
            .join(" / "),
        )?,
    )
}

pub(in crate::runtime) fn runtime_active_lifecycle_budget_policy() -> Result<
    WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
> {
    Ok(
        WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity::active_lifecycle_policy(
            RUNTIME_ACTIVE_LIFECYCLE_BUDGET_POLICY,
        )?,
    )
}

pub(in crate::runtime) fn runtime_consumer_attachment_budget_policy() -> Result<
    WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
> {
    Ok(
        WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity::consumer_attachment_policy(
            RUNTIME_CONSUMER_ATTACHMENT_BUDGET_POLICY,
        )?,
    )
}

#[cfg(test)]
pub(in crate::runtime) fn runtime_subscription_budget_digest() -> Result<
    crate::WorthQueryEvidenceIdentity,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
> {
    Ok(crate::WorthQueryEvidenceIdentity::compose(
        crate::evidence_identity::WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
    )
    .field_shape(
        crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
        "runtime_live_subscription_budget_policy_v1",
    )
    .field_evidence_identity(
        crate::evidence_identity::WorthQueryEvidenceTag::new("subscription_budget_policy"),
        runtime_subscription_budget_policy()?.evidence_identity(),
    )
    .field_evidence_identity(
        crate::evidence_identity::WorthQueryEvidenceTag::new("active_lifecycle_budget_policy"),
        runtime_active_lifecycle_budget_policy()?.evidence_identity(),
    )
    .field_evidence_identity(
        crate::evidence_identity::WorthQueryEvidenceTag::new("consumer_attachment_budget_policy"),
        runtime_consumer_attachment_budget_policy()?.evidence_identity(),
    )
    .seal()?)
}
