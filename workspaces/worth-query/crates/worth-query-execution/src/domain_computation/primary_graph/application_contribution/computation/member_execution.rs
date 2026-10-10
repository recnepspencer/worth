//! The member's bounded execution, without any owner or graph access.
use super::super::{request_execution::MemberRequestExecution, QueryRequestExecution};
use super::{WorthQueryManagedComputationExecution, WorthQueryManagedComputationInterruption};

/// Execution lent only for one prepared member's compute.
///
/// The worker receives no reader, writer, application, provider or phase.
/// Author-held handles remain subject to the worker reentry rule.
///
/// ```compile_fail,E0599
/// use worth_query_execution::facade::application_contribution::WorthQueryMemberExecution;
/// fn cannot_read(member: WorthQueryMemberExecution<'_>) {
///     let _ = member.reader();
/// }
/// ```
///
/// ```
/// use worth_query_execution::facade::application_contribution::WorthQueryMemberExecution;
/// fn can_check(member: WorthQueryMemberExecution<'_>) {
///     let _ = member.interruption();
/// }
/// ```
pub struct WorthQueryMemberExecution<'member> {
    execution: &'member QueryRequestExecution<'member>,
    charge: Option<&'member mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>>,
}
impl<'member> WorthQueryMemberExecution<'member> {
    pub(in crate::domain_computation::primary_graph) fn for_kernel(
        execution: &'member MemberRequestExecution<'member>,
        charge: &'member mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    ) -> Self {
        Self {
            execution: execution.request(),
            charge: Some(charge),
        }
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        WorthQueryManagedComputationExecution<'member>,
        Option<&'member mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>>,
    ) {
        (
            WorthQueryManagedComputationExecution::new(self.execution),
            self.charge,
        )
    }

    pub fn interruption(&self) -> Option<WorthQueryManagedComputationInterruption> {
        self.execution.interruption()
    }

    pub(in crate::domain_computation::primary_graph) fn managed(
        &self,
    ) -> WorthQueryManagedComputationExecution<'_> {
        WorthQueryManagedComputationExecution::new(self.execution)
    }
}
