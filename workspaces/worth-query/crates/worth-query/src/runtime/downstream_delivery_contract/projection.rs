use super::super::delivery::WorthQueryRuntimeRetainedDelivery;
use super::delivery::WorthQueryRuntimeDownstreamDeliveryParts;
use super::*;

pub(crate) fn project_downstream_delivery(
    contract: &WorthQueryRuntimeDownstreamDeliveryContract,
    state: &WorthQueryRuntimeLiveSubscriptionState,
) -> Result<Option<WorthQueryRuntimeDownstreamDelivery>, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
    let delivery = state.last_delivery.as_ref()?;
    let delivery_class = classify_delivery(delivery, state.async_result_state.as_ref());
    let support_posture = classify_support_posture(state.remask_posture.as_ref());
    let mixed_cause_identity = matches!(
        delivery_class,
        WorthQueryRuntimeDownstreamDeliveryClass::MixedCause
    )
    .then(|| {
        delivery
            .mixed_cause_delivery()
            .mixed_cause_identity()
            .clone()
    });
    let async_result_state_identity = state
        .async_result_state
        .as_ref()
        .map(WorthQueryRuntimeAsyncResultState::result_state_identity)
        .cloned();
    let remask_identity = state
        .remask_posture
        .as_ref()
        .map(WorthQueryRuntimeRemaskPosture::remask_identity)
        .cloned();
    let basis_identity = state.installation.basis_binding_identity().clone();
    let support_identity = state.installation.support_identity().clone();
    let delivery_batch_identity = delivery.delivery_batch_identity().clone();
    let delivery_cause_identity = delivery.delivery_cause_identity().clone();
    let delivery_identity =
        runtime_downstream_delivery_identity(RuntimeDownstreamDeliveryIdentityParts {
            view_name: state.installation.view_name(),
            delivery_batch_identity: &delivery_batch_identity,
            delivery_class,
            delivery_cause_kind: delivery.delivery_cause_kind(),
            delivery_cause_identity: &delivery_cause_identity,
            sequence: delivery.sequence(),
            basis_identity: &basis_identity,
            support_posture,
            support_identity: &support_identity,
            mixed_cause_identity: mixed_cause_identity.as_ref(),
            async_result_state_identity: async_result_state_identity.as_ref(),
            remask_identity: remask_identity.as_ref(),
            runtime_resume_support_identity: contract.runtime_resume_support_identity(),
            durable_resume_support_identity: contract.durable_resume_support_identity(),
        })?;
    Ok(Some(WorthQueryRuntimeDownstreamDelivery::from_projection(
        WorthQueryRuntimeDownstreamDeliveryParts {
            view_name: state.installation.view_name().to_string(),
            delivery_batch_identity,
            delivery_class,
            delivery_cause_kind: delivery.delivery_cause_kind(),
            delivery_cause_identity,
            sequence: delivery.sequence(),
            basis_identity,
            support_posture,
            support_identity,
            mixed_cause_identity,
            async_result_state_identity,
            remask_identity,
            runtime_resume_support_posture: contract.runtime_resume_support_posture(),
            runtime_resume_support_identity: contract.runtime_resume_support_identity().clone(),
            durable_resume_support_posture: contract.durable_resume_support_posture(),
            durable_resume_support_identity: contract.durable_resume_support_identity().clone(),
            delivery_identity,
        },
    )))
}

fn classify_delivery(
    delivery: &WorthQueryRuntimeRetainedDelivery,
    async_result_state: Option<&WorthQueryRuntimeAsyncResultState>,
) -> WorthQueryRuntimeDownstreamDeliveryClass {
    if delivery.delivery_cause_kind() == QuerySubscriptionDeliveryCauseKind::MixedCause {
        WorthQueryRuntimeDownstreamDeliveryClass::MixedCause
    } else if async_result_state.is_some() {
        WorthQueryRuntimeDownstreamDeliveryClass::AsyncBacked
    } else if !delivery.has_relational_patch() {
        WorthQueryRuntimeDownstreamDeliveryClass::TimeOnly
    } else {
        WorthQueryRuntimeDownstreamDeliveryClass::TruthPatch
    }
}

fn classify_support_posture(
    remask_posture: Option<&WorthQueryRuntimeRemaskPosture>,
) -> WorthQueryRuntimeDownstreamSupportPosture {
    match remask_posture.map(WorthQueryRuntimeRemaskPosture::disposition_kind) {
        Some(WorthQueryRuntimeRemaskDispositionKind::Remasked) => {
            WorthQueryRuntimeDownstreamSupportPosture::Remasked
        }
        Some(WorthQueryRuntimeRemaskDispositionKind::Denied) => {
            WorthQueryRuntimeDownstreamSupportPosture::Denied
        }
        None => WorthQueryRuntimeDownstreamSupportPosture::Supported,
    }
}
