use crate::domain_computation::primary_graph as graph;
use crate::domain_computation::WorthQueryInvariantExecutionDenialKind as InvariantDenial;
use crate::domain_computation::WorthQueryInvariantStateLocator;
use graph::application_attempt::provider_compare_denial::{
    provider_session_control_denied, provider_session_kind_denied,
};
use graph::application_attempt::provider_execution::outcome::{
    progression_denied, WorthQueryProviderProgressionOutcome,
};
use graph::application_attempt::WorthQueryApplicationCommitDenialStage as DenialStage;
use graph::WorthQueryApplicationCommitDenial as Denial;

pub(super) fn progress_invariant_candidate<'run>(
    staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
    fresh: crate::domain_computation::WorthQueryFreshDecisionReadSet,
    steps: std::sync::Arc<[crate::domain_computation::WorthQueryProvisionalEffectStep]>,
    provider: &std::sync::Arc<graph::provider::WorthQueryPrimaryGraphProvider>,
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
            if let Some(custom_invariant) = failure.custom_invariant_denial().cloned() {
                return Err(WorthQueryProviderProgressionOutcome::Denied(
                    Denial::custom_invariant_denied(
                        DenialStage::InvariantExecution,
                        custom_invariant,
                        failure.detail().to_owned(),
                    ),
                ));
            }
            return Err(match failure.kind() {
                InvariantDenial::ExecutionDenied(kind) => {
                    WorthQueryProviderProgressionOutcome::Denied(provider_session_kind_denied(
                        kind,
                        DenialStage::InvariantExecution,
                        failure.detail(),
                    ))
                }
                InvariantDenial::ExecutionControlStopped(kind) => {
                    WorthQueryProviderProgressionOutcome::Denied(provider_session_control_denied(
                        kind,
                        DenialStage::InvariantExecution,
                        failure.detail(),
                    ))
                }
                InvariantDenial::RetentionCapacityExhausted => {
                    WorthQueryProviderProgressionOutcome::Denied(
                        Denial::retention_capacity_exhausted(DenialStage::InvariantExecution),
                    )
                }
                InvariantDenial::RetentionIdentityExhausted => {
                    WorthQueryProviderProgressionOutcome::Denied(
                        Denial::retention_identity_exhausted(DenialStage::InvariantExecution),
                    )
                }
                InvariantDenial::ProductBasisStale => WorthQueryProviderProgressionOutcome::Denied(
                    Denial::product_basis_stale(DenialStage::InvariantExecution),
                ),
                InvariantDenial::CandidateValidatorWorkExceeded {
                    maximum_work,
                    required_work,
                } => WorthQueryProviderProgressionOutcome::Denied(
                    Denial::candidate_validator_work_exceeded(
                        DenialStage::InvariantExecution,
                        maximum_work,
                        required_work,
                    ),
                ),
                InvariantDenial::InvariantNotInstalled
                | InvariantDenial::ExecutorRoleMismatch
                | InvariantDenial::UndeclaredStateLoadFamily
                | InvariantDenial::StateLoadBudgetExceeded
                | InvariantDenial::ExecutionBudgetExceeded
                | InvariantDenial::ProviderUnsupported
                | InvariantDenial::ProviderRejected
                | InvariantDenial::RelationalDeferred(_)
                | InvariantDenial::CustomInvariantDenied
                | InvariantDenial::SnapshotIdentityExhausted
                | InvariantDenial::TransactionOverlayCapacityExhausted { .. }
                | InvariantDenial::TransactionFootprintCapacityExhausted { .. }
                | InvariantDenial::SavepointCapacityExhausted { .. }
                | InvariantDenial::SavepointFootprintCapacityExhausted { .. }
                | InvariantDenial::SavepointIdentityExhausted
                | InvariantDenial::CandidateCapacityExhausted { .. }
                | InvariantDenial::PublishedSnapshotCapacityExhausted { .. }
                | InvariantDenial::CandidateIdentityExhausted
                | InvariantDenial::PreparedRootBudgetExhausted { .. }
                | InvariantDenial::PatchPositionReservationContended
                | InvariantDenial::ProposalIdentityExhausted
                | InvariantDenial::ProviderPanicked
                | InvariantDenial::EvidenceSubstitution
                | InvariantDenial::EmptyStateLoad
                | InvariantDenial::StateLoadClosureMismatch
                | InvariantDenial::VerdictPostureMismatch => {
                    WorthQueryProviderProgressionOutcome::Denied(
                        Denial::provider_rejected_with_detail(
                            DenialStage::InvariantExecution,
                            failure.detail().to_owned(),
                        ),
                    )
                }
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
