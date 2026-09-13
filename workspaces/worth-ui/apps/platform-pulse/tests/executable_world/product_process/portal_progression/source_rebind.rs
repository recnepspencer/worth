use std::time::Instant;

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseFocusTransitionInspection, PlatformPulseLifecycleObservation,
    PlatformPulseSemanticFocusCause, PlatformPulseSemanticFocusOutcome,
    PlatformPulseSemanticFocusPublished,
};

use crate::adjudication::{
    adjudicate_focus_fallback_portal_pixels, PlatformPulsePortalFocusFallbackPixelEvidence,
};
use crate::installation::CanonicalPlatformPulse;
use crate::source_delta::{PortalFocusFallbackSourceDelta, PulseSourceDeltaIdentity};

use super::{
    capture, incidental_visual, next, unexpected, NativeBoundExecutableWorld,
    PlatformPulsePortalJourneyFailure, WatchedPulseTransition, PIXEL_POLL_SLICE,
    TRANSITION_DEADLINE,
};

pub(super) struct PlatformPulsePortalRebindEvidence {
    focused_before_edit: PlatformPulseSemanticFocusPublished,
    replacement_sequence: u64,
    focus_transition: PlatformPulseFocusTransitionInspection,
    pixels: PlatformPulsePortalFocusFallbackPixelEvidence,
}

pub(super) fn exercise(
    world: &mut NativeBoundExecutableWorld,
    before: &crate::external_observation::NativeClientPixelCapture,
    opened_focus: PlatformPulseSemanticFocusPublished,
) -> Result<PlatformPulsePortalRebindEvidence, PlatformPulsePortalJourneyFailure> {
    let delta =
        PortalFocusFallbackSourceDelta::from_checked_in(CanonicalPlatformPulse::checked_in())
            .map_err(PlatformPulsePortalJourneyFailure::SourceDefinition)?;
    let action = delta
        .apply(&world.installation)
        .map_err(PlatformPulsePortalJourneyFailure::SourceAction)?;
    if action.identity() != PulseSourceDeltaIdentity::PortalFocusFallback
        || action.action_count() != 1
        || action.written_bytes() == 0
        || action.content_fingerprint() == 0
        || action.entry_source() != world.installation.portal_primary_source()
    {
        return Err(PlatformPulsePortalJourneyFailure::RuntimeServiceEvidence(
            "portal focus fallback was not one exact external source action",
        ));
    }
    let (replacement_sequence, focus_transition) = await_replacement(world, opened_focus)?;
    let pixels = await_fallback_pixels(world, before, [1_120, 700])?;
    Ok(PlatformPulsePortalRebindEvidence {
        focused_before_edit: opened_focus,
        replacement_sequence,
        focus_transition,
        pixels,
    })
}

fn await_replacement(
    world: &mut NativeBoundExecutableWorld,
    opened: PlatformPulseSemanticFocusPublished,
) -> Result<(u64, PlatformPulseFocusTransitionInspection), PlatformPulsePortalJourneyFailure> {
    loop {
        let envelope = next(world, WatchedPulseTransition::GreenReplacement)?;
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::RebindPublished(replacement) => {
                let Some(focus) = replacement.latest_focus_transition() else {
                    return Err(PlatformPulsePortalJourneyFailure::FocusEvidence(
                        "source replacement did not expose its focus transition inspection",
                    ));
                };
                if replacement.actual_native_effect_count() == 0
                    || replacement.schema_transition().is_some()
                    || focus.cause() != PlatformPulseSemanticFocusCause::RebindFallback
                    || focus.previous_mounted_instance().is_none()
                    || focus.previous_mounted_instance()
                        != opened
                            .current()
                            .map(|participant| participant.mounted_instance())
                    || focus.current_mounted_instance().is_none()
                    || focus.current_mounted_instance() == focus.previous_mounted_instance()
                    || focus.outcome() != PlatformPulseSemanticFocusOutcome::Moved
                    || focus.participants_visited() == 0
                    || focus.revision() <= opened.revision()
                {
                    return Err(PlatformPulsePortalJourneyFailure::FocusEvidence(
                        "source replacement focus inspection did not prove a real fallback",
                    ));
                }
                return Ok((envelope.sequence().value(), focus));
            }
            outcome if incidental_visual(outcome) => {}
            outcome => return Err(unexpected(outcome)),
        }
    }
}

fn await_fallback_pixels(
    world: &mut NativeBoundExecutableWorld,
    before: &crate::external_observation::NativeClientPixelCapture,
    logical_client_extent: [u32; 2],
) -> Result<PlatformPulsePortalFocusFallbackPixelEvidence, PlatformPulsePortalJourneyFailure> {
    let deadline = Instant::now() + TRANSITION_DEADLINE;
    loop {
        let after = capture(world)?;
        if let Ok(evidence) =
            adjudicate_focus_fallback_portal_pixels(before, &after, logical_client_extent)
        {
            return Ok(evidence);
        }
        if Instant::now() >= deadline {
            return adjudicate_focus_fallback_portal_pixels(before, &after, logical_client_extent)
                .map_err(PlatformPulsePortalJourneyFailure::Pixels);
        }
        std::thread::sleep(PIXEL_POLL_SLICE);
    }
}

impl PlatformPulsePortalRebindEvidence {
    pub(crate) const fn focused_before_edit(&self) -> PlatformPulseSemanticFocusPublished {
        self.focused_before_edit
    }

    pub(crate) const fn replacement_sequence(&self) -> u64 {
        self.replacement_sequence
    }

    pub(crate) const fn pixels(&self) -> PlatformPulsePortalFocusFallbackPixelEvidence {
        self.pixels
    }

    pub(crate) const fn focus_transition(&self) -> PlatformPulseFocusTransitionInspection {
        self.focus_transition
    }
}
