use crate::domain_computation::primary_graph as graph;
use crate::domain_computation::WorthQueryInvariantStateLocator;
use graph::application_attempt::provider_execution::outcome::{
    progression_denied, WorthQueryProviderProgressionOutcome,
};
use graph::application_attempt::WorthQueryApplicationCommitDenialStage as DenialStage;
use graph::WorthQueryApplicationCommitDenial as Denial;

pub(super) fn progress_invariant_candidate<'run>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
    fresh: crate::domain_computation::WorthQueryFreshDecisionReadSet,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    steps: std::sync::Arc<[crate::domain_computation::WorthQueryProvisionalEffectStep]>,
    provider: &std::sync::Arc<
        crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
    >,
    assembly: Option<
        &dyn crate::domain_computation::primary_graph::provider::CoupledProducerAssembly,
    >,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    crate::domain_computation::WorthQueryInvariantApprovedProposedState<'run>,
    WorthQueryProviderProgressionOutcome,
> {
    let inspection = begin_invariant_inspection(staged, fresh, steps)?;
    let locators = invariant_locators(&inspection, request);
    let candidate = locators
        .as_ref()
        .map(|_| ())
        .map_err(Clone::clone)
        .and_then(|()| {
            crate::domain_computation::provider_session::check_invariant_request_live(Some(request))
        })
        .and_then(|()| match assembly {
            Some(assembly) => provider.admit_primary_candidate_coupled(
                execution,
                inspection.provider_session_view(),
                assembly,
                allocation_policy,
            ),
            None => provider.admit_primary_candidate(
                execution,
                inspection.provider_session_view(),
                allocation_policy,
            ),
        });
    match candidate {
        Ok(()) => (),
        Err(failure) => {
            inspection.discard();
            return Err(WorthQueryProviderProgressionOutcome::Denied(
                crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::invariant_execution_denied(
                    DenialStage::InvariantExecution,
                    failure,
                ),
            ));
        }
    };
    finish_invariant_inspection(inspection, request, locators)
}

pub(super) fn complete_coupled_invariants<'run>(
    inspection: crate::domain_computation::WorthQueryProposedStateInspection<'run>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<
    crate::domain_computation::WorthQueryInvariantApprovedProposedState<'run>,
    WorthQueryProviderProgressionOutcome,
> {
    let locators = invariant_locators(&inspection, request);
    finish_invariant_inspection(inspection, request, locators)
}

fn finish_invariant_inspection<'run>(
    inspection: crate::domain_computation::WorthQueryProposedStateInspection<'run>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    locators: Result<
        Vec<WorthQueryInvariantStateLocator>,
        crate::domain_computation::WorthQueryInvariantExecutionFailure,
    >,
) -> Result<
    crate::domain_computation::WorthQueryInvariantApprovedProposedState<'run>,
    WorthQueryProviderProgressionOutcome,
> {
    let receipts = match locators.and_then(|locators| {
        let slots =
                inspection
                    .installed_invariant_requirements()
                    .iter()
                    .map(|requirement| {
                        crate::domain_computation::provider_session::check_invariant_request_live(
                            Some(request),
                        )?;
                        Ok(requirement.slot().to_owned())
                    })
                    .collect::<Result<
                        Vec<_>,
                        crate::domain_computation::WorthQueryInvariantExecutionFailure,
                    >>()?;
        slots
            .into_iter()
            .map(|slot| {
                inspection
                    .select_installed_invariant(&slot)?
                    .admit_state_load_plan(locators.clone(), Some(request))?
                    .execute()
            })
            .collect::<Result<Vec<_>, _>>()
    }) {
        Ok(receipts) => receipts,
        Err(failure) => {
            inspection.discard();
            return Err(WorthQueryProviderProgressionOutcome::Denied(
                crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::invariant_execution_denied(
                    DenialStage::InvariantExecution, failure)));
        }
    };
    let progression = match inspection.admit_invariant_progression(receipts) {
        Ok(progression) => progression,
        Err(_) => {
            inspection.discard();
            return Err(progression_denied(DenialStage::InvariantExecution));
        }
    };
    inspection
        .bind_invariant_progression(progression)
        .map_err(|(_, inspection)| {
            inspection.discard();
            progression_denied(DenialStage::InvariantExecution)
        })
}

fn invariant_locators(
    inspection: &crate::domain_computation::WorthQueryProposedStateInspection<'_>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<
    Vec<WorthQueryInvariantStateLocator>,
    crate::domain_computation::WorthQueryInvariantExecutionFailure,
> {
    inspection
        .facts()
        .iter()
        .map(|fact| {
            crate::domain_computation::provider_session::check_invariant_request_live(Some(
                request,
            ))?;
            WorthQueryInvariantStateLocator::new("application-proposed-state", fact.identity())
        })
        .collect::<Result<Vec<_>, _>>()
}

/// Effect lowering and overlay materialization have one owner for all writers.
pub(super) fn begin_invariant_inspection<'run>(
    staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
    fresh: crate::domain_computation::WorthQueryFreshDecisionReadSet,
    steps: std::sync::Arc<[crate::domain_computation::WorthQueryProvisionalEffectStep]>,
) -> Result<
    crate::domain_computation::WorthQueryProposedStateInspection<'run>,
    WorthQueryProviderProgressionOutcome,
> {
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
    let inspection = staged
        .begin_provisional_attempt(fresh, lowered)
        .map_err(|_| progression_denied(DenialStage::ProvisionalState))?
        .materialize_proposed_state()
        .inspect();
    Ok(inspection)
}
