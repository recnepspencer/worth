use super::{
    WorthQueryLoweredProvisionalEffectProgram, WorthQueryProposedPostState,
    WorthQueryProvisionalDenialKind, WorthQueryProvisionalDiscardOutcome,
    WorthQueryProvisionalEffectProgramView, WorthQueryProvisionalFailure,
    WorthQueryProvisionalOverlayAdmission, WorthQueryProvisionalOverlayLease,
};
use crate::domain_computation::provider_session::{
    WorthQueryFreshDecisionReadSet, WorthQuerySessionBoundReadsAndEffects,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryProvisionalAttemptCounters {
    provider_stage_calls: usize,
    pub(super) provider_discard_calls: usize,
    staged_effects: usize,
}

impl WorthQueryProvisionalAttemptCounters {
    pub fn provider_stage_calls(self) -> usize {
        self.provider_stage_calls
    }

    pub fn provider_discard_calls(self) -> usize {
        self.provider_discard_calls
    }

    pub fn staged_effects(self) -> usize {
        self.staged_effects
    }
}

pub struct WorthQueryProvisionalAttempt<'run> {
    pub(super) overlay: WorthQueryProvisionalOverlayLease<'run>,
    pub(super) staged: WorthQuerySessionBoundReadsAndEffects<'run>,
    pub(super) read_set: WorthQueryFreshDecisionReadSet,
    pub(super) program: WorthQueryLoweredProvisionalEffectProgram,
    pub(super) generation: u64,
    pub(super) counters: WorthQueryProvisionalAttemptCounters,
}

impl<'run> WorthQuerySessionBoundReadsAndEffects<'run> {
    pub fn begin_provisional_attempt(
        self,
        read_set: WorthQueryFreshDecisionReadSet,
        program: WorthQueryLoweredProvisionalEffectProgram,
    ) -> Result<WorthQueryProvisionalAttempt<'run>, WorthQueryProvisionalFailure> {
        begin(self, read_set, program, 1, Default::default())
    }
}

impl<'run> WorthQueryProvisionalAttempt<'run> {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn program_identity(&self) -> &str {
        self.program.identity()
    }

    pub fn decision_read_set_identity(&self) -> &str {
        self.read_set.read_set_identity()
    }

    pub fn counters(&self) -> WorthQueryProvisionalAttemptCounters {
        self.counters
    }

    pub fn materialize_proposed_state(self) -> WorthQueryProposedPostState<'run> {
        WorthQueryProposedPostState { attempt: self }
    }

    pub fn discard(mut self) -> WorthQueryProvisionalDiscardOutcome {
        self.counters.provider_discard_calls += 1;
        let overlay_failure = self.overlay.discard().err();
        WorthQueryProvisionalDiscardOutcome::new(overlay_failure, self.staged.abort())
    }
}

pub(super) fn begin<'run>(
    staged: WorthQuerySessionBoundReadsAndEffects<'run>,
    read_set: WorthQueryFreshDecisionReadSet,
    program: WorthQueryLoweredProvisionalEffectProgram,
    generation: u64,
    mut counters: WorthQueryProvisionalAttemptCounters,
) -> Result<WorthQueryProvisionalAttempt<'run>, WorthQueryProvisionalFailure> {
    if let Err(failure) = validate_attempt_inputs(&staged, &read_set, &program, generation) {
        return Err(abort_with_failure(staged, failure));
    }
    let overlay = match stage_provider_overlay(&staged, &program, generation, &mut counters) {
        Ok(overlay) => overlay,
        Err(failure) => return Err(abort_with_failure(staged, failure)),
    };
    let binding = staged.provisional_binding_identity();
    if !overlay.belongs_to(binding, program.identity(), generation) {
        return Err(reject_overlay(
            staged,
            overlay,
            WorthQueryProvisionalDenialKind::ProviderEvidenceSubstitution,
            "provider returned overlay evidence for another attempt",
        ));
    }
    if !overlay.matches_program(&program) {
        return Err(reject_overlay(
            staged,
            overlay,
            WorthQueryProvisionalDenialKind::ProviderProgramMismatch,
            "provider proposed state does not match the lowered effect program",
        ));
    }
    Ok(WorthQueryProvisionalAttempt {
        overlay: WorthQueryProvisionalOverlayLease::new(
            staged.execution_request(),
            staged.provisional_provider_arc(),
            overlay,
        ),
        staged,
        read_set,
        program,
        generation,
        counters,
    })
}

fn validate_attempt_inputs(
    staged: &WorthQuerySessionBoundReadsAndEffects<'_>,
    read_set: &WorthQueryFreshDecisionReadSet,
    program: &WorthQueryLoweredProvisionalEffectProgram,
    generation: u64,
) -> Result<(), WorthQueryProvisionalFailure> {
    let binding = staged.provisional_binding_identity();
    if !read_set.belongs_to(binding)
        || !program.belongs_to(binding)
        || !program.uses_read_set(read_set.read_set_identity())
    {
        return Err(WorthQueryProvisionalFailure::new(
            WorthQueryProvisionalDenialKind::SessionBindingMismatch,
            "provisional inputs do not belong to the exact staged session",
        ));
    }
    if program.steps().iter().any(|step| {
        step.proposal_basis()
            .is_some_and(|basis| basis.target_generation() != generation)
    }) {
        return Err(WorthQueryProvisionalFailure::new(
            WorthQueryProvisionalDenialKind::ProposalBasisMismatch,
            "proposal target generation does not match the provisional attempt",
        ));
    }
    Ok(())
}

fn stage_provider_overlay(
    staged: &WorthQuerySessionBoundReadsAndEffects<'_>,
    program: &WorthQueryLoweredProvisionalEffectProgram,
    generation: u64,
    counters: &mut WorthQueryProvisionalAttemptCounters,
) -> Result<super::WorthQueryProvisionalOverlayEvidence, WorthQueryProvisionalFailure> {
    let admission = WorthQueryProvisionalOverlayAdmission::new(
        staged.provider_session_view(),
        staged.provisional_binding_identity(),
        program.identity(),
        generation,
    );
    counters.provider_stage_calls += 1;
    counters.staged_effects += program.steps().len();
    let invocation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        staged.provisional_provider().stage_provisional_overlay(
            staged.provider_session_view(),
            WorthQueryProvisionalEffectProgramView::new(program, generation),
            admission,
        )
    }));
    match invocation {
        Ok(result) => result,
        Err(_) => Err(WorthQueryProvisionalFailure::new(
            WorthQueryProvisionalDenialKind::ProviderPanicked,
            "provider panicked while staging the provisional overlay",
        )),
    }
}

fn reject_overlay(
    staged: WorthQuerySessionBoundReadsAndEffects<'_>,
    overlay: super::WorthQueryProvisionalOverlayEvidence,
    kind: WorthQueryProvisionalDenialKind,
    detail: &'static str,
) -> WorthQueryProvisionalFailure {
    let mut overlay = WorthQueryProvisionalOverlayLease::new(
        staged.execution_request(),
        staged.provisional_provider_arc(),
        overlay,
    );
    let cleanup = overlay.discard();
    let session_posture = staged.abort().recovery_posture();
    if cleanup.is_err() {
        return WorthQueryProvisionalFailure::new(
            WorthQueryProvisionalDenialKind::DiscardFailed,
            "rejected provider overlay could not be discarded",
        )
        .with_recovery_posture(
            crate::domain_computation::provider_session::WorthQueryProviderSessionRecoveryPosture::RecoveryRequired,
        );
    }
    WorthQueryProvisionalFailure::new(kind, detail).with_recovery_posture(session_posture)
}

fn abort_with_failure(
    staged: WorthQuerySessionBoundReadsAndEffects<'_>,
    failure: WorthQueryProvisionalFailure,
) -> WorthQueryProvisionalFailure {
    failure.with_recovery_posture(staged.abort().recovery_posture())
}
