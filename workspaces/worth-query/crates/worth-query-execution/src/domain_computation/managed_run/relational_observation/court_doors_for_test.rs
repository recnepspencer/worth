//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryManagedRelationalObservation {
    #[doc(hidden)]
    pub fn retain_for_test( source: &RuntimeBridgeRelationalSource, basis: AdmittedRelationalBranchBasis, current_at_admission: bool) -> Result<Self, worth_relational::facade::branch::RelationalBranchBasisDenial> {
        Self::retain(source, basis, current_at_admission)
    }
}
