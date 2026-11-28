use worth_query_execution_attempt::facade::ports::AdmittedOperationExecutionPort as _;
pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::provider_execution::registered::*;

use crate::domain_computation::primary_graph::application_attempt::provider_execution::outcome::WorthQueryProviderProgressionOutcome;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;

pub trait RegisteredProviderAttemptRuntimeExt<'run>: Sized {
    fn inspect_coupled(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider: &std::sync::Arc<WorthQueryPrimaryGraphProvider>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        candidate: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        own_outputs: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        (
            crate::domain_computation::WorthQueryProposedStateInspection<'run>,
            Option<
                crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord,
            >,
        ),
        WorthQueryProviderProgressionOutcome,
    >;

    fn progress<Schema, Operation, Input, Scope>(
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
        Input: Clone + Send + Sync + 'static;
}

impl<'run> RegisteredProviderAttemptRuntimeExt<'run> for WorthQueryRegisteredProviderAttempt<'run> {
    fn inspect_coupled(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider: &std::sync::Arc<WorthQueryPrimaryGraphProvider>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        candidate: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        own_outputs: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
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
        let (staged, steps, dispatch_outbox) = self.into_parts();
        let control = crate::domain_computation::primary_graph::request_allocation_control::RequestAllocationControl::new(request, allocation_policy);
        let freshness = provider.capture_and_compare_application_read_set(
            execution,
            &staged.read_authority(),
            Some((candidate, own_outputs)),
            crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(control.policy(), Some(request)),
        );
        let fresh = match freshness {
            Ok(crate::domain_computation::WorthQueryDecisionReadSetFreshnessOutcome::Fresh(
                fresh,
            )) => fresh,
            Ok(crate::domain_computation::WorthQueryDecisionReadSetFreshnessOutcome::Stale(
                stale,
            )) => {
                let _ = staged.abort();
                return Err(WorthQueryProviderProgressionOutcome::Stale(
                    crate::domain_computation::primary_graph::WorthQueryApplicationStaleAttempt::new(
                        stale.stale_fact_count(),
                    ),
                ));
            }
            Err(cause) => {
                let _ = staged.abort();
                return Err(
                    crate::domain_computation::primary_graph::application_attempt::provider_compare_denial::read_set_failure(cause),
                );
            }
        };
        super::invariant::begin_invariant_inspection(staged, fresh, steps)
            .map(|inspection| (inspection, dispatch_outbox))
    }

    fn progress<Schema, Operation, Input, Scope>(
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
        let (staged, steps, dispatch_outbox) = self.into_parts();
        let root_session = staged.provider_session_terminal_binding();
        let read_set = super::fresh::compare_provider_read_set(
            execution,
            staged,
            authority,
            allocation_policy,
        );
        let fresh = match read_set {
            super::fresh::WorthQueryProviderReadSetProgression::Fresh(fresh) => fresh,
            super::fresh::WorthQueryProviderReadSetProgression::Terminal(outcome) => {
                return outcome
            }
        };
        let assembly = crate::domain_computation::primary_graph::application_contribution::CoupledApplicationAssembly::new(
            phase,
            authority.coordination(),
            authority.application(),
            authority.outer_application(),
            authority.admission().graph_work().authenticating_principal(),
            authority.product,
            authority.admission().publication_request(),
            authority.coupled_outputs,
            root_session,
        );
        let candidate = match fresh.progress_invariant(
            execution,
            steps,
            authority.provider(),
            authority
                .coupled_outputs
                .is_declared()
                .then_some(&assembly as &dyn crate::domain_computation::primary_graph::provider::CoupledProducerAssembly),
            allocation_policy,
        ) {
            Ok(candidate) => candidate,
            Err(outcome) => {
                assembly.abort_writers(execution);
                let lost = assembly.broke_without_subject();
                return assembly.finish_abort(match assembly.take_denial() {
                    Some(denial) => WorthQueryProviderProgressionOutcome::Denied(
                        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::coupled_producer(denial),
                    ),
                    None if lost => WorthQueryProviderProgressionOutcome::Denied(
                        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::coupled_refusal_without_subject(),
                    ),
                    None => outcome,
                });
            }
        };
        if let Err(denial) = assembly.authorize_members(execution, authority.coordination()) {
            candidate.discard();
            assembly.abort_writers(execution);
            return assembly.finish_abort(
                crate::domain_computation::primary_graph::application_attempt::provider_execution::outcome::progression_from_authorization_denial(
                    denial,
                    crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::DecisionReadSet,
                ),
            );
        }
        let outcome = super::authorized::authorize_and_resolve_provider_commit(
            execution,
            candidate,
            authority,
            dispatch_outbox,
            allocation_policy,
        );
        assembly.settle_writers(execution, outcome)
    }
}
