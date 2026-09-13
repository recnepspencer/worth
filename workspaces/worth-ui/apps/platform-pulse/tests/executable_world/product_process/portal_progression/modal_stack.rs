use crate::external_observation::NativeClientPixelPoint;
use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseCommandLossReasonInspection, PlatformPulseCommandScopeInspection,
    PlatformPulseIntentPostureObservation, PlatformPulseLifecycleObservation,
    PlatformPulseQueryActionObservation,
};

use super::{
    activate, await_completed_portal_intent, await_portal_dismissed, await_semantic_focus, capture,
    escape, export_capture, incidental_visual, next, require_open_focus,
    require_restoration_after_rebind, unexpected, NativeBoundExecutableWorld,
    PlatformPulsePortalJourneyFailure, WatchedPulseTransition,
};

mod pixels;
use pixels::await_pixels;

pub(super) fn exercise(
    world: &mut NativeBoundExecutableWorld,
    logical_extent: [u32; 2],
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    let parent = capture(world)?;
    crate::adjudication::adjudicate_authored_portal_pixels(&parent, logical_extent)
        .map_err(PlatformPulsePortalJourneyFailure::Pixels)?;
    let [width, height] = logical_extent;
    export_capture(&format!("03-parent-modal-{width}x{height}.png"), &parent)
        .map_err(PlatformPulsePortalJourneyFailure::CaptureExport)?;
    // Authored logical geometry: details [528,88..808,408], with the
    // Review action [552,248..784,288]. Input comes from the native client.
    let review = NativeClientPixelPoint::interior(
        &parent,
        668 * parent.width() / width,
        268 * parent.height() / height,
        2,
    )
    .ok_or(PlatformPulsePortalJourneyFailure::ModalStack(
        "Review point outside native client",
    ))?;
    activate(world, review)?;
    let opened = await_completed_portal_intent(world)?;
    require_open_focus(opened)?;
    if opened.current() == opened.previous() {
        return Err(PlatformPulsePortalJourneyFailure::ModalStack(
            "nested dialog did not acquire its own Focus",
        ));
    }
    await_pixels(world, true, &parent, logical_extent)?;
    if logical_extent == [1_120, 700] {
        // This is the center of the underlying Run action, outside both dialogs.
        // The native click must reach the process while the modal Backdrop prevents
        // that application action from reaching an executable intent route.
        let underlying_action = NativeClientPixelPoint::interior(
            &parent,
            404 * parent.width() / width,
            440 * parent.height() / height,
            2,
        )
        .ok_or(PlatformPulsePortalJourneyFailure::ModalStack(
            "underlying application action lies outside the native client",
        ))?;
        activate(world, underlying_action)?;
        require_underlying_activation_shielded(world)?;
    }
    escape(world)?;
    let dismissed = await_portal_dismissed(world)?;
    let restored = await_semantic_focus(world)?;
    require_restoration_after_rebind(opened, restored)?;
    if dismissed.frame().diagnostic_value() != restored.frame() {
        return Err(PlatformPulsePortalJourneyFailure::ModalStack(
            "nested dismissal and Focus restoration differed in frame",
        ));
    }
    await_pixels(world, false, &parent, logical_extent)
}

fn require_underlying_activation_shielded(
    world: &mut NativeBoundExecutableWorld,
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    let admitted = loop {
        let envelope = next(world, WatchedPulseTransition::IntentPosturePublished)?;
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::IntentPosturePublished(posture) => {
                let PlatformPulseIntentPostureObservation::Admitted { reference } =
                    posture.posture()
                else {
                    return Err(unexpected(envelope.outcome()));
                };
                let command = posture.latest_command_transition().ok_or(
                    PlatformPulsePortalJourneyFailure::ModalStack(
                        "shielded activation omitted its command-routing evidence",
                    ),
                )?;
                let losing = command.losing_candidate().ok_or(
                    PlatformPulsePortalJourneyFailure::ModalStack(
                        "shielded activation omitted the underlying application candidate",
                    ),
                )?;
                if command.winner() != "platform.pulse.command.run.portal"
                    || command.scope() != PlatformPulseCommandScopeInspection::ActivePortal
                    || losing.command() != "platform.pulse.command.run.application"
                    || losing.reason()
                        != PlatformPulseCommandLossReasonInspection::LowerScopePrecedence
                {
                    return Err(PlatformPulsePortalJourneyFailure::ModalStack(
                        "nested modal did not relationally shield the underlying application action",
                    ));
                }
                break *reference;
            }
            outcome if incidental_visual(outcome) => {}
            outcome => return Err(unexpected(outcome)),
        }
    };
    loop {
        let envelope = next(world, WatchedPulseTransition::IntentPosturePublished)?;
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::IntentExecutorStarted(started)
                if started.reference() == admitted => {}
            PlatformPulseLifecycleObservation::QueryAction(
                PlatformPulseQueryActionObservation::Denied { reference, .. },
            ) if *reference == admitted => {}
            PlatformPulseLifecycleObservation::IntentPosturePublished(posture)
                if matches!(
                    posture.posture(),
                    PlatformPulseIntentPostureObservation::Denied
                ) =>
            {
                let command =
                    posture
                        .latest_command_transition()
                        .ok_or(PlatformPulsePortalJourneyFailure::ModalStack(
                        "shielded activation lost its relational command evidence at settlement",
                    ))?;
                if command.winner() != "platform.pulse.command.run.portal"
                    || command.scope() != PlatformPulseCommandScopeInspection::ActivePortal
                {
                    return Err(PlatformPulsePortalJourneyFailure::ModalStack(
                        "shielded activation settled through the underlying application command",
                    ));
                }
                return Ok(());
            }
            outcome if incidental_visual(outcome) => {}
            outcome => return Err(unexpected(outcome)),
        }
    }
}
