//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn prepare_selected_checkpoint_finish_for_test<'a>(
        &'a self,
        interest: &'a WorthQueryOutputDemandInterest,
    ) -> Result<PreparedSelectedCheckpointFinish<'a>, WorthQueryOutputDemandDenial> {
        Self::prepare_selected_checkpoint_finish(self, interest)
    }
}

impl PreparedSelectedCheckpointFinish<'_> {
    #[doc(hidden)]
    pub fn claim_for_test(
        self,
    ) -> Result<
        Option<(
            Self,
            WorthQueryOutputClaimIdentity,
            WorthQueryOutputCheckpoint,
        )>,
        WorthQueryOutputDemandDenial,
    > {
        Self::claim(self)
    }
}

impl PreparedSelectedCheckpointFinish<'_> {
    #[doc(hidden)]
    pub fn finish_for_test(
        self,
        claim: WorthQueryOutputClaimIdentity,
        checkpoint: WorthQueryOutputCheckpoint,
        denial: Option<&WorthQueryOutputDemandDenial>,
    ) -> Result<(), CheckpointFinishStop> {
        Self::finish(self, claim, checkpoint, denial)
    }
}
