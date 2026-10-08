use crate::domain_computation::primary_graph::application_attempt::provider_execution::outcome::{
    progression_denied, WorthQueryProviderProgressionOutcome,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenialStage as DenialStage;
use crate::domain_computation::WorthQueryInvariantStateLocator;

pub(super) fn progress_invariant_candidate<'run>(
    staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
    fresh: crate::domain_computation::WorthQueryFreshDecisionReadSet,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    steps: std::sync::Arc<[crate::domain_computation::WorthQueryProvisionalEffectStep]>,
    provider: &std::sync::Arc<
        crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
    >,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    crate::domain_computation::WorthQueryInvariantApprovedProposedState<'run>,
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
                crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::provider_rejected_with_detail(
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
    let locators = inspection
        .facts()
        .iter()
        .map(|fact| {
            crate::domain_computation::provider_session::check_invariant_request_live(Some(
                request,
            ))?;
            WorthQueryInvariantStateLocator::new("application-proposed-state", fact.identity())
        })
        .collect::<Result<Vec<_>, _>>();
    let candidate = locators
        .as_ref()
        .map(|_| ())
        .map_err(Clone::clone)
        .and_then(|()| {
            crate::domain_computation::provider_session::check_invariant_request_live(Some(request))
        })
        .and_then(|()| {
            provider.admit_primary_candidate(inspection.provider_session_view(), allocation_policy)
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
