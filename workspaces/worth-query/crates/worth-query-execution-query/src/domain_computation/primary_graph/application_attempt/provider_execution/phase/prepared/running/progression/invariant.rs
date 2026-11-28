use crate::domain_computation::primary_graph as graph;
use crate::domain_computation::{
    check_invariant_request_live, WorthQueryFreshDecisionReadSet, WorthQueryProvisionalEffectStep,
    WorthQuerySessionBoundReadsAndEffects, WorthQueryStagedProviderCandidate,
    WorthQueryValidatedProviderCandidate,
};
use graph::application_attempt::provider_execution::outcome::{
    progression_denied, WorthQueryProviderProgressionOutcome,
};
use graph::application_attempt::WorthQueryApplicationCommitDenialStage as DenialStage;
use graph::provider::{CoupledProducerAssembly, WorthQueryPrimaryGraphProvider};
use graph::WorthQueryApplicationCommitDenial as Denial;
use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

pub fn progress_invariant_candidate<'run>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    staged: WorthQuerySessionBoundReadsAndEffects<'run>,
    fresh: WorthQueryFreshDecisionReadSet,
    request: &WorthQueryRequestScope,
    steps: Arc<[WorthQueryProvisionalEffectStep]>,
    provider: &WorthQueryPrimaryGraphProvider,
    assembly: Option<&dyn CoupledProducerAssembly>,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<WorthQueryValidatedProviderCandidate<'run>, WorthQueryProviderProgressionOutcome> {
    let candidate = stage_commit_candidate(provider, staged, fresh, steps)?;
    let validation = check_invariant_request_live(Some(request)).and_then(|()| match assembly {
        Some(assembly) => provider.admit_primary_candidate_coupled(
            execution,
            candidate.provider_session_view(),
            assembly,
            allocation_policy,
        ),
        None => provider.admit_primary_candidate(
            execution,
            candidate.provider_session_view(),
            allocation_policy,
        ),
    });
    if let Err(failure) = validation {
        candidate.discard();
        return Err(WorthQueryProviderProgressionOutcome::Denied(
            Denial::invariant_execution_denied(DenialStage::InvariantExecution, failure),
        ));
    }
    bind_validated_candidate(provider, candidate, request)
}

pub(super) fn bind_validated_candidate<'run>(
    provider: &WorthQueryPrimaryGraphProvider,
    candidate: WorthQueryStagedProviderCandidate<'run>,
    request: &WorthQueryRequestScope,
) -> Result<WorthQueryValidatedProviderCandidate<'run>, WorthQueryProviderProgressionOutcome> {
    if let Err(failure) = check_invariant_request_live(Some(request)) {
        candidate.discard();
        return Err(WorthQueryProviderProgressionOutcome::Denied(
            Denial::invariant_execution_denied(DenialStage::InvariantExecution, failure),
        ));
    }
    provider
        .bind_validated_candidate(candidate)
        .map_err(|candidate| {
            candidate.discard();
            progression_denied(DenialStage::InvariantExecution)
        })
}

pub(super) fn stage_commit_candidate<'run>(
    provider: &WorthQueryPrimaryGraphProvider,
    staged: WorthQuerySessionBoundReadsAndEffects<'run>,
    fresh: WorthQueryFreshDecisionReadSet,
    steps: Arc<[WorthQueryProvisionalEffectStep]>,
) -> Result<WorthQueryStagedProviderCandidate<'run>, WorthQueryProviderProgressionOutcome> {
    let lowered = match staged
        .effect_authority()
        .lower_shared_provisional_program(&fresh, steps)
    {
        Ok(lowered) => lowered,
        Err(failure) => {
            let _ = staged.abort();
            return Err(WorthQueryProviderProgressionOutcome::Denied(
                Denial::provider_rejected_with_detail(
                    DenialStage::EffectLowering,
                    failure.detail().to_owned(),
                ),
            ));
        }
    };
    provider
        .stage_commit_candidate(staged, fresh, lowered)
        .map_err(|_| progression_denied(DenialStage::ProvisionalState))
}
