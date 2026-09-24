use super::{presentation_epoch, require_owner_reconstruction, UiNativePresentationFailure};
use crate::native::presentation::{
    reserve_presentation_owners, settle_port_result, UiNativePendingExternalObligation,
    UiNativePresentationPortFailure,
};
use crate::native::{UiNativeEffectPosture, UiNativeHostState};

struct PendingProbe(std::rc::Rc<std::cell::Cell<bool>>);

impl UiNativePendingExternalObligation for PendingProbe {
    fn poll_observation(
        &mut self,
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
        _device: Option<&wgpu::Device>,
    ) -> crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation {
        basis.observe(crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Pending)
    }
}

impl Drop for PendingProbe {
    fn drop(&mut self) {
        self.0.set(true);
    }
}

#[test]
fn scripted_before_effect_failure_keeps_before_effect_posture() {
    let state = UiNativeHostState::new();
    let failure = UiNativePresentationFailure::BeforeEffects(
        worth_ui_host_contract::UiHostSurfacePresentationDenial::AdapterDeclined,
    );
    assert!(matches!(
        failure,
        UiNativePresentationFailure::BeforeEffects(
            worth_ui_host_contract::UiHostSurfacePresentationDenial::AdapterDeclined
        )
    ));
    assert_eq!(
        state.lifecycle.effect_posture(),
        UiNativeEffectPosture::BeforeEffects
    );
}

#[test]
fn external_port_orchestration_and_effect_postures_are_exact() {
    crate::native::presentation::prove_pending_readback_handoff();
    let mut state = UiNativeHostState::new();
    let external_dropped = std::rc::Rc::new(std::cell::Cell::new(false));
    let owners = reserve_presentation_owners(
        &mut state.resources,
        &mut state.physical_signal,
        crate::native::physical_work_signal::UiNativePhysicalPresentationBasis::test(),
    )
    .unwrap_or_else(|_| panic!("empty registry must reserve presentation owners"));
    let pending = settle_port_result(
        &mut state.resources,
        &mut state.physical_signal,
        owners,
        Err(UiNativePresentationPortFailure::ReadbackUnsettled(
            Box::new(PendingProbe(std::rc::Rc::clone(&external_dropped))),
        )),
    );
    let Err(UiNativePresentationFailure::Pending(pending)) = pending else {
        panic!("unsettled port work must remain pending");
    };
    state.pending_presentations.push(pending);
    assert_eq!(
        state.lifecycle.effect_posture(),
        UiNativeEffectPosture::BeforeEffects
    );
    assert_eq!(state.pending_presentations.len(), 1);
    assert!(!external_dropped.get());
    assert_eq!(state.resources.current().readback_buffers, 1);
    assert_eq!(state.resources.current().pending_submissions, 1);
    state.pending_presentations.clear();
    assert!(external_dropped.get());
}

#[test]
fn presentation_cancellation_transitions_the_exact_physical_request_to_recovery() {
    let mut state = UiNativeHostState::new();
    let external_dropped = std::rc::Rc::new(std::cell::Cell::new(false));
    let owners = reserve_presentation_owners(
        &mut state.resources,
        &mut state.physical_signal,
        crate::native::physical_work_signal::UiNativePhysicalPresentationBasis::test(),
    )
    .unwrap_or_else(|_| panic!("empty registry must reserve presentation owners"));
    let pending = settle_port_result(
        &mut state.resources,
        &mut state.physical_signal,
        owners,
        Err(UiNativePresentationPortFailure::ReadbackUnsettled(
            Box::new(PendingProbe(std::rc::Rc::clone(&external_dropped))),
        )),
    );
    let Err(UiNativePresentationFailure::Pending(mut pending)) = pending else {
        panic!("unsettled port work must remain pending");
    };
    let token = super::text_atlas_tests::inert_view().issue_completion_token();
    assert!(pending.bind_completion_identity(token.diagnostic_value(), None));
    state.pending_presentations.push(pending);

    let outcome = super::pending_completion::stop_pending(&mut state, token);

    assert_eq!(
        outcome,
        worth_ui_host_contract::UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun
    );
    let observation = state.physical_signal.observation();
    assert_eq!(observation.counters.cancellations, 1);
    assert_eq!(observation.counters.recovery_schedules, 1);
    assert!(matches!(
        observation.recovery,
        crate::native::physical_work_signal::UiNativePhysicalRecoveryPosture::Required {
            active_requests: 1
        }
    ));
    let cancellation = state
        .physical_signal
        .transition_observations()
        .last()
        .expect("the exact cancellation retains a physical transition observation");
    assert_eq!(
        cancellation.external_status(),
        crate::native::physical_work_signal::UiNativePhysicalSignalExternalStatusClass::CancellationEffectsMayHaveBegun
    );
    assert_eq!(
        cancellation.settlement(),
        crate::native::physical_work_signal::UiNativePhysicalSignalSettlementClass::Indeterminate
    );
    assert_eq!(state.pending_presentations.len(), 1);
    assert!(state.pending_presentations[0]
        .completion_identity()
        .is_none());
    assert!(!external_dropped.get());
}

struct SettledProbe;

impl UiNativePendingExternalObligation for SettledProbe {
    fn poll_observation(
        &mut self,
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
        _device: Option<&wgpu::Device>,
    ) -> crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation {
        basis.observe(crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Completed)
    }
}

#[test]
fn cancelling_a_physically_settled_presentation_still_confirms_its_recovery() {
    let mut state = UiNativeHostState::new();
    let owners = reserve_presentation_owners(
        &mut state.resources,
        &mut state.physical_signal,
        crate::native::physical_work_signal::UiNativePhysicalPresentationBasis::test(),
    )
    .unwrap_or_else(|_| panic!("empty registry must reserve presentation owners"));
    let pending = settle_port_result(
        &mut state.resources,
        &mut state.physical_signal,
        owners,
        Err(UiNativePresentationPortFailure::ReadbackUnsettled(
            Box::new(SettledProbe),
        )),
    );
    let Err(UiNativePresentationFailure::Pending(mut pending)) = pending else {
        panic!("unsettled port work must remain pending");
    };
    let token = super::text_atlas_tests::inert_view().issue_completion_token();
    assert!(pending.bind_completion_identity(token.diagnostic_value(), None));
    state.pending_presentations.push(pending);
    let due = state
        .physical_signal
        .next_due_tick()
        .expect("pending presentation must retain one Signal-owned poll wake");
    state
        .physical_signal
        .advance_clock_to(due)
        .expect("the exact pending poll wake must become ready");
    let identity = state.pending_presentations[0].physical_work();
    assert!(state.progress_one_physical_signal_ready());
    // The physical work settled and spent its Signal request; only the
    // completion is left, waiting for the client to collect it.
    assert_eq!(state.pending_presentations.len(), 1);
    assert!(!state.pending_presentations[0].has_active_external());
    assert_eq!(state.physical_signal.observation().active_requests, 0);

    let outcome = super::pending_completion::stop_pending(&mut state, token);

    assert_eq!(
        outcome,
        worth_ui_host_contract::UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun
    );
    // Effects may have begun, so the client awaits a recovery confirmation
    // for this attempt; the spent request is replaced by fresh recovery.
    let observation = state.physical_signal.observation();
    assert_eq!(observation.counters.cancellations, 1);
    assert_eq!(observation.counters.recovery_schedules, 1);
    assert_eq!(observation.active_requests, 1);
    assert_eq!(state.pending_presentations.len(), 1);
    assert!(state.pending_presentations[0]
        .completion_identity()
        .is_none());

    // The admitted recovery is ready at once: the work it waits on is done.
    assert_eq!(
        state.progress_one_physical_signal_ready_outcome(),
        crate::native::host_state::UiNativeHostPhysicalProgress::Presentation(
            identity,
            crate::native::host_state::UiNativePresentationPhysicalProgress::RecoveryCompleted,
        )
    );
    assert!(state.pending_presentations.is_empty());
    assert_eq!(state.physical_signal.observation().active_requests, 0);
    assert!(state.resources.current().is_zero());
}

#[test]
fn derived_state_loss_rejects_without_effects_until_owner_reconstruction_arrives() {
    let mut state = UiNativeHostState::new();
    let binding = 71;
    state.lifecycle.require_recovery(
        binding,
        crate::native::UiNativeRecoveryCause::DerivedStateLost,
    );

    let first = require_owner_reconstruction(&mut state, binding);
    let second = require_owner_reconstruction(&mut state, binding);

    for outcome in [first, second] {
        assert!(matches!(
            outcome,
            worth_ui_host_contract::UiHostSurfacePresentationOutcome::RejectedBeforeEffects(
                worth_ui_host_contract::UiHostSurfacePresentationDenial::ReconstructionRequired
            )
        ));
    }
    assert!(state.lifecycle.recovery_required(binding));
    assert!(state.pending_presentations.is_empty());
    assert_eq!(
        state.lifecycle.effect_posture(),
        UiNativeEffectPosture::BeforeEffects
    );
    assert!(state.resources.current().is_zero());
}

#[test]
fn unchanged_reuses_the_last_physical_presentation_epoch() {
    let mut state = UiNativeHostState::new();
    let binding = 73;
    let physical = presentation_epoch(&mut state, binding, 101, true).unwrap();
    let unchanged = presentation_epoch(&mut state, binding, 102, false).unwrap();
    assert_eq!(unchanged, physical);
    assert_eq!(unchanged.diagnostic_value(), 101);
    assert!(presentation_epoch(&mut state, binding + 1, 103, false).is_none());
}

#[test]
fn same_frame_sample_advances_epoch_only_when_native_pixels_are_painted() {
    let mut state = UiNativeHostState::new();
    let binding = 79;
    let initial = presentation_epoch(&mut state, binding, 201, true).unwrap();
    let offscreen_sample = presentation_epoch(&mut state, binding, 202, false).unwrap();
    let painted_sample = presentation_epoch(&mut state, binding, 203, true).unwrap();

    assert_eq!(offscreen_sample, initial);
    assert_eq!(offscreen_sample.diagnostic_value(), 201);
    assert_eq!(painted_sample.diagnostic_value(), 203);
}
