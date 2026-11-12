use crate::domain_computation::primary_graph as graph;
use crate::domain_computation::{
    WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryFreshDecisionReadSet,
    WorthQuerySessionBoundReadsAndEffects,
};
use graph::application_attempt::provider_compare_denial::{
    provider_session_execution_denied, read_set_failure,
};
use graph::application_attempt::provider_execution::outcome::{
    progression_denied, WorthQueryProviderProgressionOutcome as Progression,
};
use graph::application_attempt::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationStaleAttempt,
};
use graph::provider::WorthQueryProviderIdempotencyResolutionDenial as IdempotencyDenial;
use graph::provider::{WorthQueryPrimaryGraphProvider, WorthQueryProviderIdempotencyResolution};
use graph::WorthQueryApplicationCommitDenialStage as ExecutionDenialStage;
use worth_query_installation::facade::ApplicationSchema;

use graph::application_attempt::WorthQueryApplicationCommitDenialStage as DenialStage;

pub(super) enum WorthQueryProviderReadSetProgression<'run> {
    Fresh(WorthQueryFreshProviderAttempt<'run>),
    Terminal(Progression),
}

pub(super) struct WorthQueryFreshProviderAttempt<'run> {
    staged: WorthQuerySessionBoundReadsAndEffects<'run>,
    read_set: WorthQueryFreshDecisionReadSet,
    request: worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
}

impl<'run> WorthQueryFreshProviderAttempt<'run> {
    pub(super) fn progress_invariant(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        steps: std::sync::Arc<[crate::domain_computation::WorthQueryProvisionalEffectStep]>,
        provider: &std::sync::Arc<WorthQueryPrimaryGraphProvider>,
        assembly: Option<
            &dyn crate::domain_computation::primary_graph::provider::CoupledProducerAssembly,
        >,
        history: Option<super::HistoryProducerInvocation<'_>>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        crate::domain_computation::WorthQueryInvariantApprovedProposedState<'run>,
        Progression,
    > {
        super::invariant::progress_invariant_candidate(
            execution,
            self.staged,
            self.read_set,
            &self.request,
            steps,
            provider,
            assembly,
            history,
            allocation_policy,
        )
    }
}

pub(super) fn compare_provider_read_set<'run, Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    staged: WorthQuerySessionBoundReadsAndEffects<'run>,

    authority: &super::WorthQueryApplicationCommitProgressionAuthority<
        '_,
        '_,
        Schema,
        Operation,
        Input,
        Scope,
    >,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> WorthQueryProviderReadSetProgression<'run>
where
    Schema: ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    if let Some(outcome) = registered_idempotency_outcome(execution, &staged, authority) {
        let _ = staged.abort();
        return WorthQueryProviderReadSetProgression::Terminal(outcome);
    }
    let allocation_control = crate::domain_computation::primary_graph::request_allocation_control::RequestAllocationControl::new(
        authority.admission().publication_request(),
        allocation_policy,
    );
    match authority.provider().capture_and_compare_application_read_set(execution,
        &staged.read_authority(), None,
        crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(
            allocation_control.policy(), Some(authority.admission().publication_request()),
        ),
    ) {
        Ok(WorthQueryDecisionReadSetFreshnessOutcome::Fresh(read_set)) => {
            WorthQueryProviderReadSetProgression::Fresh(WorthQueryFreshProviderAttempt {
                staged,
                read_set,
                request: authority.admission().publication_request().clone(),
            })
        }
        Ok(WorthQueryDecisionReadSetFreshnessOutcome::Stale(stale)) => {
            resolve_stale_provider_read_set(execution, staged, stale.stale_fact_count(), authority)
        }
        Err(failure) => {
            let _ = staged.abort();
            WorthQueryProviderReadSetProgression::Terminal(read_set_failure(failure))
        }
    }
}

fn resolve_stale_provider_read_set<'run, Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    staged: WorthQuerySessionBoundReadsAndEffects<'run>,
    stale_fact_count: usize,
    authority: &super::WorthQueryApplicationCommitProgressionAuthority<
        '_,
        '_,
        Schema,
        Operation,
        Input,
        Scope,
    >,
) -> WorthQueryProviderReadSetProgression<'run>
where
    Schema: ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    let outcome =
        registered_idempotency_outcome(execution, &staged, authority).unwrap_or_else(|| {
            Progression::Stale(WorthQueryApplicationStaleAttempt::new(stale_fact_count))
        });
    let _ = staged.abort();
    WorthQueryProviderReadSetProgression::Terminal(outcome)
}

fn registered_idempotency_outcome<Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    staged: &WorthQuerySessionBoundReadsAndEffects<'_>,
    authority: &super::WorthQueryApplicationCommitProgressionAuthority<
        '_,
        '_,
        Schema,
        Operation,
        Input,
        Scope,
    >,
) -> Option<Progression>
where
    Schema: ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    let proof = match authority.authorization.authorize_application_commit(execution,
        authority.application(),
        authority.admission(),
        authority.coordination(),
    ) {
        Ok(proof) => proof,
        Err(denial) => {
            return Some(
                graph::application_attempt::provider_execution::outcome::
                    progression_from_authorization_denial(
                    denial,
                    DenialStage::DecisionReadSet,
                ),
            )
        }
    };
    let provider_session = staged.provider_session_terminal_binding();
    let resolution = proof.govern((), |()| {
        authority
            .provider()
            .resolve_application_idempotency(execution, &provider_session)
    });
    match resolution {
        Ok(Ok(WorthQueryProviderIdempotencyResolution::ReleasedOriginal(_))) => Some(Progression::Denied(WorthQueryApplicationCommitDenial::original_effect_released())),
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Readmitted(receipt))) => Some(if graph::application_attempt::provider_execution::aftermath_resolution::recovered_answers_aftermath(authority.aftermath_causality(), &receipt) {
            Progression::RecoveredCommitted(receipt)
        } else { Progression::Denied(WorthQueryApplicationCommitDenial::idempotency_intent_drift()) }),
        Err(((), denial)) => Some(
            graph::application_attempt::provider_execution::outcome::
                progression_from_authorization_denial(
                denial,
                DenialStage::DecisionReadSet,
            ),
        ),
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Drift)) => {
            Some(Progression::Denied(
                WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
            ))
        }
        Ok(Ok(WorthQueryProviderIdempotencyResolution::PreparationPending { handle, stage })) => Some(Progression::ProductPreparationPending(authority.application().runtime.primary_provider.preparation_pending(&authority.product.publication_binding(), handle, stage))),
        Ok(Ok(WorthQueryProviderIdempotencyResolution::RecoveredUnpublished { handle })) => Some(
            match authority.application().runtime.primary_provider.recovered_unpublished_application(&authority.product.publication_binding(), handle) {
                Ok(effects) => Progression::ProductUnpublished(effects),
                Err(_) => Progression::Denied(WorthQueryApplicationCommitDenial::idempotency_preparation_busy()),
            }
        ),
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Unpublished)) => {
            Some(progression_denied(DenialStage::Idempotency))
        }
        Ok(Ok(WorthQueryProviderIdempotencyResolution::PreparationBusy)) | Ok(Err(IdempotencyDenial::PreparationBusy)) => Some(Progression::Denied(WorthQueryApplicationCommitDenial::idempotency_preparation_busy())),
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Absent)) => None,
        Ok(Err(IdempotencyDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        })) => Some(Progression::Denied(
            WorthQueryApplicationCommitDenial::active_snapshot_capacity_exhausted(
                DenialStage::Idempotency,
                maximum_active_snapshots,
            ),
        )),
        Ok(Err(IdempotencyDenial::ExecutionDenied(kind))) => {
            Some(Progression::Denied(
                provider_session_execution_denied(
                    kind,
                    ExecutionDenialStage::Idempotency,
                    "pending publication execution refused",
                ),
            ))
        }
        Ok(Err(IdempotencyDenial::Unavailable)) => {
            Some(progression_denied(DenialStage::Idempotency))
        }
        Ok(Err(IdempotencyDenial::ForeignAffinity)) => {
            Some(Progression::Denied(
                WorthQueryApplicationCommitDenial::idempotency_foreign_affinity(),
            ))
        }
        Ok(Err(IdempotencyDenial::RecordedIntentUnverifiable)) => {
            Some(Progression::Denied(
                WorthQueryApplicationCommitDenial::idempotency_intent_unverifiable(),
            ))
        }
        Ok(Err(IdempotencyDenial::RetentionCapacityExhausted)) => {
            Some(Progression::Denied(
                WorthQueryApplicationCommitDenial::retention_capacity_exhausted(
                    DenialStage::Idempotency,
                ),
            ))
        }
        Ok(Err(IdempotencyDenial::RetentionIdentityExhausted)) => {
            Some(Progression::Denied(
                WorthQueryApplicationCommitDenial::retention_identity_exhausted(
                    DenialStage::Idempotency,
                ),
            ))
        }
        Ok(Err(IdempotencyDenial::SnapshotIdentityExhausted)) => {
            Some(Progression::Denied(
                WorthQueryApplicationCommitDenial::snapshot_identity_exhausted(
                    DenialStage::Idempotency,
                ),
            ))
        }
    }
}
