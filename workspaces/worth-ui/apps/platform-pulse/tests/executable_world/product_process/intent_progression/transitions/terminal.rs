use crate::source_delta::{
    DeniedIntentDelta, DisabledIntentDelta, FinalHeldIntentDelta, IntentRouteRemovalSourceDelta,
};
use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseIntentExecutorGateObservation as Gate,
    PlatformPulseIntentOperabilityObservation as Operability,
};

use super::super::observation::{self, ExpectedTerminalPosture};
use super::super::states::{
    Cancelled, DisabledStopped, FinalHeld, PolicyDeniedStopped, SecondCompleted,
};
use super::super::PlatformPulseIntentJourneyFailure;
use super::{await_rebind_cancellation, IntentJourney};

impl<'a> IntentJourney<'a, SecondCompleted> {
    pub(super) fn stop_disabled(
        mut self,
        delta: DisabledIntentDelta,
    ) -> Result<IntentJourney<'a, DisabledStopped>, PlatformPulseIntentJourneyFailure> {
        self.advance_action("edit-intent-disabled")?;
        delta
            .apply(&self.world.installation)
            .map_err(PlatformPulseIntentJourneyFailure::SourceAction)?;
        self.evidence.record_source_action();
        let sequence =
            observation::await_intent_input(self.world, 5, Operability::Disabled, Gate::Released)?;
        self.evidence.record_sequence(sequence);
        self.advance_action("activate-disabled-action")?;
        self.activate(self.action.point())?;
        let sequence =
            observation::await_terminal_posture(self.world, ExpectedTerminalPosture::Denied)?;
        self.evidence.record_sequence(sequence);
        self.record_refresh()?;
        self.visible(self.action.region())?;
        self.advance_action("observe-disabled-denial")?;
        Ok(self.with_state(DisabledStopped))
    }
}

impl<'a> IntentJourney<'a, DisabledStopped> {
    pub(super) fn stop_policy_denied(
        mut self,
        delta: DeniedIntentDelta,
    ) -> Result<IntentJourney<'a, PolicyDeniedStopped>, PlatformPulseIntentJourneyFailure> {
        self.advance_action("edit-intent-policy-denied")?;
        delta
            .apply(&self.world.installation)
            .map_err(PlatformPulseIntentJourneyFailure::SourceAction)?;
        self.evidence.record_source_action();
        let sequence =
            observation::await_intent_input(self.world, 6, Operability::Denied, Gate::Released)?;
        self.evidence.record_sequence(sequence);
        self.advance_action("activate-policy-denied-action")?;
        self.activate(self.action.point())?;
        let sequence =
            observation::await_terminal_posture(self.world, ExpectedTerminalPosture::Denied)?;
        self.evidence.record_sequence(sequence);
        self.record_refresh()?;
        self.visible(self.action.region())?;
        self.advance_action("observe-policy-denial")?;
        Ok(self.with_state(PolicyDeniedStopped))
    }
}

impl<'a> IntentJourney<'a, PolicyDeniedStopped> {
    pub(super) fn activate_final_held(
        mut self,
        delta: FinalHeldIntentDelta,
    ) -> Result<IntentJourney<'a, FinalHeld>, PlatformPulseIntentJourneyFailure> {
        self.advance_action("edit-intent-final-held")?;
        delta
            .apply(&self.world.installation)
            .map_err(PlatformPulseIntentJourneyFailure::SourceAction)?;
        self.evidence.record_source_action();
        let sequence =
            observation::await_intent_input(self.world, 7, Operability::Ready, Gate::Held)?;
        self.evidence.record_sequence(sequence);
        self.advance_action("activate-final-held-action")?;
        self.activate(self.action.point())?;
        let admitted = observation::await_admitted(self.world)?;
        self.evidence.record_sequence(admitted.sequence);
        let sequence = observation::await_executor_started(self.world, admitted.value)?;
        self.evidence.record_provider_start();
        self.evidence.record_sequence(sequence);
        self.record_refresh()?;
        self.visible(self.action.region())?;
        self.advance_action("observe-final-held-action")?;
        Ok(self.with_state(FinalHeld {
            attempt: admitted.value,
        }))
    }
}

impl<'a> IntentJourney<'a, FinalHeld> {
    pub(super) fn cancel_through_rebind(
        mut self,
        delta: IntentRouteRemovalSourceDelta,
    ) -> Result<IntentJourney<'a, Cancelled>, PlatformPulseIntentJourneyFailure> {
        self.advance_action("edit-remove-intent-route")?;
        delta
            .apply(&self.world.installation)
            .map_err(PlatformPulseIntentJourneyFailure::SourceAction)?;
        self.evidence.record_source_action();
        let sequence = await_rebind_cancellation(self.world, self.state.attempt)?;
        self.evidence.record_sequence(sequence);
        self.evidence.record_cancelled_attempt(self.state.attempt);
        self.record_refresh()?;
        self.visible(self.action.region())?;
        self.advance_action("observe-rebind-cancellation")?;
        Ok(self.with_state(Cancelled))
    }
}
