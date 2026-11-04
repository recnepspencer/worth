//! Sealed test doors over this module's private members.
use super::*;

impl<Member> AggregateObservation<Member> {
    #[doc(hidden)]
    pub fn key_for_test(&self) -> &AggregateKey<Member> {
        Self::key(self)
    }
}

impl AggregateObservation {
    #[doc(hidden)]
    pub fn read_for_test(
        pin: &RelationalPinnedRead,
        execution: &ExecutionRequest<'_, '_>,
        query: AggregateQuery<'_>,
    ) -> Result<Arc<Self>, PinnedAggregateReadDenial> {
        Self::read(pin, execution, query)
    }
}

impl AggregateObservation {
    #[doc(hidden)]
    pub fn same_truth_with_poll_for_test<E>(
        &self,
        other: &Self,
        poll: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        Self::same_truth_with_poll(self, other, poll)
    }
}

impl<Member> AggregateObservation<Member> {
    #[doc(hidden)]
    pub fn source_for_test(&self) -> &SnapshotHandle {
        Self::source(self)
    }
}
