use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::ordinary_outcome::{
    WorthQueryOrdinaryRuntimeAsyncPostureKind, WorthQueryOrdinaryRuntimeBasisPostureKind,
    WorthQueryOrdinaryRuntimeCausePostureKind, WorthQueryOrdinaryRuntimePosture,
    WorthQueryOrdinaryRuntimePostureKind, WorthQueryOrdinaryRuntimeRemaskPostureKind,
};
use crate::subscription::QuerySubscriptionDeliveryCauseKind;

use super::{
    WorthQueryRuntimeAsyncResultStateKind, WorthQueryRuntimeDeliveryCoalescingKind,
    WorthQueryRuntimeLiveSubscriptionState,
};

pub(in crate::runtime) fn project_live_subscription_ordinary_runtime_posture(
    state: &WorthQueryRuntimeLiveSubscriptionState,
) -> Result<WorthQueryOrdinaryRuntimePosture, IdentityDerivationDenial> {
    let cause_posture = state
        .last_delivery
        .as_ref()
        .map(|delivery| {
            let mixed_cause_delivery = delivery.mixed_cause_delivery();
            if mixed_cause_delivery.coalescing_kind()
                == WorthQueryRuntimeDeliveryCoalescingKind::Coalesced
                || delivery.delivery_cause_kind() == QuerySubscriptionDeliveryCauseKind::MixedCause
            {
                WorthQueryOrdinaryRuntimeCausePostureKind::MixedCause
            } else if matches!(
                delivery.delivery_cause_kind(),
                QuerySubscriptionDeliveryCauseKind::FreshnessOnly
                    | QuerySubscriptionDeliveryCauseKind::WindowEntry
                    | QuerySubscriptionDeliveryCauseKind::WindowExit
                    | QuerySubscriptionDeliveryCauseKind::Deadline
                    | QuerySubscriptionDeliveryCauseKind::PreviousValueTransition
            ) {
                WorthQueryOrdinaryRuntimeCausePostureKind::TimeOnly
            } else {
                WorthQueryOrdinaryRuntimeCausePostureKind::Ordinary
            }
        })
        .unwrap_or(WorthQueryOrdinaryRuntimeCausePostureKind::Ordinary);
    let async_posture = state
        .async_result_state
        .as_ref()
        .map(|state| ordinary_async_posture_kind(state.kind()));
    let remask_posture = state
        .remask_posture
        .as_ref()
        .map(|posture| ordinary_remask_posture_kind(posture.reason_kind()));
    let basis_posture = state
        .async_result_state
        .as_ref()
        .map(|async_result_state| {
            if async_result_state.basis_identity() != state.installation.basis_binding_identity() {
                WorthQueryOrdinaryRuntimeBasisPostureKind::BasisDrift
            } else if async_result_state.checkpoint_identity()
                != state.active_lane_handle.checkpoint_identity()
            {
                WorthQueryOrdinaryRuntimeBasisPostureKind::GenerationDrift
            } else {
                WorthQueryOrdinaryRuntimeBasisPostureKind::Stable
            }
        })
        .unwrap_or(WorthQueryOrdinaryRuntimeBasisPostureKind::Stable);
    let kind = match state
        .remask_posture
        .as_ref()
        .map(|posture| posture.disposition_kind())
    {
        Some(super::WorthQueryRuntimeRemaskDispositionKind::Remasked) => {
            WorthQueryOrdinaryRuntimePostureKind::Remasked
        }
        Some(super::WorthQueryRuntimeRemaskDispositionKind::Denied) => {
            WorthQueryOrdinaryRuntimePostureKind::Denied
        }
        None => match async_posture {
            Some(WorthQueryOrdinaryRuntimeAsyncPostureKind::Pending) => {
                WorthQueryOrdinaryRuntimePostureKind::Pending
            }
            Some(WorthQueryOrdinaryRuntimeAsyncPostureKind::Current) => {
                WorthQueryOrdinaryRuntimePostureKind::Current
            }
            Some(WorthQueryOrdinaryRuntimeAsyncPostureKind::Failed) => {
                WorthQueryOrdinaryRuntimePostureKind::Failed
            }
            Some(WorthQueryOrdinaryRuntimeAsyncPostureKind::Stale) => {
                WorthQueryOrdinaryRuntimePostureKind::Stale
            }
            Some(WorthQueryOrdinaryRuntimeAsyncPostureKind::Cancelled) => {
                WorthQueryOrdinaryRuntimePostureKind::Cancelled
            }
            Some(WorthQueryOrdinaryRuntimeAsyncPostureKind::Retried) => {
                WorthQueryOrdinaryRuntimePostureKind::Retried
            }
            Some(WorthQueryOrdinaryRuntimeAsyncPostureKind::Revalidating) => {
                WorthQueryOrdinaryRuntimePostureKind::Revalidating
            }
            Some(WorthQueryOrdinaryRuntimeAsyncPostureKind::Superseded) => {
                WorthQueryOrdinaryRuntimePostureKind::Superseded
            }
            Some(WorthQueryOrdinaryRuntimeAsyncPostureKind::Denied) => {
                WorthQueryOrdinaryRuntimePostureKind::Denied
            }
            Some(WorthQueryOrdinaryRuntimeAsyncPostureKind::Unresolved) => {
                WorthQueryOrdinaryRuntimePostureKind::Unresolved
            }
            None => WorthQueryOrdinaryRuntimePostureKind::Current,
        },
    };
    WorthQueryOrdinaryRuntimePosture::new_with_support_identity(
        kind,
        cause_posture,
        async_posture,
        basis_posture,
        remask_posture,
        state.installation.support_identity().clone(),
    )
}

fn ordinary_async_posture_kind(
    kind: WorthQueryRuntimeAsyncResultStateKind,
) -> WorthQueryOrdinaryRuntimeAsyncPostureKind {
    match kind {
        WorthQueryRuntimeAsyncResultStateKind::Pending => {
            WorthQueryOrdinaryRuntimeAsyncPostureKind::Pending
        }
        WorthQueryRuntimeAsyncResultStateKind::Current => {
            WorthQueryOrdinaryRuntimeAsyncPostureKind::Current
        }
        WorthQueryRuntimeAsyncResultStateKind::Failed => {
            WorthQueryOrdinaryRuntimeAsyncPostureKind::Failed
        }
        WorthQueryRuntimeAsyncResultStateKind::Stale => {
            WorthQueryOrdinaryRuntimeAsyncPostureKind::Stale
        }
        WorthQueryRuntimeAsyncResultStateKind::Cancelled => {
            WorthQueryOrdinaryRuntimeAsyncPostureKind::Cancelled
        }
        WorthQueryRuntimeAsyncResultStateKind::Retried => {
            WorthQueryOrdinaryRuntimeAsyncPostureKind::Retried
        }
        WorthQueryRuntimeAsyncResultStateKind::Revalidating => {
            WorthQueryOrdinaryRuntimeAsyncPostureKind::Revalidating
        }
        WorthQueryRuntimeAsyncResultStateKind::Superseded => {
            WorthQueryOrdinaryRuntimeAsyncPostureKind::Superseded
        }
        WorthQueryRuntimeAsyncResultStateKind::Denied => {
            WorthQueryOrdinaryRuntimeAsyncPostureKind::Denied
        }
        WorthQueryRuntimeAsyncResultStateKind::Unresolved => {
            WorthQueryOrdinaryRuntimeAsyncPostureKind::Unresolved
        }
    }
}

fn ordinary_remask_posture_kind(
    kind: super::WorthQueryRuntimeRemaskReasonKind,
) -> WorthQueryOrdinaryRuntimeRemaskPostureKind {
    match kind {
        super::WorthQueryRuntimeRemaskReasonKind::PolicyDrift => {
            WorthQueryOrdinaryRuntimeRemaskPostureKind::PolicyDrift
        }
        super::WorthQueryRuntimeRemaskReasonKind::TenantDrift => {
            WorthQueryOrdinaryRuntimeRemaskPostureKind::TenantDrift
        }
        super::WorthQueryRuntimeRemaskReasonKind::RelationshipProofDrift => {
            WorthQueryOrdinaryRuntimeRemaskPostureKind::RelationshipProofDrift
        }
        super::WorthQueryRuntimeRemaskReasonKind::SchemaContextDrift => {
            WorthQueryOrdinaryRuntimeRemaskPostureKind::SchemaContextDrift
        }
    }
}
