//! Sealed test doors over this module's private members.
use super::*;

impl SealedComputationFacts {
    #[doc(hidden)]
    pub fn replace_fact_for_test(
        &mut self, key: &WorthQueryApplicationFactKey, fact: WorthQueryApplicationObservedFact) {
        Self::replace_fact(self, key, fact)
    }
}
