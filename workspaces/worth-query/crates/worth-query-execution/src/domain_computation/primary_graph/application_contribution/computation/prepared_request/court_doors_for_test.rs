//! Sealed test doors over this module's private members.
use super::*;

impl PreparedCompute {
    #[doc(hidden)]
    pub fn declared_test_request_for_test(rank: usize, fails: bool) -> Self {
        Self::declared_test_request(rank, fails)
    }
}

impl ComputedSlot {
    #[doc(hidden)]
    pub fn into_value_for_test<T: Any + Send>(self) -> T {
        Self::into_value::<T>(self)
    }
}

impl PreparedCompute {
    #[doc(hidden)]
    pub fn new_for_test<T: Any + Send>(
        run: impl for<'a> FnOnce(WorthQueryMemberExecution<'a>) -> T + Send + 'static,
    ) -> Self {
        Self::new(run)
    }
}
