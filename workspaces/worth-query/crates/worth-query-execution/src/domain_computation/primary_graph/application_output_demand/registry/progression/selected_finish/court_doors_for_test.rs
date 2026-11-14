//! Sealed test doors over this module's private members.
use super::*;

impl<'a> PreparedSelectedSchedulingFinish<'a> {
    #[doc(hidden)]
    pub fn finish_for_test(
        self,
        performed_source: Option<WorthQueryRetainedOutputDemandSource>,
        result: &mut Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial>,
    ) {
        Self::finish(self, performed_source, result)
    }
}
