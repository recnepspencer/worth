//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn finish_checkpoint_owned_for_test(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        claim: WorthQueryOutputClaimIdentity,
        checkpoint: WorthQueryOutputCheckpoint,
        denial: Option<&WorthQueryOutputDemandDenial>,
    ) -> Result<(), CheckpointFinishStop> {
        Self::finish_checkpoint_owned(self, interest, claim, checkpoint, denial)
    }
}
