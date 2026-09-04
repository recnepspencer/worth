use super::state::{UiRebindReservation, UiRebindRuntimeState};
use crate::runtime::rebind::{
    UiRebindCancellationPolicy, UiRebindDeadlinePolicy, UiRebindPlan, UiRebindReservationDenial,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiRebindExecutionRequest {
    now_tick: u64,
    cancellation_requested: bool,
}

pub(crate) struct UiRebindFinalAdmissionBasis<'basis> {
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    source_basis: u64,
    predecessor_generation: &'basis crate::facade::prepared_application_authority::
        WorthUiPreparedApplicationGenerationIdentity,
}

#[derive(Debug, PartialEq)]
pub enum UiRebindPreparationDenial {
    ForeignSession,
    StaleSourceBasis,
    StalePredecessorGeneration,
    CandidateGenerationMismatch,
    TimedOutBeforeEffects,
    CancelledBeforeEffects,
    Reservation(UiRebindReservationDenial),
    CandidateBindingMismatch,
    CandidateAllocation,
    CandidateLowering,
    CandidateStaging,
    FrameBoundaryUnavailable,
    ContentMountedPreparation(Box<crate::mounting::UiMountedFramePreparationDenial>),
    CandidateMountedPreparation(Box<crate::mounting::UiMountedFramePreparationDenial>),
    CandidateCutoverPreparation,
    PlannedChangeBecameSemanticNoOp,
    UnsupportedNonSourcePlan,
    InvalidSemanticProof,
    AppearanceThemeSuccession(
        crate::runtime::presentation_state::UiAppearanceGenerationSuccessionDenial,
    ),
    AppearanceInspectionSuccession(
        crate::runtime::appearance::UiAppearanceInspectionGenerationSuccessionDenial,
    ),
}

impl UiRebindExecutionRequest {
    pub const fn new(now_tick: u64) -> Self {
        Self {
            now_tick,
            cancellation_requested: false,
        }
    }

    pub const fn with_cancellation_requested(mut self) -> Self {
        self.cancellation_requested = true;
        self
    }

    pub const fn with_now_tick(mut self, now_tick: u64) -> Self {
        self.now_tick = now_tick;
        self
    }

    pub const fn now_tick(self) -> u64 {
        self.now_tick
    }
}

impl<'basis> UiRebindFinalAdmissionBasis<'basis> {
    pub(crate) const fn new(
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        source_basis: u64,
        predecessor_generation: &'basis crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
    ) -> Self {
        Self {
            session,
            source_basis,
            predecessor_generation,
        }
    }
}

pub(crate) fn admit_plan(
    state: &UiRebindRuntimeState,
    basis: UiRebindFinalAdmissionBasis<'_>,
    plan: &UiRebindPlan,
    request: UiRebindExecutionRequest,
) -> Result<UiRebindReservation, UiRebindPreparationDenial> {
    let classification = plan.basis().classification();
    if classification.session() != basis.session {
        return Err(UiRebindPreparationDenial::ForeignSession);
    }
    if classification.source_basis() != basis.source_basis {
        return Err(UiRebindPreparationDenial::StaleSourceBasis);
    }
    if classification.predecessor_generation() != basis.predecessor_generation {
        return Err(UiRebindPreparationDenial::StalePredecessorGeneration);
    }
    if let Some(candidate) = plan.semantic_candidate_generation() {
        if candidate != plan.basis().candidate_generation() {
            return Err(UiRebindPreparationDenial::CandidateGenerationMismatch);
        }
    }
    if matches!(
        plan.execution_policy().deadline(),
        UiRebindDeadlinePolicy::At(deadline) if request.now_tick > deadline.tick()
    ) {
        return Err(UiRebindPreparationDenial::TimedOutBeforeEffects);
    }
    if request.cancellation_requested
        && matches!(
            plan.execution_policy().cancellation(),
            UiRebindCancellationPolicy::AtSafePoints(_)
        )
    {
        return Err(UiRebindPreparationDenial::CancelledBeforeEffects);
    }
    state
        .reserve_plan()
        .map_err(UiRebindPreparationDenial::Reservation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::observation::UiChangeClassificationOutcome;

    #[test]
    fn final_admission_rejects_elapsed_plan_before_reserving() {
        let mut session = crate::runtime::tests::active_application_session_test_support::
            source_backed_component_session();
        let candidate = crate::runtime::tests::active_application_session_test_support::
            component_candidate_submission(
                &session,
                "phase-312-final-admission-timeout",
                "workspace.component.active_session_candidate",
            );
        let mut turn = session.begin_observation_turn().unwrap();
        turn.admit_source(candidate).unwrap();
        let admitted = turn.seal().unwrap();
        let changed = match session.classify_observations(admitted).unwrap() {
            UiChangeClassificationOutcome::Changed(changed) => changed,
            _ => panic!("candidate changes semantics"),
        };
        let lifecycle = session
            .resolve_affected_scope(changed)
            .unwrap()
            .resolve_identity_lifecycle()
            .unwrap();
        let policy = crate::runtime::rebind::UiRebindExecutionPolicy::ordinary()
            .with_deadline(session.rebind_deadline_at(10));
        let plan = session.compile_rebind_plan(lifecycle, policy).unwrap();
        assert!(matches!(
            session.prepare_rebind(plan, UiRebindExecutionRequest::new(11)),
            Err(UiRebindPreparationDenial::TimedOutBeforeEffects)
        ));
        assert!(session.shutdown().rebind().is_empty());
    }

    #[test]
    fn final_admission_checks_exact_session_source_and_predecessor_before_reserving() {
        let mut session = crate::runtime::tests::active_application_session_test_support::
            source_backed_component_session();
        let foreign = crate::runtime::tests::active_application_session_test_support::
            source_backed_component_session();
        let candidate = crate::runtime::tests::active_application_session_test_support::
            component_candidate_submission(
                &session,
                "phase-312-final-admission-currentness",
                "workspace.component.active_session_candidate",
            );
        let mut turn = session.begin_observation_turn().unwrap();
        turn.admit_source(candidate).unwrap();
        let admitted = turn.seal().unwrap();
        let changed = match session.classify_observations(admitted).unwrap() {
            UiChangeClassificationOutcome::Changed(changed) => changed,
            _ => panic!("candidate changes semantics"),
        };
        let lifecycle = session
            .resolve_affected_scope(changed)
            .unwrap()
            .resolve_identity_lifecycle()
            .unwrap();
        let plan = session
            .compile_rebind_plan(
                lifecycle,
                crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
            )
            .unwrap();
        let classification = plan.basis().classification();
        let state =
            UiRebindRuntimeState::new(crate::runtime::rebind::UiRebindProfile::platform_pulse());
        let request = UiRebindExecutionRequest::new(1);

        let current = UiRebindFinalAdmissionBasis::new(
            classification.session(),
            classification.source_basis(),
            classification.predecessor_generation(),
        );
        let reservation = admit_plan(&state, current, &plan, request)
            .expect("exact current plan reserves one pending slot");
        assert_eq!(state.pending_plan_count(), 1);
        drop(reservation);
        assert_eq!(state.pending_plan_count(), 0);

        let foreign_session = UiRebindFinalAdmissionBasis::new(
            foreign.session_identity(),
            classification.source_basis(),
            classification.predecessor_generation(),
        );
        assert!(matches!(
            admit_plan(&state, foreign_session, &plan, request),
            Err(UiRebindPreparationDenial::ForeignSession)
        ));
        let stale_source = UiRebindFinalAdmissionBasis::new(
            classification.session(),
            classification.source_basis().wrapping_add(1),
            classification.predecessor_generation(),
        );
        assert!(matches!(
            admit_plan(&state, stale_source, &plan, request),
            Err(UiRebindPreparationDenial::StaleSourceBasis)
        ));
        let stale_predecessor = UiRebindFinalAdmissionBasis::new(
            classification.session(),
            classification.source_basis(),
            plan.basis().candidate_generation(),
        );
        assert!(matches!(
            admit_plan(&state, stale_predecessor, &plan, request),
            Err(UiRebindPreparationDenial::StalePredecessorGeneration)
        ));
        assert_eq!(state.pending_plan_count(), 0);
        assert!(state.shutdown().is_empty());
        assert!(session.shutdown().rebind().is_empty());
        assert!(foreign.shutdown().rebind().is_empty());
    }
}
