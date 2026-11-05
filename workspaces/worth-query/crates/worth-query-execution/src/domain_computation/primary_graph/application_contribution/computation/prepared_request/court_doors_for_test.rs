//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::prepared_request::ComputedSlot;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::prepared_request::PreparedCompute;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::member_execution::WorthQueryMemberExecution;



impl PreparedCompute {
    #[doc(hidden)]
    pub fn new_for_test<T: Any + Send>(
        run: impl for<'a> FnOnce(WorthQueryMemberExecution<'a>) -> T + Send + 'static,
    ) -> Self {
        Self::new(run)
    }
}
