//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Family> TypedRequiredSuccessor<Schema, Family>
where
    Schema: ApplicationSchema,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    #[doc(hidden)]
    pub fn new_for_test(
        demand: Option<WorthQueryAdmittedOutputDemand<Schema, Family>>,
        pending_claim: Option<SelectedRequiredRefreshClaim>,
        pending_provenance: Option<RequiredSuccessorProvenance>,
        predecessor: Option<SelectedReadyReadmission>,
    ) -> Self {
        Self {
            demand,
            pending_claim,
            pending_provenance,
            predecessor,
        }
    }
}
