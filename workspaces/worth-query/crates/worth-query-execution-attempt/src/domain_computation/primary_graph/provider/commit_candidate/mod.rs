mod commit_progression;
mod overlay_lease;

use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::{
    WorthQueryFreshDecisionReadSet, WorthQueryLoweredProvisionalEffectProgram,
    WorthQueryProviderSessionView, WorthQueryProvisionalDenialKind, WorthQueryProvisionalFailure,
    WorthQuerySessionBoundReadsAndEffects,
};
use overlay_lease::CandidatePhaseLease;

pub use commit_progression::*;

/// The exact lowered application effects have been matched to the registered attempt.
pub struct WorthQueryStagedProviderCandidate<'run> {
    // Candidate custody retires before abandonment of the live session.
    lease: CandidatePhaseLease,
    staged: WorthQuerySessionBoundReadsAndEffects<'run>,
    read_set: WorthQueryFreshDecisionReadSet,
}

impl WorthQueryPrimaryGraphProvider {
    pub fn stage_commit_candidate<'run>(
        &self,
        staged: WorthQuerySessionBoundReadsAndEffects<'run>,
        read_set: WorthQueryFreshDecisionReadSet,
        program: WorthQueryLoweredProvisionalEffectProgram,
    ) -> Result<WorthQueryStagedProviderCandidate<'run>, WorthQueryProvisionalFailure> {
        let result = validate_candidate_inputs(&staged, &read_set, &program).and_then(|()| {
            self.attempts
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .stage_candidate(staged.provider_session_view(), program.steps())
                .map_err(WorthQueryProvisionalFailure::invalid_program)
        });
        if let Err(failure) = result {
            return Err(failure.with_recovery_posture(staged.abort().recovery_posture()));
        }
        Ok(WorthQueryStagedProviderCandidate {
            lease: CandidatePhaseLease::new(
                self.attempts.clone(),
                staged.provider_session_terminal_binding(),
            ),
            staged,
            read_set,
        })
    }

    pub fn bind_validated_candidate<'run>(
        &self,
        candidate: WorthQueryStagedProviderCandidate<'run>,
    ) -> Result<WorthQueryValidatedProviderCandidate<'run>, WorthQueryStagedProviderCandidate<'run>>
    {
        let approved = self
            .attempts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_staged_session_preparable(candidate.provider_session_view());
        if approved {
            Ok(WorthQueryValidatedProviderCandidate::new(candidate))
        } else {
            Err(candidate)
        }
    }
}

impl WorthQueryStagedProviderCandidate<'_> {
    pub fn provider_session_view(&self) -> WorthQueryProviderSessionView<'_> {
        self.staged.provider_session_view()
    }

    pub fn discard(mut self) {
        self.lease.discard();
        let _ = self.staged.abort();
    }
}

fn validate_candidate_inputs(
    staged: &WorthQuerySessionBoundReadsAndEffects<'_>,
    read_set: &WorthQueryFreshDecisionReadSet,
    program: &WorthQueryLoweredProvisionalEffectProgram,
) -> Result<(), WorthQueryProvisionalFailure> {
    let binding = staged.provisional_binding_identity();
    if !read_set.belongs_to(binding)
        || !program.belongs_to(binding)
        || !program.uses_read_set(read_set.read_set_identity())
    {
        return Err(WorthQueryProvisionalFailure::new(
            WorthQueryProvisionalDenialKind::SessionBindingMismatch,
            "lowered effects and reads belong to another provider session",
        ));
    }
    if program.steps().iter().any(|step| {
        step.proposal_basis()
            .is_some_and(|basis| basis.target_generation() != 1)
    }) {
        return Err(WorthQueryProvisionalFailure::new(
            WorthQueryProvisionalDenialKind::ProposalBasisMismatch,
            "proposal target generation does not match the staged attempt",
        ));
    }
    Ok(())
}
