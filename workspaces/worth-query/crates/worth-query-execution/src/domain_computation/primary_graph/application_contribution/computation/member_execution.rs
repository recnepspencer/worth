//! The member's bounded execution, without any owner or graph access.
use super::super::QueryRequestExecution;
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
}
impl<'member> WorthQueryMemberExecution<'member> {
    pub(in crate::domain_computation::primary_graph) fn new(
        execution: &'member QueryRequestExecution<'member>,
    ) -> Self {
        Self { execution }
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
