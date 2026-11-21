//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn accepted_checkpoint_identities_for_test(
        &self,
    ) -> Vec<WorthQueryAcceptedOutputCheckpointIdentity> {
        Self::accepted_checkpoint_identities(self)
    }
}
