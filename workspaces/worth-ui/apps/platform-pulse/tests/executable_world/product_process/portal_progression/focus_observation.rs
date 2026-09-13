use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseLifecycleObservation, PlatformPulsePortalDismissed,
    PlatformPulseSemanticFocusCause, PlatformPulseSemanticFocusPhysicalOutcome,
    PlatformPulseSemanticFocusPublished,
};

use super::{
    incidental_visual, next, unexpected, NativeBoundExecutableWorld,
    PlatformPulsePortalJourneyFailure, WatchedPulseTransition,
};

pub(super) fn await_semantic_focus(
    world: &mut NativeBoundExecutableWorld,
) -> Result<PlatformPulseSemanticFocusPublished, PlatformPulsePortalJourneyFailure> {
    loop {
        let envelope = next(world, WatchedPulseTransition::SemanticFocusPublished)?;
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::SemanticFocusPublished(focus) => return Ok(*focus),
            outcome if incidental_visual(outcome) => {}
            outcome => return Err(unexpected(outcome)),
        }
    }
}

pub(super) fn await_portal_dismissed(
    world: &mut NativeBoundExecutableWorld,
) -> Result<PlatformPulsePortalDismissed, PlatformPulsePortalJourneyFailure> {
    loop {
        let envelope = next(world, WatchedPulseTransition::PortalDismissed)?;
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::PortalDismissed(dismissed) => return Ok(*dismissed),
            outcome if incidental_visual(outcome) => {}
            outcome => return Err(unexpected(outcome)),
        }
    }
}

pub(super) fn require_open_focus(
    focus: PlatformPulseSemanticFocusPublished,
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    if focus.cause() != PlatformPulseSemanticFocusCause::PortalInitial {
        return Err(focus_failure(
            "portal open did not publish PortalInitial Focus cause",
        ));
    }
    if focus.current().is_none() || focus.host_request().is_none() {
        return Err(focus_failure(
            "portal open omitted semantic or native Focus identity",
        ));
    }
    if focus.physical_outcome() != PlatformPulseSemanticFocusPhysicalOutcome::Applied {
        return Err(focus_failure(
            "portal open native Focus placement was not applied",
        ));
    }
    Ok(())
}

pub(super) fn require_traversal(
    opened: PlatformPulseSemanticFocusPublished,
    traversed: PlatformPulseSemanticFocusPublished,
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    if traversed.cause() != PlatformPulseSemanticFocusCause::KeyboardTraversal
        || traversed.outcome() != worth_ui_platform_pulse::observation_contract::PlatformPulseSemanticFocusOutcome::Moved
        || traversed.physical_outcome() != PlatformPulseSemanticFocusPhysicalOutcome::Applied
        || traversed.previous() != opened.current()
        || traversed.current().is_none()
        || traversed.current() == traversed.previous()
        || traversed.revision() <= opened.revision()
    {
        return Err(focus_failure("native Tab did not publish the exact Focus traversal"));
    }
    Ok(())
}

pub(super) fn require_restoration_after_rebind(
    opened: PlatformPulseSemanticFocusPublished,
    restored: PlatformPulseSemanticFocusPublished,
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    if restored.cause() != PlatformPulseSemanticFocusCause::PortalRestoration
        || restored.current() != opened.previous()
    {
        return Err(focus_failure(
            "portal close after fallback did not restore the exact pre-open participant",
        ));
    }
    let expected_physical = if restored.current().is_some() {
        PlatformPulseSemanticFocusPhysicalOutcome::Applied
    } else {
        PlatformPulseSemanticFocusPhysicalOutcome::Cleared
    };
    if restored.physical_outcome() != expected_physical {
        return Err(focus_failure(
            "portal close after fallback carried the wrong physical Focus outcome",
        ));
    }
    Ok(())
}

fn focus_failure(message: &'static str) -> PlatformPulsePortalJourneyFailure {
    PlatformPulsePortalJourneyFailure::FocusEvidence(message)
}
