//! Sealed test doors over this module's private members.
use super::*;

impl SelectedRequiredWork {
    #[doc(hidden)]
    pub fn acknowledge_current_admitted_for_test<P: crate::ports::CurrentProofPort>(
        self,
        proof: P,
    ) -> Result<Option<ReplacedRequiredWorkHint>, WorthQueryOutputDemandDenial> {
        Self::acknowledge_current_admitted::<P>(self, proof)
    }
}
