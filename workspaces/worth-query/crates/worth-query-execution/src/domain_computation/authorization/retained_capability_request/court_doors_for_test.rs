//! Sealed test doors over this module's private members.
use super::*;

impl RetainedCapabilityRequestData {
    #[doc(hidden)]
    pub fn elevation_for_test(&self) -> &Option<worth_relational::facade::identity::EntityId> {
        &self.elevation
    }
}

impl WorthQueryRetainedCapabilityRequest {
    #[doc(hidden)]
    pub fn data_mut_for_test(&mut self) -> &mut Arc<RetainedCapabilityRequestData> {
        &mut self.data
    }
}

impl WorthQueryRetainedCapabilityRequest {
    #[doc(hidden)]
    pub fn matches_elevated_request_for_test(
        &self,
        candidate: &Self,
        elevation: worth_relational::facade::identity::EntityId,
    ) -> bool {
        Self::matches_elevated_request(self, candidate, elevation)
    }
}

impl WorthQueryRetainedCapabilityRequest {
    #[doc(hidden)]
    pub fn matches_use_for_test(&self, candidate: &Self) -> bool {
        Self::matches_use(self, candidate)
    }
}
