use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::ordinary_outcome::WorthQueryOrdinaryRuntimePosture;

use super::super::evidence_identities::{
    runtime_live_view_inspection_identity, RuntimeLiveViewInspectionIdentityParts,
};
use super::super::ordinary_runtime_posture::project_live_subscription_ordinary_runtime_posture;
use super::super::{
    WorthQueryAuthorityLane, WorthQueryRuntimeAsyncResultState,
    WorthQueryRuntimeLiveSubscriptionState, WorthQueryRuntimeMixedCauseDelivery,
    WorthQueryRuntimeRemaskPosture,
};
use super::live_counters::WorthQueryLiveSubscriptionInspectionCounters;
use crate::subscription::QuerySubscriptionDeliveryCauseKind;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLiveViewInspection {
    pub(in crate::runtime::inspection) view_name: String,
    pub(in crate::runtime::inspection) authority_lane: WorthQueryAuthorityLane,
    pub(in crate::runtime::inspection) query_identity: WorthQueryEvidenceIdentity,
    pub(in crate::runtime::inspection) view_shape_identity: WorthQueryEvidenceIdentity,
    pub(in crate::runtime::inspection) subscription_family: String,
    pub(in crate::runtime::inspection) subscription_family_identity: WorthQueryEvidenceIdentity,
    pub(in crate::runtime::inspection) subscription_declaration_identity:
        WorthQueryEvidenceIdentity,
    pub(in crate::runtime::inspection) bridge_declaration_identity: WorthQueryEvidenceIdentity,
    pub(in crate::runtime::inspection) admission_identity: WorthQueryEvidenceIdentity,
    pub(in crate::runtime::inspection) activation_identity: WorthQueryEvidenceIdentity,
    pub(in crate::runtime::inspection) basis_binding_identity: WorthQueryEvidenceIdentity,
    pub(in crate::runtime::inspection) signal_strategy_identity: WorthQueryEvidenceIdentity,
    pub(in crate::runtime::inspection) active_lane_identity: WorthQueryEvidenceIdentity,
    pub(in crate::runtime::inspection) consumer_attachment_identity: WorthQueryEvidenceIdentity,
    pub(in crate::runtime::inspection) consumer_identity: WorthQueryEvidenceIdentity,
    pub(in crate::runtime::inspection) delivery_cursor_identity: WorthQueryEvidenceIdentity,
    pub(in crate::runtime::inspection) subscription_budget_policy: String,
    pub(in crate::runtime::inspection) active_lifecycle_budget_policy: String,
    pub(in crate::runtime::inspection) consumer_attachment_budget_policy: String,
    pub(in crate::runtime::inspection) runtime_budget_identity: WorthQueryEvidenceIdentity,
    pub(in crate::runtime::inspection) support_identity: WorthQueryEvidenceIdentity,
    pub(in crate::runtime::inspection) last_delivery_cause_kind:
        Option<QuerySubscriptionDeliveryCauseKind>,
    pub(in crate::runtime::inspection) last_delivery_cause_identity:
        Option<WorthQueryEvidenceIdentity>,
    pub(in crate::runtime::inspection) last_delivery_had_relational_patch: bool,
    pub(in crate::runtime::inspection) mixed_cause_delivery:
        Option<WorthQueryRuntimeMixedCauseDelivery>,
    pub(in crate::runtime::inspection) ordinary_runtime_posture: WorthQueryOrdinaryRuntimePosture,
    pub(in crate::runtime::inspection) async_result_state:
        Option<WorthQueryRuntimeAsyncResultState>,
    pub(in crate::runtime::inspection) remask_posture: Option<WorthQueryRuntimeRemaskPosture>,
    pub(in crate::runtime::inspection) installation_identity: WorthQueryEvidenceIdentity,
    pub(in crate::runtime::inspection) counters: WorthQueryLiveSubscriptionInspectionCounters,
    pub(in crate::runtime::inspection) inspection_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryLiveViewInspection {
    pub(in crate::runtime) fn from_state(state: &WorthQueryRuntimeLiveSubscriptionState) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let installation = &state.installation;
        let counters =
            WorthQueryLiveSubscriptionInspectionCounters::from_installation(installation)?;
        let last_delivery_cause_kind = state
            .last_delivery
            .as_ref()
            .map(|delivery| delivery.delivery_cause_kind());
        let last_delivery_cause_identity = state
            .last_delivery
            .as_ref()
            .map(|delivery| delivery.delivery_cause_identity().clone());
        let last_delivery_had_relational_patch = state
            .last_delivery
            .as_ref()
            .map(|delivery| delivery.has_relational_patch())
            .unwrap_or(false);
        let mixed_cause_delivery = state
            .last_delivery
            .as_ref()
            .map(|delivery| delivery.mixed_cause_delivery().clone());
        let ordinary_runtime_posture = project_live_subscription_ordinary_runtime_posture(state);
        let async_result_state = state.async_result_state.clone();
        let remask_posture = state.remask_posture.clone();
        let mixed_cause_identity = mixed_cause_delivery
            .as_ref()
            .map(WorthQueryRuntimeMixedCauseDelivery::mixed_cause_identity);
        let inspection_identity =
            runtime_live_view_inspection_identity(RuntimeLiveViewInspectionIdentityParts {
                view_name: installation.view_name(),
                authority_lane: installation.authority_lane(),
                query_identity: installation.query_identity(),
                view_shape_identity: installation.view_shape_identity(),
                subscription_family: installation.subscription_family(),
                subscription_family_identity: installation.subscription_family_identity(),
                subscription_declaration_identity: installation.subscription_declaration_identity(),
                bridge_declaration_identity: installation.bridge_declaration_identity(),
                admission_identity: installation.admission_identity(),
                activation_identity: installation.activation_identity(),
                basis_binding_identity: installation.basis_binding_identity(),
                signal_strategy_identity: installation.signal_strategy_identity(),
                active_lane_identity: installation.active_lane_identity(),
                consumer_attachment_identity: installation.consumer_attachment_identity(),
                consumer_identity: installation.consumer_identity(),
                delivery_cursor_identity: installation.delivery_cursor_identity(),
                subscription_budget_policy: installation.subscription_budget_policy(),
                active_lifecycle_budget_policy: installation.active_lifecycle_budget_policy(),
                consumer_attachment_budget_policy: installation.consumer_attachment_budget_policy(),
                runtime_budget_identity: installation.runtime_budget_identity(),
                support_identity: installation.support_identity(),
                last_delivery_cause_kind,
                last_delivery_cause_identity: last_delivery_cause_identity.as_ref(),
                last_delivery_had_relational_patch,
                mixed_cause_identity,
                ordinary_runtime_posture: Some(&ordinary_runtime_posture),
                async_result_state: async_result_state.as_ref(),
                remask_posture: remask_posture.as_ref(),
                installation_identity: installation.installation_identity(),
                counter_inspection_identity: counters.counter_inspection_identity(),
            })?;

        Ok(Self {
            view_name: installation.view_name().to_string(),
            authority_lane: installation.authority_lane(),
            query_identity: installation.query_identity().clone(),
            view_shape_identity: installation.view_shape_identity().clone(),
            subscription_family: installation.subscription_family().to_string(),
            subscription_family_identity: installation.subscription_family_identity().clone(),
            subscription_declaration_identity: installation
                .subscription_declaration_identity()
                .clone(),
            bridge_declaration_identity: installation.bridge_declaration_identity().clone(),
            admission_identity: installation.admission_identity().clone(),
            activation_identity: installation.activation_identity().clone(),
            basis_binding_identity: installation.basis_binding_identity().clone(),
            signal_strategy_identity: installation.signal_strategy_identity().clone(),
            active_lane_identity: installation.active_lane_identity().clone(),
            consumer_attachment_identity: installation.consumer_attachment_identity().clone(),
            consumer_identity: installation.consumer_identity().clone(),
            delivery_cursor_identity: installation.delivery_cursor_identity().clone(),
            subscription_budget_policy: installation.subscription_budget_policy().to_string(),
            active_lifecycle_budget_policy: installation
                .active_lifecycle_budget_policy()
                .to_string(),
            consumer_attachment_budget_policy: installation
                .consumer_attachment_budget_policy()
                .to_string(),
            runtime_budget_identity: installation.runtime_budget_identity().clone(),
            support_identity: installation.support_identity().clone(),
            last_delivery_cause_kind,
            last_delivery_cause_identity,
            last_delivery_had_relational_patch,
            mixed_cause_delivery,
            ordinary_runtime_posture,
            async_result_state,
            remask_posture,
            installation_identity: installation.installation_identity().clone(),
            counters,
            inspection_identity,
        })
    }

    pub fn view_name(&self) -> &str {
        &self.view_name
    }

    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.authority_lane
    }

    pub fn subscription_family(&self) -> &str {
        &self.subscription_family
    }

    pub fn subscription_budget_policy(&self) -> &str {
        &self.subscription_budget_policy
    }

    pub fn active_lifecycle_budget_policy(&self) -> &str {
        &self.active_lifecycle_budget_policy
    }

    pub fn consumer_attachment_budget_policy(&self) -> &str {
        &self.consumer_attachment_budget_policy
    }

    pub fn last_delivery_cause_kind(&self) -> Option<QuerySubscriptionDeliveryCauseKind> {
        self.last_delivery_cause_kind
    }

    pub fn last_delivery_had_relational_patch(&self) -> bool {
        self.last_delivery_had_relational_patch
    }

    pub fn mixed_cause_delivery(&self) -> Option<&WorthQueryRuntimeMixedCauseDelivery> {
        self.mixed_cause_delivery.as_ref()
    }

    pub fn ordinary_runtime_posture(&self) -> &WorthQueryOrdinaryRuntimePosture {
        &self.ordinary_runtime_posture
    }

    pub fn async_result_state(&self) -> Option<&WorthQueryRuntimeAsyncResultState> {
        self.async_result_state.as_ref()
    }

    pub fn remask_posture(&self) -> Option<&WorthQueryRuntimeRemaskPosture> {
        self.remask_posture.as_ref()
    }

    pub fn counters(&self) -> &WorthQueryLiveSubscriptionInspectionCounters {
        &self.counters
    }

    pub fn inspection_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.inspection_identity
    }
}
