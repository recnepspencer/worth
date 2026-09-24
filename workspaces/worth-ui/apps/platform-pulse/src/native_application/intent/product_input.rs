use worth_ui::facade::app::WorthUiNativeApplicationShell;
use worth_ui_platform_pulse::intent::{
    platform_pulse_action_confirmation_fact, platform_pulse_action_mutability_fact,
    platform_pulse_action_policy_fact, platform_pulse_action_readiness_fact,
    platform_pulse_action_revision_fact, platform_pulse_query_denial_fact,
    PlatformPulseIntentInputEvent, PlatformPulseIntentInputRecord,
};

use super::super::{PlatformPulseApplicationRuntime, PlatformPulseTerminalError};

impl PlatformPulseApplicationRuntime {
    pub(in crate::native_application) fn poll_intent_input(&mut self) {
        while self.terminal.is_running() {
            let event = self
                .intent_watch
                .as_mut()
                .and_then(|watch| watch.try_next());
            match event {
                Some(PlatformPulseIntentInputEvent::Record(record)) => {
                    if let Err(error) = self.apply_intent_input(&record) {
                        let observation = self.publisher.intent_preparation_failure();
                        self.fail(error, observation);
                        return;
                    }
                }
                Some(PlatformPulseIntentInputEvent::Failed(denial)) => {
                    let observation = self.publisher.intent_preparation_failure();
                    self.fail(PlatformPulseTerminalError::IntentWatch(denial), observation);
                    return;
                }
                None => return,
            }
        }
    }

    fn apply_intent_input(
        &mut self,
        record: &PlatformPulseIntentInputRecord,
    ) -> Result<(), PlatformPulseTerminalError> {
        let shell = self.shell.as_mut().ok_or_else(|| {
            PlatformPulseTerminalError::FrameExecution(
                "intent input arrived before native shell launch".to_owned(),
            )
        })?;
        apply_intent_facts(shell, record).map_err(PlatformPulseTerminalError::IntentFact)?;
        self.intent_gate
            .as_ref()
            .expect("prepared Pulse retains its intent executor gate")
            .apply(record.revision(), record.executor_held())
            .map_err(PlatformPulseTerminalError::IntentGate)?;
        self.publisher
            .intent_input_admitted(record)
            .map_err(|_| PlatformPulseTerminalError::ObservationPublication)
    }
}

fn apply_intent_facts(
    shell: &mut WorthUiNativeApplicationShell,
    record: &PlatformPulseIntentInputRecord,
) -> Result<(), worth_ui::facade::intent::UiIntentApplicationFactUpdateDenial> {
    shell.update_intent_boolean_fact(&platform_pulse_action_mutability_fact(), record.mutable())?;
    shell.update_intent_boolean_fact(&platform_pulse_action_readiness_fact(), record.ready())?;
    shell.update_intent_boolean_fact(
        &platform_pulse_action_policy_fact(),
        record.policy_allowed(),
    )?;
    shell.update_intent_boolean_fact(
        &platform_pulse_action_confirmation_fact(),
        record.confirmation_required(),
    )?;
    shell
        .update_intent_unsigned64_fact(&platform_pulse_action_revision_fact(), record.revision())?;
    shell.update_intent_boolean_fact(
        &platform_pulse_query_denial_fact(),
        record.query_denial_requested(),
    )?;
    Ok(())
}
