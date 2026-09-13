use crate::adjudication::{NativeControlPixelRegion, PlatformPulseActionControlPoint};
use crate::external_observation::{NativeClientPixelCapture, NativeClientPixelPoint};
use crate::source_delta::{
    ConfirmationHeldIntentDelta, ConfirmationReleasedIntentDelta, DeniedIntentDelta,
    DisabledIntentDelta, FinalHeldIntentDelta, IntentRouteRemovalSourceDelta,
    PulseCausalActionCursor, PulseCausalActionManifestFailure, ReadyReleasedIntentDelta,
};
use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseIntentExecutorGateObservation as Gate,
    PlatformPulseIntentOperabilityObservation as Operability,
};

use super::evidence::PlatformPulseIntentJourneyEvidenceBuilder;
use super::observation::{self, ExpectedTerminalPosture};
use super::states::{
    Cancelled, ConfirmationPending, ConfirmationStale, FirstCompleted, FirstHeld,
    FreshConfirmationPending, Ready, SecondCompleted,
};
use super::PlatformPulseIntentJourneyFailure;
use crate::product_process::NativeBoundExecutableWorld;

mod cancellation;
mod terminal;

use cancellation::await_rebind_cancellation;

pub(super) fn run(
    world: &mut NativeBoundExecutableWorld,
    baseline: &NativeClientPixelCapture,
    action: PlatformPulseActionControlPoint,
    evidence: &mut PlatformPulseIntentJourneyEvidenceBuilder,
    route_removal: IntentRouteRemovalSourceDelta,
    actions: &mut dyn IntentCausalActionAuthority,
) -> Result<(), PlatformPulseIntentJourneyFailure> {
    report_checkpoint(world, "intent-open");
    let ready = IntentJourney {
        world,
        baseline,
        action,
        evidence,
        actions,
        state: Ready,
    };
    let first_held = ready.activate_first()?;
    report_checkpoint(first_held.world, "intent-first-held");
    let first_completed = first_held.release_first(ReadyReleasedIntentDelta)?;
    let confirmation = first_completed.require_confirmation(ConfirmationHeldIntentDelta)?;
    let stale = confirmation.stale_confirmation(ConfirmationReleasedIntentDelta)?;
    let fresh = stale.obtain_fresh_confirmation()?;
    let second_completed = fresh.confirm_and_complete()?;
    let disabled = second_completed.stop_disabled(DisabledIntentDelta)?;
    let denied = disabled.stop_policy_denied(DeniedIntentDelta)?;
    let final_held = denied.activate_final_held(FinalHeldIntentDelta)?;
    report_checkpoint(final_held.world, "intent-final-held");
    let cancelled = final_held.cancel_through_rebind(route_removal)?;
    report_checkpoint(cancelled.world, "intent-route-removal-cancelled");
    let IntentJourney {
        state: Cancelled, ..
    } = cancelled;
    Ok(())
}

fn report_checkpoint(world: &NativeBoundExecutableWorld, phase: &str) {
    eprintln!(
        "WORTH_UI_EXECUTABLE_WORLD_PHASE phase={phase} elapsed_ms={}",
        world.journey_started.elapsed().as_millis()
    );
}

pub(super) fn run_causal_pulse(
    world: &mut NativeBoundExecutableWorld,
    baseline: &NativeClientPixelCapture,
    action: PlatformPulseActionControlPoint,
    evidence: &mut PlatformPulseIntentJourneyEvidenceBuilder,
) -> Result<(), PlatformPulseIntentJourneyFailure> {
    let mut actions = UntrackedIntentCausalActions;
    let ready = IntentJourney {
        world,
        baseline,
        action,
        evidence,
        actions: &mut actions,
        state: Ready,
    };
    let first_held = ready.activate_first()?;
    let IntentJourney {
        state: FirstCompleted,
        ..
    } = first_held.release_first(ReadyReleasedIntentDelta)?;
    Ok(())
}

pub(super) struct UntrackedIntentCausalActions;

pub(super) trait IntentCausalActionAuthority {
    fn advance(&mut self, action: &'static str) -> Result<(), PulseCausalActionManifestFailure>;
}

impl IntentCausalActionAuthority for UntrackedIntentCausalActions {
    fn advance(&mut self, _action: &'static str) -> Result<(), PulseCausalActionManifestFailure> {
        Ok(())
    }
}

impl IntentCausalActionAuthority for PulseCausalActionCursor<'_> {
    fn advance(&mut self, action: &'static str) -> Result<(), PulseCausalActionManifestFailure> {
        PulseCausalActionCursor::advance(self, action)
    }
}

struct IntentJourney<'a, State> {
    world: &'a mut NativeBoundExecutableWorld,
    baseline: &'a NativeClientPixelCapture,
    action: PlatformPulseActionControlPoint,
    evidence: &'a mut PlatformPulseIntentJourneyEvidenceBuilder,
    actions: &'a mut dyn IntentCausalActionAuthority,
    state: State,
}

impl<State> IntentJourney<'_, State> {
    fn advance_action(
        &mut self,
        action: &'static str,
    ) -> Result<(), PlatformPulseIntentJourneyFailure> {
        self.actions
            .advance(action)
            .map_err(PlatformPulseIntentJourneyFailure::CausalManifest)
    }

    fn activate(
        &mut self,
        point: NativeClientPixelPoint,
    ) -> Result<(), PlatformPulseIntentJourneyFailure> {
        observation::activate_native_control(self.world, point)?;
        self.evidence.record_native_activation();
        Ok(())
    }

    fn visible(
        &mut self,
        region: NativeControlPixelRegion,
    ) -> Result<(), PlatformPulseIntentJourneyFailure> {
        let change = observation::capture_visible_change(self.world, self.baseline, region)?;
        self.evidence.record_visible_change(change);
        Ok(())
    }

    fn visible_completed(
        &mut self,
        region: NativeControlPixelRegion,
    ) -> Result<(), PlatformPulseIntentJourneyFailure> {
        let change = observation::capture_visible_change(self.world, self.baseline, region)?;
        if !self.evidence.record_causal_visible_change(change) {
            return Err(PlatformPulseIntentJourneyFailure::EvidenceOrder(
                "completed visible change had no exact pending causal trace",
            ));
        }
        Ok(())
    }

    fn record_refresh(&mut self) -> Result<(), PlatformPulseIntentJourneyFailure> {
        let sequence = observation::await_visual_refresh(self.world)?;
        self.evidence.record_sequence(sequence);
        Ok(())
    }

    fn record_query_completion(
        &mut self,
        attempt: worth_ui_platform_pulse::observation_contract::PlatformPulseIntentAttemptObservationReference,
        action_input_revision: u64,
        query_source_revision: u64,
        status: &str,
    ) -> Result<(), PlatformPulseIntentJourneyFailure> {
        let trace = observation::await_query_completion(
            self.world,
            attempt,
            action_input_revision,
            query_source_revision,
            status,
        )?;
        if !self.evidence.record_causal_trace(trace.value) {
            return Err(PlatformPulseIntentJourneyFailure::EvidenceOrder(
                "completed intent overlapped an unobserved causal pixel edge",
            ));
        }
        self.evidence.record_query_action();
        self.evidence.record_completion();
        self.evidence.record_sequence(trace.sequence);
        Ok(())
    }
}

impl<'a> IntentJourney<'a, Ready> {
    fn activate_first(
        mut self,
    ) -> Result<IntentJourney<'a, FirstHeld>, PlatformPulseIntentJourneyFailure> {
        self.advance_action("activate-first-action")?;
        self.activate(self.action.point())?;
        let admitted = observation::await_admitted(self.world)?;
        self.evidence.record_sequence(admitted.sequence);
        let started = observation::await_executor_started(self.world, admitted.value)?;
        self.evidence.record_provider_start();
        self.evidence.record_sequence(started);
        self.record_refresh()?;
        self.visible(self.action.region())?;
        self.evidence.record_first_attempt(admitted.value);
        self.advance_action("observe-first-action-held")?;
        Ok(self.with_state(FirstHeld {
            attempt: admitted.value,
        }))
    }
}

impl<'a> IntentJourney<'a, FirstHeld> {
    fn release_first(
        mut self,
        delta: ReadyReleasedIntentDelta,
    ) -> Result<IntentJourney<'a, FirstCompleted>, PlatformPulseIntentJourneyFailure> {
        self.advance_action("edit-intent-ready-released")?;
        delta
            .apply(&self.world.installation)
            .map_err(PlatformPulseIntentJourneyFailure::SourceAction)?;
        self.evidence.record_source_action();
        let sequence =
            observation::await_intent_input(self.world, 2, Operability::Ready, Gate::Released)?;
        self.evidence.record_sequence(sequence);
        self.record_query_completion(self.state.attempt, 1, 2, "ACTION 1")?;
        self.record_refresh()?;
        self.visible_completed(self.action.region())?;
        self.advance_action("observe-first-consequence")?;
        Ok(self.with_state(FirstCompleted))
    }
}

impl<'a> IntentJourney<'a, FirstCompleted> {
    fn require_confirmation(
        mut self,
        delta: ConfirmationHeldIntentDelta,
    ) -> Result<IntentJourney<'a, ConfirmationPending>, PlatformPulseIntentJourneyFailure> {
        self.advance_action("edit-intent-confirmation-held")?;
        delta
            .apply(&self.world.installation)
            .map_err(PlatformPulseIntentJourneyFailure::SourceAction)?;
        self.evidence.record_source_action();
        let sequence = observation::await_intent_input(
            self.world,
            3,
            Operability::ConfirmationRequired,
            Gate::Held,
        )?;
        self.evidence.record_sequence(sequence);
        self.advance_action("activate-confirmation-action")?;
        self.activate(self.action.point())?;
        let challenge = observation::await_confirmation_required(self.world)?;
        self.evidence.record_sequence(challenge.sequence);
        self.record_refresh()?;
        let (control, change) =
            observation::capture_visible_confirmation(self.world, self.baseline, self.action)?;
        self.evidence.record_visible_change(change);
        self.advance_action("observe-confirmation-required")?;
        Ok(self.with_state(ConfirmationPending {
            challenge: challenge.value,
            control,
        }))
    }
}

impl<'a> IntentJourney<'a, ConfirmationPending> {
    fn stale_confirmation(
        mut self,
        delta: ConfirmationReleasedIntentDelta,
    ) -> Result<IntentJourney<'a, ConfirmationStale>, PlatformPulseIntentJourneyFailure> {
        self.advance_action("edit-intent-confirmation-released")?;
        delta
            .apply(&self.world.installation)
            .map_err(PlatformPulseIntentJourneyFailure::SourceAction)?;
        self.evidence.record_source_action();
        let sequence = observation::await_intent_input(
            self.world,
            4,
            Operability::ConfirmationRequired,
            Gate::Released,
        )?;
        self.evidence.record_sequence(sequence);
        self.advance_action("activate-stale-confirmation")?;
        self.activate(self.state.control.point())?;
        let sequence = observation::await_terminal_posture(
            self.world,
            ExpectedTerminalPosture::StaleConfirmation,
        )?;
        self.evidence.record_sequence(sequence);
        self.record_refresh()?;
        self.visible(self.state.control.region())?;
        self.advance_action("observe-stale-confirmation")?;
        let predecessor = self.state.challenge;
        let control = self.state.control;
        Ok(self.with_state(ConfirmationStale {
            predecessor,
            control,
        }))
    }
}

impl<'a> IntentJourney<'a, ConfirmationStale> {
    fn obtain_fresh_confirmation(
        mut self,
    ) -> Result<IntentJourney<'a, FreshConfirmationPending>, PlatformPulseIntentJourneyFailure>
    {
        self.advance_action("activate-fresh-action")?;
        self.activate(self.action.point())?;
        let challenge = observation::await_confirmation_required(self.world)?;
        if challenge.value == self.state.predecessor {
            return Err(PlatformPulseIntentJourneyFailure::Cancellation(
                "fresh activation replayed the stale confirmation challenge".to_owned(),
            ));
        }
        self.evidence.record_sequence(challenge.sequence);
        self.record_refresh()?;
        self.visible(self.state.control.region())?;
        self.advance_action("observe-fresh-confirmation")?;
        let control = self.state.control;
        Ok(self.with_state(FreshConfirmationPending {
            challenge: challenge.value,
            control,
        }))
    }
}

impl<'a> IntentJourney<'a, FreshConfirmationPending> {
    fn confirm_and_complete(
        mut self,
    ) -> Result<IntentJourney<'a, SecondCompleted>, PlatformPulseIntentJourneyFailure> {
        if self.state.challenge.expires_at_millis == 0 {
            return Err(PlatformPulseIntentJourneyFailure::Cancellation(
                "fresh challenge carried no expiry boundary".to_owned(),
            ));
        }
        self.advance_action("activate-fresh-confirmation")?;
        self.activate(self.state.control.point())?;
        let admitted = observation::await_admitted(self.world)?;
        self.evidence.record_sequence(admitted.sequence);
        let sequence = observation::await_executor_started(self.world, admitted.value)?;
        self.evidence.record_provider_start();
        self.evidence.record_sequence(sequence);
        self.record_query_completion(admitted.value, 4, 2, "ACTION 4")?;
        self.evidence.record_second_attempt(admitted.value);
        self.record_refresh()?;
        self.visible_completed(self.action.region())?;
        self.advance_action("observe-second-consequence")?;
        Ok(self.with_state(SecondCompleted))
    }
}

impl<'a, State> IntentJourney<'a, State> {
    fn with_state<Next>(self, state: Next) -> IntentJourney<'a, Next> {
        IntentJourney {
            world: self.world,
            baseline: self.baseline,
            action: self.action,
            evidence: self.evidence,
            actions: self.actions,
            state,
        }
    }
}
