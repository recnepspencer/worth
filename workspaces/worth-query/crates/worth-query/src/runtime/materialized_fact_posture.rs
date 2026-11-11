use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::{
    ProjectionMaterializedFactPosture, ProjectionMaterializedFactPostureKind,
};
use crate::subscription::QuerySubscriptionDeliveryCauseKind;
use crate::WorthQueryEvidenceIdentity;

use super::WorthQueryRuntimeLiveSubscriptionState;

pub(super) fn materialized_fact_posture_from_live_subscription_state(
    state: &WorthQueryRuntimeLiveSubscriptionState,
    basis_identity: &WorthQueryEvidenceIdentity,
) -> Result<ProjectionMaterializedFactPosture, IdentityDerivationDenial> {
    Ok(ProjectionMaterializedFactPosture::new(
        materialized_fact_posture_kind_from_live_subscription_state(state),
        state.installation.query_projection().label(),
        basis_identity.as_str(),
        state.installation.support_projection().label(),
        Some(
            state
                .installation
                .installation_projection()
                .label()
                .to_string(),
        ),
    )?)
}

fn materialized_fact_posture_kind_from_live_subscription_state(
    state: &WorthQueryRuntimeLiveSubscriptionState,
) -> ProjectionMaterializedFactPostureKind {
    if state.remask_posture.is_some() {
        ProjectionMaterializedFactPostureKind::Remasked
    } else if state
        .last_delivery
        .as_ref()
        .is_some_and(|delivery| delivery.mixed_cause_delivery().ordered_member_kinds().len() > 1)
    {
        ProjectionMaterializedFactPostureKind::MixedCause
    } else if state.last_delivery.as_ref().is_some_and(|delivery| {
        matches!(
            delivery.delivery_cause_kind(),
            QuerySubscriptionDeliveryCauseKind::FreshnessOnly
                | QuerySubscriptionDeliveryCauseKind::WindowEntry
                | QuerySubscriptionDeliveryCauseKind::WindowExit
                | QuerySubscriptionDeliveryCauseKind::Deadline
                | QuerySubscriptionDeliveryCauseKind::PreviousValueTransition
        )
    }) {
        ProjectionMaterializedFactPostureKind::TimeOnly
    } else if state.async_result_state.is_some() {
        ProjectionMaterializedFactPostureKind::AsyncBacked
    } else {
        ProjectionMaterializedFactPostureKind::Ordinary
    }
}
