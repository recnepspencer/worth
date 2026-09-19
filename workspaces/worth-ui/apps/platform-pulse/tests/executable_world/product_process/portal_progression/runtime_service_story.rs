use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseCommandTransitionInspection, PlatformPulseIntentAttemptObservationReference,
    PlatformPulseIntentExecutorGateObservation, PlatformPulseIntentOperabilityObservation,
    PlatformPulseIntentPostureObservation, PlatformPulseLifecycleObservation,
    PlatformPulseQueryActionObservation, PlatformPulseQueryActionPreconditionDenial,
};

use super::{
    incidental_visual, input::run_platform_command, next, unexpected, NativeBoundExecutableWorld,
    PlatformPulsePortalJourneyFailure, WatchedPulseTransition,
};
use crate::source_delta::{PulseSourceDeltaIdentity, QueryDenialRequestedIntentDelta};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlatformPulseRuntimeServiceStoryEvidence {
    input_sequence: u64,
    application_action_sequence: u64,
    application_terminal_sequence: u64,
    portal_action_sequence: u64,
    portal_terminal_sequence: u64,
    active_query_source_revision: u64,
    submitted_query_source_revision: u64,
    application_command: PlatformPulseCommandTransitionInspection,
    portal_command: PlatformPulseCommandTransitionInspection,
}

#[derive(Clone)]
pub(super) struct PlatformPulseApplicationCommandEvidence {
    input_sequence: u64,
    action_sequence: u64,
    terminal_sequence: u64,
    active_query_source_revision: u64,
    submitted_query_source_revision: u64,
    command: PlatformPulseCommandTransitionInspection,
}

pub(super) fn exercise_application(
    world: &mut NativeBoundExecutableWorld,
) -> Result<PlatformPulseApplicationCommandEvidence, PlatformPulsePortalJourneyFailure> {
    let action = QueryDenialRequestedIntentDelta
        .apply(&world.installation)
        .map_err(PlatformPulsePortalJourneyFailure::SourceAction)?;
    if action.identity() != PulseSourceDeltaIdentity::IntentQueryDenialRequested
        || action.action_count() != 1
        || action.written_bytes() == 0
        || action.content_fingerprint() == 0
        || action.entry_source() != world.installation.intent_source()
    {
        return Err(PlatformPulsePortalJourneyFailure::RuntimeServiceEvidence(
            "Query-denial input was not one exact atomic product-source action",
        ));
    }
    let input_sequence = await_denial_input(world)?;
    let (action_sequence, terminal_sequence, active, submitted, command) = invoke(world)?;
    if !(input_sequence < action_sequence && action_sequence < terminal_sequence) {
        return Err(PlatformPulsePortalJourneyFailure::RuntimeServiceEvidence(
            "application command lifecycle sequence was not causal",
        ));
    }
    Ok(PlatformPulseApplicationCommandEvidence {
        input_sequence,
        action_sequence,
        terminal_sequence,
        active_query_source_revision: active,
        submitted_query_source_revision: submitted,
        command,
    })
}

pub(super) fn exercise_portal(
    world: &mut NativeBoundExecutableWorld,
    application: PlatformPulseApplicationCommandEvidence,
) -> Result<PlatformPulseRuntimeServiceStoryEvidence, PlatformPulsePortalJourneyFailure> {
    let (portal_action_sequence, portal_terminal_sequence, active, submitted, portal_command) =
        invoke(world)?;
    if application.active_query_source_revision != active
        || application.submitted_query_source_revision != submitted
        || application.terminal_sequence >= portal_action_sequence
        || portal_action_sequence >= portal_terminal_sequence
    {
        return Err(PlatformPulsePortalJourneyFailure::RuntimeServiceEvidence(
            "application and active-portal command contexts were not one ordered Query story",
        ));
    }
    Ok(PlatformPulseRuntimeServiceStoryEvidence {
        input_sequence: application.input_sequence,
        application_action_sequence: application.action_sequence,
        application_terminal_sequence: application.terminal_sequence,
        portal_action_sequence,
        portal_terminal_sequence,
        active_query_source_revision: active,
        submitted_query_source_revision: submitted,
        application_command: application.command,
        portal_command,
    })
}

fn invoke(
    world: &mut NativeBoundExecutableWorld,
) -> Result<
    (u64, u64, u64, u64, PlatformPulseCommandTransitionInspection),
    PlatformPulsePortalJourneyFailure,
> {
    run_platform_command(world)?;
    let admitted = await_admitted(world)?;
    await_executor_started(world, admitted)?;
    let (action_sequence, active_query_source_revision, submitted_query_source_revision) =
        await_query_denial(world, admitted)?;
    let (terminal_sequence, command) = await_denied_posture(world)?;
    Ok((
        action_sequence,
        terminal_sequence,
        active_query_source_revision,
        submitted_query_source_revision,
        command,
    ))
}

fn await_denial_input(
    world: &mut NativeBoundExecutableWorld,
) -> Result<u64, PlatformPulsePortalJourneyFailure> {
    loop {
        let envelope = next(world, WatchedPulseTransition::IntentInputAdmitted)?;
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::IntentInputAdmitted(observed)
                if observed.revision() == 2
                    && observed.operability()
                        == PlatformPulseIntentOperabilityObservation::Ready
                    && observed.executor_gate()
                        == PlatformPulseIntentExecutorGateObservation::Released
                    && observed.query_denial_requested() =>
            {
                return Ok(envelope.sequence().value())
            }
            outcome if incidental_visual(outcome) => {}
            outcome => return Err(unexpected(outcome)),
        }
    }
}

fn await_admitted(
    world: &mut NativeBoundExecutableWorld,
) -> Result<PlatformPulseIntentAttemptObservationReference, PlatformPulsePortalJourneyFailure> {
    loop {
        let envelope = next(world, WatchedPulseTransition::IntentPosturePublished)?;
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::IntentPosturePublished(posture) => {
                let PlatformPulseIntentPostureObservation::Admitted { reference } =
                    posture.posture()
                else {
                    return Err(unexpected(envelope.outcome()));
                };
                return Ok(*reference);
            }
            outcome if incidental_visual(outcome) => {}
            outcome => return Err(unexpected(outcome)),
        }
    }
}

fn await_executor_started(
    world: &mut NativeBoundExecutableWorld,
    admitted: PlatformPulseIntentAttemptObservationReference,
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    loop {
        let envelope = next(world, WatchedPulseTransition::IntentExecutorStarted)?;
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::IntentExecutorStarted(started)
                if started.reference() == admitted
                    && started.transition_count() == 1
                    && started.active_slots_visited() == 1
                    && started.provider_calls() == 1
                    && started.provider_polls() == 0
                    && started.cancellation_calls() == 0
                    && started.settlements() == 0 =>
            {
                return Ok(())
            }
            outcome if incidental_visual(outcome) => {}
            outcome => return Err(unexpected(outcome)),
        }
    }
}

fn await_query_denial(
    world: &mut NativeBoundExecutableWorld,
    admitted: PlatformPulseIntentAttemptObservationReference,
) -> Result<(u64, u64, u64), PlatformPulsePortalJourneyFailure> {
    loop {
        let envelope = next(world, WatchedPulseTransition::IntentQueryAction)?;
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::QueryAction(
                PlatformPulseQueryActionObservation::Denied {
                    reference,
                    action_input_revision,
                    denial: PlatformPulseQueryActionPreconditionDenial::SourceRevisionMismatch,
                    active_query_source_revision,
                    submitted_query_source_revision,
                },
            ) if *reference == admitted
                && *action_input_revision == 2
                && active_query_source_revision.checked_add(1)
                    == Some(*submitted_query_source_revision) =>
            {
                return Ok((
                    envelope.sequence().value(),
                    *active_query_source_revision,
                    *submitted_query_source_revision,
                ))
            }
            outcome if incidental_visual(outcome) => {}
            outcome => return Err(unexpected(outcome)),
        }
    }
}

fn await_denied_posture(
    world: &mut NativeBoundExecutableWorld,
) -> Result<(u64, PlatformPulseCommandTransitionInspection), PlatformPulsePortalJourneyFailure> {
    loop {
        let envelope = next(world, WatchedPulseTransition::IntentPosturePublished)?;
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::IntentPosturePublished(posture)
                if matches!(
                    posture.posture(),
                    PlatformPulseIntentPostureObservation::Denied
                ) && posture.frame().diagnostic_value() > 0 =>
            {
                let command = posture.latest_command_transition().cloned().ok_or(
                    PlatformPulsePortalJourneyFailure::RuntimeServiceEvidence(
                        "typed command inspection was absent from the native command turn",
                    ),
                )?;
                return Ok((envelope.sequence().value(), command));
            }
            outcome if incidental_visual(outcome) => {}
            outcome => return Err(unexpected(outcome)),
        }
    }
}

impl PlatformPulseRuntimeServiceStoryEvidence {
    pub(crate) const fn sequences(&self) -> [u64; 5] {
        [
            self.input_sequence,
            self.application_action_sequence,
            self.application_terminal_sequence,
            self.portal_action_sequence,
            self.portal_terminal_sequence,
        ]
    }

    pub(crate) const fn active_query_source_revision(&self) -> u64 {
        self.active_query_source_revision
    }

    pub(crate) const fn submitted_query_source_revision(&self) -> u64 {
        self.submitted_query_source_revision
    }

    pub(crate) fn command_transitions(&self) -> [&PlatformPulseCommandTransitionInspection; 2] {
        [&self.application_command, &self.portal_command]
    }
}
