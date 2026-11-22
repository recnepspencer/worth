//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn begin_admitted_for_test<'a>(
        &'a self,
        interest: &'a WorthQueryOutputDemandInterest,
    ) -> Result<
        (
            WorthQueryOutputDemandAdvanceAdmission,
            Option<selected_finish::PreparedSelectedSchedulingFinish<'a>>,
        ),
        WorthQueryOutputDemandDenial,
    > {
        Self::begin_admitted(self, interest)
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn finish_scheduling_for_test(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        performed_source: Option<WorthQueryRetainedOutputDemandSource>,
        result: &mut Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial>,
    ) {
        Self::finish_scheduling(self, interest, performed_source, result)
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn finish_checkpoint_for_test(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        claim: WorthQueryOutputClaimIdentity,
        checkpoint: WorthQueryOutputCheckpoint,
        denial: Option<&WorthQueryOutputDemandDenial>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        Self::finish_checkpoint(self, interest, claim, checkpoint, denial)
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn finish_execution_failure_for_test(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        denial: &mut WorthQueryOutputDemandDenial,
    ) {
        Self::finish_execution_failure(self, interest, denial)
    }
}
