//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn assert_cancelled_fact_dispatch_for_test(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        source: &CancellationSource,
    ) {
        Self::assert_cancelled_fact_dispatch(self, runtime, execution, snapshot, source)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn assert_retained_activation_read_at_every_consult_for_test(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        source: &CancellationSource,
    ) {
        Self::assert_retained_activation_read_at_every_consult(
            self, runtime, execution, snapshot, source,
        )
    }
}
