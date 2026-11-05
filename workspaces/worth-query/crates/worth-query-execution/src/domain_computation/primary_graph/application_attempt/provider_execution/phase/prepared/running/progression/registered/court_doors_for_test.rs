//! Sealed test doors over this module's private members.
use super::*;

impl<'run> WorthQueryRegisteredProviderAttempt<'run> {
    #[doc(hidden)]
    pub fn into_parts_for_test(
        self,
    ) -> (
        crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
        std::sync::Arc<[crate::domain_computation::WorthQueryProvisionalEffectStep]>,
        Option<crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryDispatchOutboxRecord>,
    ){
        (self.staged, self.steps, self.dispatch_outbox)
    }
}

impl<'run> WorthQueryRegisteredProviderAttempt<'run> {
    #[doc(hidden)]
    pub fn staged_for_test(
        &self,
    ) -> &crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run> {
        &self.staged
    }
}

impl<'run> WorthQueryRegisteredProviderAttempt<'run> {
    #[doc(hidden)]
    pub fn staged_into_for_test(
        self,
    ) -> crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run> {
        self.staged
    }
}

impl<'a, Schema, Operation, Input, Scope>
    WorthQueryProviderAttemptRegistrationContext<'a, Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn new_for_test(
        provider: &'a std::sync::Arc<WorthQueryPrimaryGraphProvider>,
        admission: &'a crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        outcome_identity: crate::domain_computation::primary_graph::vocabulary::application_attempt::commit_outcome_identity::WorthQueryApplicationCommitOutcomeIdentity,
        aftermath_causality: Option<
            &'a crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
        >,
    ) -> Self
    where
        Input: Clone + Send + Sync + 'static,
    {
        Self::new(
            provider,
            admission,
            idempotency,
            outcome_identity,
            aftermath_causality,
        )
    }
}
