//! The member's bounded execution, without any owner or graph access.
use super::super::{request_execution::MemberRequestExecution, QueryRequestExecution};
use super::nested_failure::NestedFailureTransport;
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
    take: &'member mut dyn FnMut() -> Option<worth_execution::NestedComputationStop>,
    poll: &'member mut dyn FnMut() -> Result<(), worth_execution::MapKernelStop>,
    charge: Option<&'member mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>>,
}
impl<'member> WorthQueryMemberExecution<'member> {
    pub(in crate::domain_computation::primary_graph) fn for_kernel(
        execution: &'member MemberRequestExecution<'member>,
        poll: &'member mut dyn FnMut() -> Result<(), worth_execution::MapKernelStop>,
        charge: Option<&'member mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>>,
        take: &'member mut dyn FnMut() -> Option<worth_execution::NestedComputationStop>,
    ) -> Self {
        Self {
            execution: execution.request(),
            poll,
            charge,
            take,
        }
    }

    pub(super) fn into_checkpoint(self) -> super::WorthQueryManagedComputationCheckpoint<'member> {
        super::WorthQueryManagedComputationCheckpoint {
            charge: self.charge,
            poll: self.poll,
            refused: None,
            #[cfg(test)]
            native: None,
        }
    }

    /// An ordinary returned error acknowledges nothing. Only a nested call's
    /// transported failure consumes its own computational cause.
    pub(in crate::domain_computation::primary_graph) fn compute<T, E>(
        self,
        run: impl FnOnce(
            WorthQueryManagedComputationExecution<'member>,
            &'member mut dyn FnMut() -> Result<(), worth_execution::MapKernelStop>,
            &mut NestedFailureTransport<'member>,
        ) -> Result<T, E>,
    ) -> Result<T, E> {
        let Self {
            execution,
            poll,
            take,
            charge: _,
        } = self;
        run(
            WorthQueryManagedComputationExecution::new(execution),
            poll,
            &mut NestedFailureTransport::new(take),
        )
    }

    #[cfg(test)]
    pub(super) fn into_parts(
        self,
    ) -> (
        WorthQueryManagedComputationExecution<'member>,
        &'member mut dyn FnMut() -> Result<(), worth_execution::MapKernelStop>,
    ) {
        (
            WorthQueryManagedComputationExecution::new(self.execution),
            self.poll,
        )
    }

    pub fn interruption(&self) -> Option<WorthQueryManagedComputationInterruption> {
        self.execution.interruption()
    }
}
