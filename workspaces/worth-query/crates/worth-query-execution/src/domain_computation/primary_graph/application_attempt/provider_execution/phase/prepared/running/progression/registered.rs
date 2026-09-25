use crate::domain_computation::primary_graph::application_attempt::provider_binding::WorthQueryProviderRegistrationInspectionPermit;
use crate::domain_computation::primary_graph::application_attempt::provider_binding::WorthQueryRegisteredProviderAttemptSeal;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::outcome::WorthQueryProviderProgressionOutcome;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;

pub(in crate::domain_computation) struct WorthQueryRegisteredProviderAttempt<'run> {
    staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
    requests: Vec<crate::domain_computation::WorthQueryDecisionFactRequest>,
    steps: std::sync::Arc<[crate::domain_computation::WorthQueryProvisionalEffectStep]>,
    dispatch_outbox:
        Option<crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord>,
}

pub(in crate::domain_computation) struct WorthQueryProviderAttemptRegistrationContext<
    'a,
    Schema,
    Operation,
    Input,
    Scope,
> {
    provider: &'a std::sync::Arc<WorthQueryPrimaryGraphProvider>,
    admission: &'a crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        Scope,
    >,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    aftermath_causality: Option<
        &'a crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    >,
}

impl<'a, Schema, Operation, Input, Scope>
    WorthQueryProviderAttemptRegistrationContext<'a, Schema, Operation, Input, Scope>
{
    pub(super) const fn new(
        provider: &'a std::sync::Arc<WorthQueryPrimaryGraphProvider>,
        admission: &'a crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        aftermath_causality: Option<
            &'a crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
        >,
    ) -> Self {
        Self {
            provider,
            admission,
            idempotency,
            aftermath_causality,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn provider(
        &self,
        _permit: &WorthQueryProviderRegistrationInspectionPermit,
    ) -> &std::sync::Arc<WorthQueryPrimaryGraphProvider> {
        self.provider
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn admission(
        &self,
        _permit: &WorthQueryProviderRegistrationInspectionPermit,
    ) -> &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        Scope,
    > {
        self.admission
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn idempotency(
        &self,
        _permit: &WorthQueryProviderRegistrationInspectionPermit,
    ) -> WorthQueryApplicationIdempotencyBinding {
        self.idempotency
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn aftermath_causality(
        &self,
        _permit: &WorthQueryProviderRegistrationInspectionPermit,
    ) -> Option<
        &crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    > {
        self.aftermath_causality
    }
}

impl<'run> WorthQueryRegisteredProviderAttempt<'run> {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn from_registration(
        _seal: WorthQueryRegisteredProviderAttemptSeal,
        staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
        requests: Vec<crate::domain_computation::WorthQueryDecisionFactRequest>,
        steps: std::sync::Arc<[crate::domain_computation::WorthQueryProvisionalEffectStep]>,
        dispatch_outbox: Option<
            crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord,
        >,
    ) -> Self {
        Self {
            staged,
            requests,
            steps,
            dispatch_outbox,
        }
    }

    pub(super) fn progress<Schema, Operation, Input, Scope>(
        self,
        authority: &super::WorthQueryApplicationCommitProgressionAuthority<
            '_,
            '_,
            Schema,
            Operation,
            Input,
            Scope,
        >,
    ) -> WorthQueryProviderProgressionOutcome
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
        Input: Clone + Send + Sync + 'static,
    {
        let read_set =
            super::fresh::compare_provider_read_set(self.staged, self.requests, authority);
        let fresh = match read_set {
            super::fresh::WorthQueryProviderReadSetProgression::Fresh(fresh) => fresh,
            super::fresh::WorthQueryProviderReadSetProgression::Terminal(outcome) => {
                return outcome
            }
        };
        let candidate = match fresh.progress_invariant(self.steps, authority.provider()) {
            Ok(candidate) => candidate,
            Err(outcome) => return outcome,
        };
        super::authorized::authorize_and_resolve_provider_commit(
            candidate,
            authority,
            self.dispatch_outbox,
        )
    }
}

#[cfg(test)]
#[path = "registered/overlay_conflict_tests.rs"]
mod overlay_conflict_tests;

#[cfg(test)]
pub(super) use overlay_conflict_tests::assert_second_real_overlay_is_rejected;
