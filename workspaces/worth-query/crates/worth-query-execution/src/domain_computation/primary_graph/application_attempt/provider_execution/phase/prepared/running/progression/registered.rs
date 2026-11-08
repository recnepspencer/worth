use crate::domain_computation::primary_graph::application_attempt::provider_binding::WorthQueryProviderRegistrationInspectionPermit;
use crate::domain_computation::primary_graph::application_attempt::provider_binding::WorthQueryRegisteredProviderAttemptSeal;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::outcome::WorthQueryProviderProgressionOutcome;
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
    pub(super) fn new(
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

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn outcome_identity(
        &self,
        _permit: &WorthQueryProviderRegistrationInspectionPermit,
    ) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity{
        self.outcome_identity
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn original_authority(
        &self,
        _permit: &WorthQueryProviderRegistrationInspectionPermit,
    ) -> std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitAuthorityBinding,
    > {
        std::sync::Arc::clone(&self.original_authority)
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

    pub(super) fn inspect_coupled(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider: &std::sync::Arc<WorthQueryPrimaryGraphProvider>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        candidate: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        (
            crate::domain_computation::WorthQueryProposedStateInspection<'run>,
            Option<
                crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord,
            >,
        ),
        WorthQueryProviderProgressionOutcome,
    > {
        let control = crate::domain_computation::primary_graph::request_allocation_control::RequestAllocationControl::new(request, allocation_policy);
        let freshness = provider.capture_and_compare_application_read_set(
            execution, &self.staged.read_authority(), Some(candidate),
            crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(control.policy(), Some(request)),
        );
        let fresh = match freshness {
            Ok(crate::domain_computation::WorthQueryDecisionReadSetFreshnessOutcome::Fresh(
                fresh,
            )) => fresh,
            Ok(crate::domain_computation::WorthQueryDecisionReadSetFreshnessOutcome::Stale(
                stale,
            )) => {
                let _ = self.staged.abort();
                return Err(WorthQueryProviderProgressionOutcome::Stale(crate::domain_computation::primary_graph::WorthQueryApplicationStaleAttempt::new(stale.stale_fact_count())));
            }
            Err(cause) => {
                let _ = self.staged.abort();
                return Err(crate::domain_computation::primary_graph::application_attempt::provider_compare_denial::read_set_failure(cause));
            }
        };
        super::invariant::begin_invariant_inspection(self.staged, fresh, self.steps)
            .map(|inspection| (inspection, self.dispatch_outbox))
    }

    pub(super) fn progress<Schema, Operation, Input, Scope>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &'run crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        authority: &super::WorthQueryApplicationCommitProgressionAuthority<
            '_,
            '_,
            Schema,
            Operation,
            Input,
            Scope,
        >,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryProviderProgressionOutcome
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
        Input: Clone + Send + Sync + 'static,
    {
        let root_session = self.staged.provider_session_terminal_binding();
        let read_set = super::fresh::compare_provider_read_set(
            execution,
            self.staged,
            authority,
            allocation_policy,
        );
        let fresh = match read_set {
            super::fresh::WorthQueryProviderReadSetProgression::Fresh(fresh) => fresh,
            super::fresh::WorthQueryProviderReadSetProgression::Terminal(outcome) => {
                return outcome
            }
        };
        let assembly = crate::domain_computation::primary_graph::application_contribution::DeclaredProducerAssembly::new(
            phase, authority.coordination(), authority.application(), authority.admission().graph_work().authenticating_principal(),
            authority.product, authority.admission().publication_request(), authority.coupled_outputs, root_session,
        );
        let candidate = match fresh.progress_invariant(
            execution,
            self.steps,
            authority.provider(),
            authority.coupled_outputs.is_declared().then_some(&assembly as &dyn crate::domain_computation::primary_graph::provider::CoupledProducerAssembly),
            allocation_policy,
        ) {
            Ok(candidate) => candidate,
            Err(outcome) => {
                assembly.abort_writers(execution);
                return assembly.finish_abort(match assembly.take_denial() {
                    Some(denial) => WorthQueryProviderProgressionOutcome::Denied(crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::coupled_producer(denial)),
                    None => outcome,
                });
            },
        };
        if let Err(denial) = assembly.authorize_members(execution, authority.coordination()) {
            candidate.discard();
            assembly.abort_writers(execution);
            return assembly.finish_abort(crate::domain_computation::primary_graph::application_attempt::provider_execution::outcome::progression_from_authorization_denial(
                denial, crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::DecisionReadSet,
            ));
        }
        let outcome = super::authorized::authorize_and_resolve_provider_commit(
            execution,
            candidate,
            authority,
            self.dispatch_outbox,
            allocation_policy,
        );
        assembly.settle_writers(execution, outcome)
    }
}

mod court_doors_for_test;
