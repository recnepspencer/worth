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

impl WorthQueryLiveDeliverySourceState {
    #[doc(hidden)]
    pub fn partitions_for_test(
        &self,
    ) -> &BTreeMap<WorthQueryLiveProductKey, WorthQueryLiveProductPartition> {
        &self.partitions
    }
}

impl WorthQueryLiveDeliverySourceState {
    #[doc(hidden)]
    pub fn subscriber_count_for_test(&self) -> &usize {
        &self.subscriber_count
    }
}

impl WorthQueryLiveDeliverySourceState {
    #[doc(hidden)]
    pub fn subscriber_count_mut_for_test(&mut self) -> &mut usize {
        &mut self.subscriber_count
    }
}

impl WorthQueryLiveProductPartition {
    #[doc(hidden)]
    pub fn subscriber_count_mut_for_test(&mut self) -> &mut usize {
        &mut self.subscriber_count
    }
}

impl WorthQueryLiveDeliverySource {
    #[doc(hidden)]
    pub fn close_for_test(&self) -> usize {
        Self::close(self)
    }
}

impl WorthQueryLiveDeliverySourceState {
    #[doc(hidden)]
    pub fn partitions_mut_for_test(
        &mut self,
    ) -> &mut BTreeMap<WorthQueryLiveProductKey, WorthQueryLiveProductPartition> {
        &mut self.partitions
    }
}
