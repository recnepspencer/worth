//! Sealed test doors over this module's private members.
use super::*;

impl PerformedMembers {
    #[doc(hidden)]
    pub fn inspection_for_test(&self) -> InspectionStamp {
        Self::inspection(self)
    }
}

impl PerformedMembers {
    #[doc(hidden)]
    pub fn reinspection_for_test(
        &self,
        key: &WorthQueryOutputDemandKey,
        inspected: InspectionStamp,
    ) -> Result<Option<ReinspectionCredit>, WorthQueryOutputDemandDenial> {
        Self::reinspection(self, key, inspected)
    }
}
