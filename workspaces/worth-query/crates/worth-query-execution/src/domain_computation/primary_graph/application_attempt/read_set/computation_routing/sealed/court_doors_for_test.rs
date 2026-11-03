//! Sealed test doors over this module's private members.
use super::*;

impl SealedComputationFacts {
    #[doc(hidden)]
    pub fn facts_for_test(
        &self) -> impl Iterator<
        Item = (
            &WorthQueryApplicationFactKey,
            &WorthQueryApplicationObservedFact,
            &ComputationFactReaders,
        ),
    > {
        Self::facts(self)
    }
}
