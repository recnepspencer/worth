//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Operation, Input, Scope>
    WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn lease_for_test(&self) -> &WorthQueryApplicationSnapshotLease {
        &self.lease
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn mutation_run_for_test(
        &self,
    ) -> &crate::domain_computation::provider_session::WorthQueryMutationRunBinding {
        &self.mutation_run
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn running_for_test(&self) -> &crate::domain_computation::WorthQueryRunningDirectRun {
        &self.running
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn running_mut_for_test(
        &mut self,
    ) -> &mut crate::domain_computation::WorthQueryRunningDirectRun {
        &mut self.running
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn mutation_run_into_for_test(
        self,
    ) -> crate::domain_computation::provider_session::WorthQueryMutationRunBinding {
        self.mutation_run
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn running_into_for_test(self) -> crate::domain_computation::WorthQueryRunningDirectRun {
        self.running
    }
}

