//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn assert_cancelled_retained_activation_read_for_test(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        source: &CancellationSource,
    ) {
        Self::assert_cancelled_retained_activation_read(self, runtime, execution, snapshot, source)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn assert_fact_dispatch_stop_oracle_for_test(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
    ) {
        Self::assert_fact_dispatch_stop_oracle(self, runtime, execution, snapshot)
    }
}
