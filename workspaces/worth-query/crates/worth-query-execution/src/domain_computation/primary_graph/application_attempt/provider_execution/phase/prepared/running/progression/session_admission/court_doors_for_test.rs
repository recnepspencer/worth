//! Sealed test doors over this module's private members.
use super::*;

impl<'run> WorthQueryAdmittedProviderSession<'run> {
    #[doc(hidden)]
    pub fn staged_for_test(
        &self,
    ) -> &crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run> {
        &self.staged
    }
}

impl<'run> WorthQueryAdmittedProviderSession<'run> {
    #[doc(hidden)]
    pub fn register_for_test<Schema, Operation, Input, Scope>(
        self,
        authorization: &mut crate::domain_computation::authorization::WorthQueryProviderCommitAuthorization,
        prepared: super::super::WorthQueryPreparedApplicationProviderAttempt,
        attempt_basis: super::super::WorthQueryApplicationAttemptBasis,
        context: super::super::WorthQueryProviderAttemptRegistrationContext<
            '_,
            Schema,
            Operation,
            Input,
            Scope,
        >,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryRegisteredProviderSession<'run>,
        WorthQueryProviderSessionRegistrationFailure,
    > {
        Self::register::<Schema, Operation, Input, Scope>(
            self,
            authorization,
            prepared,
            attempt_basis,
            context,
            allocation_policy,
        )
    }
}
