use std::time::{Duration, Instant};

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseFocusTransitionInspection, PlatformPulseIntentAttemptObservationReference,
    PlatformPulseIntentPostureObservation, PlatformPulseLifecycleObservation,
    PlatformPulseSemanticFocusPublished,
};

use crate::adjudication::{
    adjudicate_portal_control_point, adjudicate_runtime_service_story_pixels, portal_action_points,
    portal_occupancy_point, IntentControlPointFailure, PlatformPulseAuthoredPortalPixelEvidence,
    PlatformPulsePortalPixelEvidence, PlatformPulsePortalPixelFailure,
    PlatformPulseRuntimeServicePixelEvidence, PlatformPulseRuntimeServicePixelFailure,
    PlatformPulseWrappingTextFailure,
};
use crate::external_observation::PlatformPulseLifecycleStreamFailure;
use crate::failure_teardown::{
    teardown_native_bound_world, PulseExecutableWorldFailure, PulseExecutableWorldFailureReport,
};
use crate::native_platform::NativePlatformFailure;

use super::{
    await_next_observation, NativeBoundExecutableWorld, PortalReady, Published,
    PulseExecutableWorld, WatchedPulseObservationFailure, WatchedPulseTransition,
};

mod focus_observation;
mod input;
mod journey_failure;
mod modal_stack;
mod pixel_observation;
mod resize;
mod runtime_service_story;
mod source_rebind;
mod theme_switch;
use focus_observation::{
    await_portal_dismissed, await_semantic_focus, require_open_focus,
    require_restoration_after_rebind,
};
use input::{activate, escape, require_intent_quiet_after_occupancy_click};
use pixel_observation::{await_closed_pixels, await_open_pixels, capture, export_capture};

const TRANSITION_DEADLINE: Duration = Duration::from_secs(5);
const PIXEL_POLL_SLICE: Duration = Duration::from_millis(10);
const LIFECYCLE_IDLE_SLICE: Duration = Duration::from_millis(100);
#[derive(Debug)]
pub(crate) enum PlatformPulsePortalJourneyFailure {
    Native(NativePlatformFailure),
    ControlPoint(IntentControlPointFailure),
    Pixels(PlatformPulsePortalPixelFailure),
    TextClipping(PlatformPulseWrappingTextFailure),
    ServicePixels(PlatformPulseRuntimeServicePixelFailure),
    SourceAction(crate::source_delta::PulseSourceActionFailure),
    SourceDefinition(crate::source_delta::PulseSourceDeltaDefinitionFailure),
    Observation(WatchedPulseObservationFailure),
    FocusEvidence(&'static str),
    UnexpectedObservation(String),
    InputDelivery(&'static str),
    RuntimeServiceEvidence(&'static str),
    ModalStack(&'static str),
    CaptureExport(String),
}

pub(crate) struct PlatformPulsePortalJourneyEvidence {
    initial_open_pixels: PlatformPulsePortalPixelEvidence,
    resized_open_pixels: PlatformPulseAuthoredPortalPixelEvidence,
    runtime_service_pixels: PlatformPulseRuntimeServicePixelEvidence,
    runtime_service_story: runtime_service_story::PlatformPulseRuntimeServiceStoryEvidence,
    portal_rebind: source_rebind::PlatformPulsePortalRebindEvidence,
    open_focus: PlatformPulseSemanticFocusPublished,
    escape_close_focus: PlatformPulseSemanticFocusPublished,
    escape_dismissed_frame: u64,
    expected_shutdown_sequence: u64,
}

pub(crate) struct CompletedPlatformPulsePortalJourney {
    ready: PulseExecutableWorld<Published<PortalReady>>,
    evidence: PlatformPulsePortalJourneyEvidence,
}

impl PulseExecutableWorld<Published<PortalReady>> {
    pub(crate) fn complete_portal_journey(
        self,
    ) -> Result<CompletedPlatformPulsePortalJourney, PulseExecutableWorldFailureReport> {
        let Published { mut world, stage } = self.state;
        let result = complete(&mut world);
        match result {
            Ok(evidence) => Ok(CompletedPlatformPulsePortalJourney {
                ready: PulseExecutableWorld {
                    state: Published { world, stage },
                },
                evidence,
            }),
            Err(failure) => Err(teardown_native_bound_world(
                PulseExecutableWorldFailure::PortalJourney(failure),
                world.into_failure_resources(),
            )),
        }
    }
}

fn complete(
    world: &mut NativeBoundExecutableWorld,
) -> Result<PlatformPulsePortalJourneyEvidence, PlatformPulsePortalJourneyFailure> {
    theme_switch::exercise(world)?;
    let baseline = capture(world)?;
    let target = adjudicate_portal_control_point(&baseline)
        .map_err(PlatformPulsePortalJourneyFailure::ControlPoint)?;
    export_capture("01-main-default-960x600.png", &baseline)
        .map_err(PlatformPulsePortalJourneyFailure::CaptureExport)?;
    let application_command = runtime_service_story::exercise_application(world)?;

    activate(world, target.point())?;
    let first_open_focus = await_completed_portal_intent(world)?;
    require_open_focus(first_open_focus)?;
    let opened = await_open_pixels(world, &baseline)?;
    let runtime_service_story = runtime_service_story::exercise_portal(world, application_command)?;
    modal_stack::exercise(world, [960, 600])?;
    let open_capture = capture(world)?;
    let occupancy =
        portal_occupancy_point(&open_capture).map_err(PlatformPulsePortalJourneyFailure::Pixels)?;
    portal_action_points(&open_capture).map_err(PlatformPulsePortalJourneyFailure::Pixels)?;
    activate(world, occupancy)?;
    require_intent_quiet_after_occupancy_click(world)?;
    input::focus_next(world)?;
    let traversed = await_semantic_focus(world)?;
    focus_observation::require_traversal(first_open_focus, traversed)?;
    escape(world)?;
    let initial_dismissed = await_portal_dismissed(world)?;
    let initial_close_focus = await_semantic_focus(world)?;
    require_restoration_after_rebind(first_open_focus, initial_close_focus)?;
    if initial_dismissed.frame().diagnostic_value() != initial_close_focus.frame() {
        return Err(PlatformPulsePortalJourneyFailure::FocusEvidence(
            "Escape dismissal and Focus restoration did not share one publication frame",
        ));
    }
    let closed_after_service_story = await_closed_pixels(world, &baseline)?;
    let runtime_service_pixels =
        adjudicate_runtime_service_story_pixels(&baseline, &closed_after_service_story)
            .map_err(PlatformPulsePortalJourneyFailure::ServicePixels)?;
    let resized = resize::exercise(world)?;
    input::focus_next(world)?;
    let resized_traversed = await_semantic_focus(world)?;
    focus_observation::require_traversal(resized.root_focus(), resized_traversed)?;
    drain_until_idle(world)?;
    let before_rebind = capture(world)?;
    let portal_rebind = source_rebind::exercise(world, &before_rebind, resized_traversed)?;
    escape(world)?;
    let dismissed = await_portal_dismissed(world)?;
    let escape_close_focus = await_semantic_focus(world)?;
    require_restoration_after_rebind(resized.root_focus(), escape_close_focus)?;
    if dismissed.frame().diagnostic_value() != escape_close_focus.frame() {
        return Err(PlatformPulsePortalJourneyFailure::FocusEvidence(
            "resized Escape dismissal and Focus restoration did not share one publication frame",
        ));
    }
    await_closed_pixels(world, resized.closed_baseline())?;
    let expected_shutdown_sequence = drain_until_idle(world)?;

    Ok(PlatformPulsePortalJourneyEvidence {
        initial_open_pixels: opened,
        resized_open_pixels: resized.pixels(),
        runtime_service_pixels,
        runtime_service_story,
        portal_rebind,
        open_focus: first_open_focus,
        escape_close_focus,
        escape_dismissed_frame: dismissed.frame().diagnostic_value(),
        expected_shutdown_sequence,
    })
}

fn await_completed_portal_intent(
    world: &mut NativeBoundExecutableWorld,
) -> Result<PlatformPulseSemanticFocusPublished, PlatformPulsePortalJourneyFailure> {
    let admitted = await_admitted(world)?;
    if await_executor_or_completion(world, admitted)? {
        await_completed(world, admitted)?;
    }
    await_semantic_focus(world)
}

fn await_admitted(
    world: &mut NativeBoundExecutableWorld,
) -> Result<PlatformPulseIntentAttemptObservationReference, PlatformPulsePortalJourneyFailure> {
    loop {
        let envelope = next(world, WatchedPulseTransition::IntentPosturePublished)?;
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::IntentPosturePublished(posture) => {
                if let PlatformPulseIntentPostureObservation::Admitted { reference } =
                    posture.posture()
                {
                    return Ok(*reference);
                }
                return Err(unexpected(envelope.outcome()));
            }
            outcome if incidental_visual(outcome) => {}
            outcome => return Err(unexpected(outcome)),
        }
    }
}

fn await_executor_or_completion(
    world: &mut NativeBoundExecutableWorld,
    admitted: PlatformPulseIntentAttemptObservationReference,
) -> Result<bool, PlatformPulsePortalJourneyFailure> {
    loop {
        let envelope = next(world, WatchedPulseTransition::IntentExecutorStarted)?;
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::IntentExecutorStarted(started)
                if started.reference() == admitted =>
            {
                return Ok(true)
            }
            PlatformPulseLifecycleObservation::IntentPosturePublished(posture)
                if matches!(
                    posture.posture(),
                    PlatformPulseIntentPostureObservation::Completed { reference }
                        if *reference == admitted
                ) =>
            {
                return Ok(false)
            }
            outcome if incidental_visual(outcome) => {}
            outcome => return Err(unexpected(outcome)),
        }
    }
}

fn await_completed(
    world: &mut NativeBoundExecutableWorld,
    admitted: PlatformPulseIntentAttemptObservationReference,
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    loop {
        let envelope = next(world, WatchedPulseTransition::IntentPosturePublished)?;
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::IntentPosturePublished(posture)
                if matches!(
                    posture.posture(),
                    PlatformPulseIntentPostureObservation::Completed { reference }
                        if *reference == admitted
                ) =>
            {
                return Ok(())
            }
            outcome if incidental_visual(outcome) => {}
            outcome => return Err(unexpected(outcome)),
        }
    }
}

fn next(
    world: &mut NativeBoundExecutableWorld,
    expected: WatchedPulseTransition,
) -> Result<
    worth_ui_platform_pulse::observation_contract::PlatformPulseLifecycleObservationEnvelope,
    PlatformPulsePortalJourneyFailure,
> {
    await_next_observation(
        &mut world.process,
        &mut world.lifecycle,
        expected,
        Instant::now() + TRANSITION_DEADLINE,
    )
    .map_err(PlatformPulsePortalJourneyFailure::Observation)
}

fn incidental_visual(outcome: &PlatformPulseLifecycleObservation) -> bool {
    matches!(
        outcome,
        PlatformPulseLifecycleObservation::VisualSnapshotCaptured(_)
            | PlatformPulseLifecycleObservation::VisualPointTrace(_)
            | PlatformPulseLifecycleObservation::VisualOverlayPublished(_)
            | PlatformPulseLifecycleObservation::VisualOverlayCleared(_)
            | PlatformPulseLifecycleObservation::VisualSnapshotRetired(_)
            | PlatformPulseLifecycleObservation::VisualComparison(_)
    )
}

fn unexpected(outcome: &PlatformPulseLifecycleObservation) -> PlatformPulsePortalJourneyFailure {
    PlatformPulsePortalJourneyFailure::UnexpectedObservation(format!("{outcome:?}"))
}

fn drain_until_idle(
    world: &mut NativeBoundExecutableWorld,
) -> Result<u64, PlatformPulsePortalJourneyFailure> {
    loop {
        match world.lifecycle.next(Instant::now() + LIFECYCLE_IDLE_SLICE) {
            Ok(envelope) if incidental_visual(envelope.outcome()) => {}
            Ok(envelope) => return Err(unexpected(envelope.outcome())),
            Err(PlatformPulseLifecycleStreamFailure::Deadline) => {
                return Ok(world.lifecycle.measurement().accepted_events() as u64 + 1)
            }
            Err(failure) => {
                return Err(PlatformPulsePortalJourneyFailure::Observation(
                    WatchedPulseObservationFailure::Lifecycle(failure),
                ))
            }
        }
    }
}

impl CompletedPlatformPulsePortalJourney {
    pub(crate) fn evidence(&self) -> &PlatformPulsePortalJourneyEvidence {
        &self.evidence
    }

    pub(crate) fn into_ready(self) -> PulseExecutableWorld<Published<PortalReady>> {
        self.ready
    }
}

impl PlatformPulsePortalJourneyEvidence {
    pub(crate) const fn focus_before_rebind(&self) -> PlatformPulseSemanticFocusPublished {
        self.portal_rebind.focused_before_edit()
    }

    pub(crate) const fn initial_open_pixels(&self) -> PlatformPulsePortalPixelEvidence {
        self.initial_open_pixels
    }

    pub(crate) const fn resized_open_pixels(&self) -> PlatformPulseAuthoredPortalPixelEvidence {
        self.resized_open_pixels
    }

    pub(crate) const fn runtime_service_changed_pixels(&self) -> [usize; 2] {
        [
            self.runtime_service_pixels.command_story_changed_pixels(),
            self.runtime_service_pixels
                .query_denial_story_changed_pixels(),
        ]
    }

    pub(crate) const fn runtime_service_query_posture_changed_pixels(&self) -> usize {
        self.runtime_service_pixels.query_posture_changed_pixels()
    }

    pub(crate) const fn runtime_service_sequences(&self) -> [u64; 5] {
        self.runtime_service_story.sequences()
    }

    pub(crate) const fn runtime_service_query_revisions(&self) -> [u64; 2] {
        [
            self.runtime_service_story.active_query_source_revision(),
            self.runtime_service_story.submitted_query_source_revision(),
        ]
    }

    pub(crate) fn runtime_service_command_transitions(
        &self,
    ) -> [&worth_ui_platform_pulse::observation_contract::PlatformPulseCommandTransitionInspection; 2]
    {
        self.runtime_service_story.command_transitions()
    }

    pub(crate) const fn expected_shutdown_sequence(&self) -> u64 {
        self.expected_shutdown_sequence
    }

    pub(crate) const fn focus_publications(&self) -> [PlatformPulseSemanticFocusPublished; 2] {
        [self.open_focus, self.escape_close_focus]
    }

    pub(crate) const fn focus_rebind_inspection(&self) -> PlatformPulseFocusTransitionInspection {
        self.portal_rebind.focus_transition()
    }

    pub(crate) const fn portal_rebind_sequence(&self) -> u64 {
        self.portal_rebind.replacement_sequence()
    }

    pub(crate) const fn portal_rebind_pixels(&self) -> [usize; 2] {
        let pixels = self.portal_rebind.pixels();
        [
            pixels.removed_primary_pixels(),
            pixels.fallback_action_pixels(),
        ]
    }

    pub(crate) const fn escape_dismissed_frame(&self) -> u64 {
        self.escape_dismissed_frame
    }
}
