use super::delivery::WorthQueryRuntimeRetainedDelivery;
use super::*;
use crate::subscription::{
    advance_subscription_acknowledgement, emit_query_time_only_delivery_batch,
    open_query_delivery_window, ActiveAllocationScopeWidth, ActiveSubscriptionAllocationPosture,
    DeliveryBackpressurePolicy, DeliveryWindowWidth, MaintenanceDeltaWidth, PatchGroupWidth,
    QueryDeliveryDenialKind, QueryDeliveryError, QueryDeliveryWindowBudget,
    QuerySubscriptionDeliveryCause, QuerySubscriptionDeliveryCauseKind,
};

pub(crate) fn emit_time_only_live_subscription_delivery(
    active_subscriptions: &mut ActiveSubscriptionRuntime,
    live_subscriptions: &mut BTreeMap<
        WorthQueryLiveArtifactTarget,
        WorthQueryRuntimeLiveSubscriptionState,
    >,
    view_name: &str,
    cause_kind: QuerySubscriptionDeliveryCauseKind,
    evidence_digest: &str,
    previous_value_available: bool,
    temporal_basis_fresh: bool,
) -> Result<WorthQueryRuntimeDeliveryBatch, WorthQueryRuntimeError> {
    let target = WorthQueryLiveArtifactTarget::from_view_name(view_name);
    let state = live_subscriptions
        .get_mut(&target)
        .ok_or_else(|| WorthQueryRuntimeError::MissingLiveSubscription(view_name.to_string()))?;
    if cause_kind.requires_previous_value() && !previous_value_available {
        return Err(WorthQueryRuntimeError::LiveSubscriptionInstallation {
            view_name: view_name.to_string(),
            stage: "time-only-delivery",
            message: "MissingPreviousValueEvidence".to_string(),
        });
    }
    if !cause_kind.has_relational_patch() && !temporal_basis_fresh {
        return Err(WorthQueryRuntimeError::LiveSubscriptionInstallation {
            view_name: view_name.to_string(),
            stage: "time-only-delivery",
            message: "StaleTemporalBasis".to_string(),
        });
    }

    let window = open_query_delivery_window(
        active_subscriptions,
        &state.consumer_attachment,
        runtime_time_only_delivery_window_budget(),
    )
    .map_err(|error| time_only_delivery_error(view_name, error))?;
    let cause = QuerySubscriptionDeliveryCause::time_only(cause_kind, evidence_digest)?;
    let batch = emit_query_time_only_delivery_batch(active_subscriptions, window, cause)
        .map_err(|error| time_only_delivery_error(view_name, error))?;
    let delivery_receipt = batch.receipt().clone();
    let runtime_batch =
        WorthQueryRuntimeDeliveryBatch::from_query_delivery(view_name, &batch, None, None, None)?;
    state.last_delivery = Some(WorthQueryRuntimeRetainedDelivery::from_batch(
        &runtime_batch,
    ));
    state.delivery_batches.push(runtime_batch.clone());
    state.consumer_attachment = advance_subscription_acknowledgement(
        active_subscriptions,
        state.consumer_attachment.clone(),
        delivery_receipt,
    )
    .map_err(
        |error| WorthQueryRuntimeError::LiveSubscriptionInstallation {
            view_name: view_name.to_string(),
            stage: "time-only-delivery-acknowledgement",
            message: format!("{error:?}"),
        },
    )?;
    Ok(runtime_batch)
}

fn runtime_time_only_delivery_window_budget() -> QueryDeliveryWindowBudget {
    QueryDeliveryWindowBudget::admitted(
        DeliveryWindowWidth::measured(1),
        PatchGroupWidth::measured(1),
        MaintenanceDeltaWidth::measured(1),
        ActiveAllocationScopeWidth::measured(1),
        ActiveSubscriptionAllocationPosture::DeliveryWindowArena,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    )
}

fn time_only_delivery_error(view_name: &str, error: QueryDeliveryError) -> WorthQueryRuntimeError {
    match error.denial_kind() {
        QueryDeliveryDenialKind::MissingPreviousValueEvidence => {
            WorthQueryRuntimeError::LiveSubscriptionInstallation {
                view_name: view_name.to_string(),
                stage: "time-only-delivery",
                message: "MissingPreviousValueEvidence".to_string(),
            }
        }
        QueryDeliveryDenialKind::StaleTemporalBasis => {
            WorthQueryRuntimeError::LiveSubscriptionInstallation {
                view_name: view_name.to_string(),
                stage: "time-only-delivery",
                message: "StaleTemporalBasis".to_string(),
            }
        }
        _ => WorthQueryRuntimeError::LiveSubscriptionInstallation {
            view_name: view_name.to_string(),
            stage: "time-only-delivery",
            message: format!("{error:?}"),
        },
    }
}

impl WorthQueryRuntime {
    pub(crate) fn emit_time_only_delivery(
        &mut self,
        view_name: &str,
        cause_kind: QuerySubscriptionDeliveryCauseKind,
        evidence_digest: &str,
        previous_value_available: bool,
        temporal_basis_fresh: bool,
    ) -> Result<WorthQueryRuntimeDeliveryBatch, WorthQueryRuntimeError> {
        emit_time_only_live_subscription_delivery(
            &mut self.active_subscriptions,
            &mut self.live_subscriptions,
            view_name,
            cause_kind,
            evidence_digest,
            previous_value_available,
            temporal_basis_fresh,
        )
    }
}
