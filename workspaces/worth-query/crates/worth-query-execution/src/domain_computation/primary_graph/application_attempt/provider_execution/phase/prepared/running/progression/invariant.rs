use crate::domain_computation::primary_graph::application_attempt::provider_execution::outcome::{
    progression_denied, WorthQueryProviderProgressionOutcome,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenialStage as DenialStage;
use crate::domain_computation::WorthQueryInvariantStateLocator;

pub(super) fn progress_invariant_candidate<'run>(
    staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
    fresh: crate::domain_computation::WorthQueryFreshDecisionReadSet,
    steps: Vec<crate::domain_computation::WorthQueryProvisionalEffectStep>,
    provider: &std::sync::Arc<
        crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
    >,
) -> Result<
    crate::domain_computation::WorthQueryInvariantApprovedProposedState<'run>,
    WorthQueryProviderProgressionOutcome,
> {
    let lowered = match staged
        .effect_authority()
        .lower_provisional_program(&fresh, steps)
    {
        Ok(lowered) => lowered,
        Err(_) => {
            let _ = staged.abort();
            return Err(progression_denied(DenialStage::EffectLowering));
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
            WorthQueryInvariantStateLocator::new("application-proposed-state", fact.identity())
        })
        .collect::<Result<Vec<_>, _>>();
    let _candidate_admission = match provider
        .admit_primary_candidate(inspection.provider_session_view())
    {
        Ok(admission) => admission,
        Err(failure) => {
            inspection.discard();
            return Err(match failure.kind() {
                    crate::domain_computation::WorthQueryInvariantExecutionDenialKind::RetentionCapacityExhausted => {
                        WorthQueryProviderProgressionOutcome::Denied(
                            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::retention_capacity_exhausted(
                                DenialStage::InvariantExecution,
                            ),
                        )
                    }
                    crate::domain_computation::WorthQueryInvariantExecutionDenialKind::RetentionIdentityExhausted => {
                        WorthQueryProviderProgressionOutcome::Denied(
                            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::retention_identity_exhausted(
                                DenialStage::InvariantExecution,
                            ),
                        )
                    }
                    crate::domain_computation::WorthQueryInvariantExecutionDenialKind::ProductBasisStale => {
                        WorthQueryProviderProgressionOutcome::Denied(
                            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDenial::product_basis_stale(
                                DenialStage::InvariantExecution,
                            ),
                        )
                    }
                    _ => progression_denied(DenialStage::InvariantExecution),
                });
        }
    };
    let receipts = match locators.and_then(|locators| {
        let slots = inspection
            .installed_invariant_requirements()
            .iter()
            .map(|requirement| requirement.slot().to_owned())
            .collect::<Vec<_>>();
        slots
            .into_iter()
            .map(|slot| {
                inspection
                    .select_installed_invariant(&slot)?
                    .admit_state_load_plan(locators.clone())?
                    .execute()
            })
            .collect::<Result<Vec<_>, _>>()
    }) {
        Ok(receipts) => receipts,
        Err(_) => {
            inspection.discard();
            return Err(progression_denied(DenialStage::InvariantExecution));
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
