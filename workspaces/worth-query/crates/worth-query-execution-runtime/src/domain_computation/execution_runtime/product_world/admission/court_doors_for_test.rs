//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn admit_product_branch_for_test(
        &self,
        identity: &worth_runtime_world::facade::ProductBranchIdentity,
    ) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial> {
        Self::admit_product_branch(self, identity)
    }
}

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn admit_product_occurrence_for_test(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial> {
        Self::admit_product_occurrence(self, occurrence)
    }
}

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn lease_from_observation_for_test(
        &self,
        observation: worth_runtime_world::facade::ProductBranchObservation,
    ) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial> {
        Self::lease_from_observation(self, observation)
    }
}

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn with_product_observation_for_test<Output>(
        &self,
        identity: &worth_runtime_world::facade::ProductBranchIdentity,
        admit: impl FnOnce(
            worth_runtime_world::facade::ProductBranchObservation,
        ) -> Result<Output, WorthQueryProductBranchAdmissionDenial>,
    ) -> Result<Output, WorthQueryProductBranchAdmissionDenial> {
        Self::with_product_observation(self, identity, admit)
    }
}
