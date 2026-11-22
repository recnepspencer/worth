//! Sealed test doors over this module's private members.
use super::*;

impl ComparableSourceFacts {
    #[doc(hidden)]
    pub fn facts_for_test(&self) -> &FactPayload {
        Self::facts(self)
    }
}

impl RetainedSourceFacts {
    #[doc(hidden)]
    pub fn len_for_test(&self) -> usize {
        Self::len(self)
    }
}

impl RetainedSourceFacts {
    #[doc(hidden)]
    pub fn for_test_for_test(stale: bool, facts: Arc<[Fact]>) -> Self {
        Self::for_test(stale, facts)
    }
}

impl RetainedSourceFacts {
    #[doc(hidden)]
    pub fn with_posting_continuation_for_test(
        self,
        continuation: Option<super::super::invalidation::PostingContinuation>,
    ) -> Self {
        Self::with_posting_continuation(self, continuation)
    }
}

impl RetainedSourceFacts {
    #[doc(hidden)]
    pub fn postconditions_for_test(&self) -> &FactPayload {
        Self::postconditions(self)
    }
}
