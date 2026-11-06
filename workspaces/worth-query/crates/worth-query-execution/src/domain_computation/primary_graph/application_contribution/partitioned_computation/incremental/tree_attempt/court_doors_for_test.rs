//! Sealed test doors over this module's private members.
use super::*;

impl<T> TreeAttempt<T, ReductionRunFailure<worth_execution::MapKernelStop>> {
    #[doc(hidden)]
    pub fn finish_edits_for_test<E>(self) -> TreeAttempt<EditDisposition<T>, Denial<E>> {
        Self::finish_edits::<E>(self)
    }
}

impl<T, E> TreeAttempt<T, E> {
    #[doc(hidden)]
    pub fn map_result_for_test<U, F>(
        self,
        convert: impl FnOnce(Result<T, E>) -> Result<U, F>,
    ) -> TreeAttempt<U, F> {
        Self::map_result(self, convert)
    }
}

impl<T, E> TreeAttempt<T, ReductionRunFailure<E>> {
    #[doc(hidden)]
    pub fn native_for_test(outcome: Result<(T, ReductionMetrics), ReductionRunFailure<E>>) -> Self {
        Self::native(outcome)
    }
}

impl<T, E> TreeAttempt<EditDisposition<T>, Denial<E>> {
    #[doc(hidden)]
    pub fn resolve_for_test(
        self,
        rebuild: impl FnOnce() -> TreeAttempt<T, Denial<E>>,
    ) -> ReportedTree<T, E> {
        Self::resolve(self, rebuild)
    }
}
