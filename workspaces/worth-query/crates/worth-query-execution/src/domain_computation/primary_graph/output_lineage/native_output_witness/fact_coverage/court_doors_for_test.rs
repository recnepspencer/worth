//! Sealed test doors over this module's private members.
use super::*;

impl SealedNativeOutputWitness {
    #[doc(hidden)]
    pub fn covers_content_fact_for_test(
        &self,
        fact: &Fact,
    ) -> Result<bool, CompanionPreflightStop> {
        Self::covers_content_fact(self, fact)
    }
}
