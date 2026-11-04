//! Sealed test doors over this module's private members.
use super::*;

impl SealedComputationFacts {
    #[doc(hidden)]
    pub fn facts_for_test(
        &self,
    ) -> impl Iterator<
        Item = (
            &WorthQueryApplicationFactKey,
            &WorthQueryApplicationObservedFact,
            &ComputationFactReaders,
        ),
    > {
        Self::facts(self)
    }
}

impl SealedComputationFacts {
    #[doc(hidden)]
    pub fn at_ordinal_for_test(
        &self,
        ordinal: usize,
    ) -> Option<(
        &WorthQueryApplicationFactKey,
        &WorthQueryApplicationObservedFact,
        &ComputationFactReaders,
    )> {
        Self::at_ordinal(self, ordinal)
    }
}

impl PreparedComputationFacts {
    #[doc(hidden)]
    pub fn bind_fixture_for_test(self, original: FactPayload) -> SealedComputationFacts {
        Self::bind_fixture(self, original)
    }
}
