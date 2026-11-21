use crate::domain_computation::primary_graph::CommitAuthorityBindingExt;
use crate::domain_computation::primary_graph::application_attempt::provider_binding::WorthQueryProviderRegistrationInspectionPermit;
use crate::domain_computation::primary_graph::application_attempt::provider_binding::WorthQueryRegisteredProviderAttemptSeal;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;

pub struct WorthQueryRegisteredProviderAttempt<'run> {
    staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,

    steps: std::sync::Arc<[crate::domain_computation::WorthQueryProvisionalEffectStep]>,
    dispatch_outbox:
        Option<crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord>,
}

pub struct WorthQueryProviderAttemptRegistrationContext<
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
    original_authority: std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitAuthorityBinding,
    >,
    outcome_identity: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
    aftermath_causality: Option<
        &'a crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    >,
}

impl<'a, Schema, Operation, Input, Scope>
    WorthQueryProviderAttemptRegistrationContext<'a, Schema, Operation, Input, Scope>
{
    pub fn new(
        provider: &'a std::sync::Arc<WorthQueryPrimaryGraphProvider>,
        admission: &'a crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        outcome_identity: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
        aftermath_causality: Option<
            &'a crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
        >,
    ) -> Self
    where
        Input: Clone + Send + Sync + 'static,
    {
        Self {
            provider,
            admission,
            idempotency,
            original_authority: std::sync::Arc::new(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitAuthorityBinding::from_admission(
                    admission, idempotency,
                ),
            ),
            outcome_identity,
            aftermath_causality,
        }
    }

    pub const fn provider(
        &self,
        _permit: &WorthQueryProviderRegistrationInspectionPermit,
    ) -> &std::sync::Arc<WorthQueryPrimaryGraphProvider> {
        self.provider
    }

    pub const fn admission(
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

    pub const fn idempotency(
        &self,
        _permit: &WorthQueryProviderRegistrationInspectionPermit,
    ) -> WorthQueryApplicationIdempotencyBinding {
        self.idempotency
    }

    pub const fn outcome_identity(
        &self,
        _permit: &WorthQueryProviderRegistrationInspectionPermit,
    ) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity{
        self.outcome_identity
    }

    pub fn original_authority(
        &self,
        _permit: &WorthQueryProviderRegistrationInspectionPermit,
    ) -> std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitAuthorityBinding,
    > {
        std::sync::Arc::clone(&self.original_authority)
    }

    pub const fn aftermath_causality(
        &self,
        _permit: &WorthQueryProviderRegistrationInspectionPermit,
    ) -> Option<
        &crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    > {
        self.aftermath_causality
    }
}

impl<'run> WorthQueryRegisteredProviderAttempt<'run> {
    pub fn from_registration(
        _seal: WorthQueryRegisteredProviderAttemptSeal,
        staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,

        steps: std::sync::Arc<[crate::domain_computation::WorthQueryProvisionalEffectStep]>,
        dispatch_outbox: Option<
            crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord,
        >,
    ) -> Self {
        Self {
            staged,

            steps,
            dispatch_outbox,
        }
    }
    pub fn into_parts(
        self,
    ) -> (
        crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
        std::sync::Arc<[crate::domain_computation::WorthQueryProvisionalEffectStep]>,
        Option<crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord>,
    ) {
        (self.staged, self.steps, self.dispatch_outbox)
    }

    pub fn staged_ref(&self) -> &crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run> {
        &self.staged
    }

    pub fn steps_ref(&self) -> &std::sync::Arc<[crate::domain_computation::WorthQueryProvisionalEffectStep]> {
        &self.steps
    }

    pub fn dispatch_outbox_ref(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord> {
        self.dispatch_outbox.as_ref()
    }
}

mod court_doors_for_test;
