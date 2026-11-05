//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::nested_failure::NestedFailureTransport;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::checkpoint::WorthQueryManagedComputationExecution;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::member_execution::WorthQueryMemberExecution;


impl<'member> WorthQueryMemberExecution<'member> {
    #[doc(hidden)]
    pub fn into_parts_for_test(
        self,
    ) -> (
        WorthQueryManagedComputationExecution<'member>,
        &'member mut dyn FnMut() -> Result<(), worth_execution::MapKernelStop>,
    ) {
        Self::into_parts(self)
    }
}
