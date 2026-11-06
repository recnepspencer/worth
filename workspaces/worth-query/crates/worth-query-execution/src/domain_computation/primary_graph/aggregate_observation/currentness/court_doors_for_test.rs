//! Sealed test doors over this module's private members.
use super::*;

impl AggregateObservation {
    #[doc(hidden)]
    pub fn remains_equal_for_test(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        control: Option<&dyn Fn() -> Result<(), Cause>>,
    ) -> Result<bool, Failure> {
        Self::remains_equal(self, runtime, execution, snapshot, control)
    }
}

impl AggregateObservation {
    #[doc(hidden)]
    pub fn remains_equal_at_for_test(
        &self,
        pin: &RelationalPinnedRead,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
        control: Option<&dyn Fn() -> Result<(), Cause>>,
    ) -> Result<bool, Failure> {
        Self::remains_equal_at(self, pin, execution, context, control)
    }
}
