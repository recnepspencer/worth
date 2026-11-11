use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::ordinary_outcome::WorthQueryOrdinaryRuntimePosture;

use super::super::ordinary_runtime_posture::project_live_subscription_ordinary_runtime_posture;

use super::super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedLiveLifecyclePosture {
    Active,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedLiveSubscriptionFamily {
    DetailExact,
    CollectionMembership,
    BoundedMaterialization,
    GroupedCollectionMembership,
    InspectorDetailExact,
}

impl WorthQueryManagedLiveSubscriptionFamily {
    fn from_runtime(family: &crate::subscription::QuerySubscriptionFamily) -> Self {
        match family {
            crate::subscription::QuerySubscriptionFamily::DetailExact => Self::DetailExact,
            crate::subscription::QuerySubscriptionFamily::CollectionMembership => {
                Self::CollectionMembership
            }
            crate::subscription::QuerySubscriptionFamily::BoundedMaterialization => {
                Self::BoundedMaterialization
            }
            crate::subscription::QuerySubscriptionFamily::GroupedCollectionMembership => {
                Self::GroupedCollectionMembership
            }
            crate::subscription::QuerySubscriptionFamily::InspectorDetailExact => {
                Self::InspectorDetailExact
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryManagedLiveActivationWork {
    family_selection_count: u64,
    declaration_count: u64,
    admission_count: u64,
    activation_input_count: u64,
    active_lane_creation_count: u64,
    active_lane_join_count: u64,
    consumer_attachment_count: u64,
}

impl WorthQueryManagedLiveActivationWork {
    pub fn family_selection_count(&self) -> u64 {
        self.family_selection_count
    }

    pub fn declaration_count(&self) -> u64 {
        self.declaration_count
    }

    pub fn admission_count(&self) -> u64 {
        self.admission_count
    }

    pub fn activation_input_count(&self) -> u64 {
        self.activation_input_count
    }

    pub fn active_lane_creation_count(&self) -> u64 {
        self.active_lane_creation_count
    }

    pub fn active_lane_join_count(&self) -> u64 {
        self.active_lane_join_count
    }

    pub fn consumer_attachment_count(&self) -> u64 {
        self.consumer_attachment_count
    }

    fn from_installation(installation: &WorthQueryRuntimeLiveSubscriptionInstallation) -> Self {
        let declaration = installation.counters();
        let active_lane = installation.active_lane_counters();
        let consumer_attachment = installation.consumer_attachment_counters();
        Self {
            family_selection_count: declaration.family_selection_count(),
            declaration_count: declaration.declaration_count(),
            admission_count: declaration.admission_count(),
            activation_input_count: declaration.activation_input_count(),
            active_lane_creation_count: active_lane.active_lane_creation_count(),
            active_lane_join_count: active_lane.active_lane_join_count(),
            consumer_attachment_count: consumer_attachment.consumer_attachment_count(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryManagedLiveLifecycleObservation {
    resource_name: String,
    posture: WorthQueryManagedLiveLifecyclePosture,
    authority_lane: WorthQueryAuthorityLane,
    subscription_family: WorthQueryManagedLiveSubscriptionFamily,
    activation_work: WorthQueryManagedLiveActivationWork,
    installation_identity: WorthQueryEvidenceIdentity,
    basis_binding_identity: WorthQueryEvidenceIdentity,
    runtime_posture: WorthQueryOrdinaryRuntimePosture,
    pending_delivery_batch_count: usize,
    last_delivery_sequence: Option<u64>,
}

impl WorthQueryManagedLiveLifecycleObservation {
    pub fn resource_name(&self) -> &str {
        &self.resource_name
    }

    pub fn posture(&self) -> WorthQueryManagedLiveLifecyclePosture {
        self.posture
    }

    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.authority_lane
    }

    pub fn subscription_family(&self) -> WorthQueryManagedLiveSubscriptionFamily {
        self.subscription_family
    }

    pub fn activation_work(&self) -> &WorthQueryManagedLiveActivationWork {
        &self.activation_work
    }

    pub fn installation_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.installation_identity
    }

    pub fn basis_binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_binding_identity
    }

    pub fn runtime_posture(&self) -> &WorthQueryOrdinaryRuntimePosture {
        &self.runtime_posture
    }

    pub fn pending_delivery_batch_count(&self) -> usize {
        self.pending_delivery_batch_count
    }

    pub fn last_delivery_sequence(&self) -> Option<u64> {
        self.last_delivery_sequence
    }

    pub(super) fn from_state(
        state: &WorthQueryRuntimeLiveSubscriptionState,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            resource_name: state.installation.view_name().to_string(),
            posture: WorthQueryManagedLiveLifecyclePosture::Active,
            authority_lane: state.installation.authority_lane(),
            subscription_family: WorthQueryManagedLiveSubscriptionFamily::from_runtime(
                state.installation.subscription_family_kind(),
            ),
            activation_work: WorthQueryManagedLiveActivationWork::from_installation(
                &state.installation,
            ),
            installation_identity: state.installation.installation_identity().clone(),
            basis_binding_identity: state.installation.basis_binding_identity().clone(),
            runtime_posture: project_live_subscription_ordinary_runtime_posture(state)?,
            pending_delivery_batch_count: state.delivery_batches.len(),
            last_delivery_sequence: state
                .last_delivery
                .as_ref()
                .map(|delivery| delivery.sequence()),
        })
    }
}

impl WorthQueryRuntime {
    pub(crate) fn observe_managed_live_view<T>(
        &mut self,
        view: &WorthQueryLiveView<T>,
    ) -> Result<WorthQueryManagedLiveLifecycleObservation, WorthQueryRuntimeError> {
        self.reap_abandoned_managed_live_resources()?;
        let target = WorthQueryLiveArtifactTarget::from_subscription_installation(
            view.subscription_installation(),
        );
        let state = self.live_subscriptions.get(&target).ok_or_else(|| {
            WorthQueryRuntimeError::MissingLiveSubscription(view.name().to_string())
        })?;
        if state.installation != *view.subscription_installation()
            || state.read_authority_binding.is_none()
        {
            return Err(WorthQueryRuntimeError::MissingLiveSubscription(
                view.name().to_string(),
            ));
        }
        Ok(WorthQueryManagedLiveLifecycleObservation::from_state(
            state,
        )?)
    }
}
