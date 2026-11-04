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

impl<'run> WorthQueryRegisteredProviderSession<'run> {
    #[doc(hidden)]
    pub fn registered_for_test(&self) -> &super::super::WorthQueryRegisteredProviderAttempt<'run> {
        &self.registered
    }
}

impl WorthQueryProviderSessionRegistrationFailure {
    #[doc(hidden)]
    pub fn into_completion_for_test(self) -> super::super::WorthQueryProviderProgressionCompletion {
        Self::into_completion(self)
    }
}

impl<'run> WorthQueryRegisteredProviderSession<'run> {
    #[doc(hidden)]
    pub fn progress_for_test<Schema, Operation, Input, Scope>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: &super::super::WorthQueryApplicationCommitProgressionAuthority<
            '_,
            '_,
            Schema,
            Operation,
            Input,
            Scope,
        >,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> super::super::WorthQueryProviderProgressionCompletion
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
        Input: Clone + Send + Sync + 'static,
    {
        Self::progress::<Schema, Operation, Input, Scope>(
            self,
            execution,
            authority,
            allocation_policy,
        )
    }
}

impl<'run> WorthQueryRegisteredProviderSession<'run> {
    #[doc(hidden)]
    pub fn registered_into_for_test(
        self,
    ) -> super::super::WorthQueryRegisteredProviderAttempt<'run> {
        self.registered
    }
}
