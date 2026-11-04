//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryLiveDeliverySource {
    #[doc(hidden)]
    pub fn state_for_test(&self) -> &Arc<Mutex<WorthQueryLiveDeliverySourceState>> {
        &self.state
    }
}

impl WorthQueryLiveProductKey {
    #[doc(hidden)]
    pub fn from_observation_for_test(
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Self {
        Self::from_observation(observation)
    }
}

impl WorthQueryLiveDeliverySource {
    #[doc(hidden)]
    pub fn reserve_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Result<WorthQueryLivePublicationReservation, WorthQueryLiveAdmissionDenial> {
        Self::reserve(self, execution, observation)
    }
}
