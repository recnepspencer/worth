use crate::evidence_identity::WorthQueryEvidenceIdentity;

use super::super::evidence_identities::runtime_live_subscription_counter_inspection_identity;
use super::super::WorthQueryRuntimeLiveSubscriptionInstallation;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLiveSubscriptionInspectionCounters {
    declaration_counter_identity: WorthQueryEvidenceIdentity,
    active_lane_counter_identity: WorthQueryEvidenceIdentity,
    consumer_attachment_counter_identity: WorthQueryEvidenceIdentity,
    family_selection_count: u64,
    declaration_count: u64,
    bridge_lowering_count: u64,
    admission_count: u64,
    activation_input_count: u64,
    active_lane_admission_count: u64,
    active_lane_creation_count: u64,
    active_lane_join_count: u64,
    active_lane_handle_issue_count: u64,
    consumer_attachment_count: u64,
    consumer_attachment_denial_count: u64,
    counter_inspection_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryLiveSubscriptionInspectionCounters {
    pub(in crate::runtime) fn from_installation(
        installation: &WorthQueryRuntimeLiveSubscriptionInstallation,
    ) -> Self {
        let counters = installation.counters();
        let active_lane_counters = installation.active_lane_counters();
        let consumer_attachment_counters = installation.consumer_attachment_counters();
        let declaration_counter_identity = counters.evidence_identity().clone();
        let active_lane_counter_identity = active_lane_counters.evidence_identity().clone();
        let consumer_attachment_counter_identity =
            consumer_attachment_counters.evidence_identity().clone();
        let family_selection_count = counters.family_selection_count();
        let declaration_count = counters.declaration_count();
        let bridge_lowering_count = counters.bridge_lowering_count();
        let admission_count = counters.admission_count();
        let activation_input_count = counters.activation_input_count();
        let active_lane_admission_count = active_lane_counters.active_lane_admission_count();
        let active_lane_creation_count = active_lane_counters.active_lane_creation_count();
        let active_lane_join_count = active_lane_counters.active_lane_join_count();
        let active_lane_handle_issue_count = active_lane_counters.active_lane_handle_issue_count();
        let consumer_attachment_count = consumer_attachment_counters.consumer_attachment_count();
        let consumer_attachment_denial_count =
            consumer_attachment_counters.consumer_attachment_denial_count();
        let counter_inspection_identity = runtime_live_subscription_counter_inspection_identity(
            &declaration_counter_identity,
            &active_lane_counter_identity,
            &consumer_attachment_counter_identity,
            family_selection_count,
            declaration_count,
            bridge_lowering_count,
            admission_count,
            activation_input_count,
            active_lane_admission_count,
            active_lane_creation_count,
            active_lane_join_count,
            active_lane_handle_issue_count,
            consumer_attachment_count,
            consumer_attachment_denial_count,
        );

        Self {
            declaration_counter_identity,
            active_lane_counter_identity,
            consumer_attachment_counter_identity,
            family_selection_count,
            declaration_count,
            bridge_lowering_count,
            admission_count,
            activation_input_count,
            active_lane_admission_count,
            active_lane_creation_count,
            active_lane_join_count,
            active_lane_handle_issue_count,
            consumer_attachment_count,
            consumer_attachment_denial_count,
            counter_inspection_identity,
        }
    }

    pub fn declaration_counter_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.declaration_counter_identity
    }

    pub fn declaration_counter_for_reporting(&self) -> &str {
        self.declaration_counter_identity.as_str()
    }

    pub fn active_lane_counter_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.active_lane_counter_identity
    }

    pub fn active_lane_counter_for_reporting(&self) -> &str {
        self.active_lane_counter_identity.as_str()
    }

    pub fn consumer_attachment_counter_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.consumer_attachment_counter_identity
    }

    pub fn consumer_attachment_counter_for_reporting(&self) -> &str {
        self.consumer_attachment_counter_identity.as_str()
    }

    pub fn family_selection_count(&self) -> u64 {
        self.family_selection_count
    }

    pub fn declaration_count(&self) -> u64 {
        self.declaration_count
    }

    pub fn bridge_lowering_count(&self) -> u64 {
        self.bridge_lowering_count
    }

    pub fn admission_count(&self) -> u64 {
        self.admission_count
    }

    pub fn activation_input_count(&self) -> u64 {
        self.activation_input_count
    }

    pub fn active_lane_admission_count(&self) -> u64 {
        self.active_lane_admission_count
    }

    pub fn active_lane_creation_count(&self) -> u64 {
        self.active_lane_creation_count
    }

    pub fn active_lane_join_count(&self) -> u64 {
        self.active_lane_join_count
    }

    pub fn active_lane_handle_issue_count(&self) -> u64 {
        self.active_lane_handle_issue_count
    }

    pub fn consumer_attachment_count(&self) -> u64 {
        self.consumer_attachment_count
    }

    pub fn consumer_attachment_denial_count(&self) -> u64 {
        self.consumer_attachment_denial_count
    }

    pub fn counter_inspection_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.counter_inspection_identity
    }

    pub fn counter_inspection_for_reporting(&self) -> &str {
        self.counter_inspection_identity.as_str()
    }
}
