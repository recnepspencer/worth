//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn begin_admitted_for_test<'a>(
        &'a self, interest: &'a WorthQueryOutputDemandInterest) -> Result<
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
        &self, interest: &WorthQueryOutputDemandInterest, performed_source: Option<WorthQueryPerformedOutputDemandSource>, result: &mut Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial>) {
        Self::finish_scheduling(self, interest, performed_source, result)
    }
}
