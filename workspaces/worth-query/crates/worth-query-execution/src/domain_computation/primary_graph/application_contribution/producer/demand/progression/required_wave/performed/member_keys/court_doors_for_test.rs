//! Sealed test doors over this module's private members.
use super::*;

impl PerformedMembers {
    #[doc(hidden)]
    pub fn attach_successor_for_test(
        &mut self,
        readiness: FreshReadiness,
        successor: &WorthQueryOutputDemandKey,
    ) -> Result<PublicationReadiness, WorthQueryOutputDemandDenial> {
        Self::attach_successor(self, readiness, successor)
    }
}
