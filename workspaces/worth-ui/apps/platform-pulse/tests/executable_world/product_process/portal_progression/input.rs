use std::time::{Duration, Instant};

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseIntentInteractionFamily, PlatformPulseIntentRoutingStoppedObservation,
    PlatformPulseLifecycleObservation,
};

use crate::external_observation::PlatformPulseLifecycleStreamFailure;
use crate::external_observation::{NativeInputProbeKind, NativeKeyboardCommand};
use crate::native_platform::NativePlatformContract;

use super::{
    incidental_visual, unexpected, NativeBoundExecutableWorld, PlatformPulsePortalJourneyFailure,
    WatchedPulseObservationFailure,
};

pub(super) fn activate(
    world: &mut NativeBoundExecutableWorld,
    point: crate::external_observation::NativeClientPixelPoint,
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    let delivery = world
        .platform
        .deliver_pointer_activation(&world.native_client, point)
        .map_err(PlatformPulsePortalJourneyFailure::Native)?;
    require_delivery(
        delivery,
        NativeInputProbeKind::Pointer,
        2,
        world.process.id(),
    )
}

pub(super) fn escape(
    world: &mut NativeBoundExecutableWorld,
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    keyboard(world, NativeKeyboardCommand::Escape)
}

pub(super) fn run_platform_command(
    world: &mut NativeBoundExecutableWorld,
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    keyboard(world, NativeKeyboardCommand::PrimaryShiftP)
}

pub(super) fn focus_next(
    world: &mut NativeBoundExecutableWorld,
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    keyboard(world, NativeKeyboardCommand::Tab)
}

pub(super) fn require_intent_quiet_after_occupancy_click(
    world: &mut NativeBoundExecutableWorld,
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    let deadline = Instant::now() + Duration::from_millis(500);
    loop {
        match world.lifecycle.next(deadline) {
            Ok(envelope) if incidental_visual(envelope.outcome()) => {}
            Ok(envelope)
                if matches!(
                    envelope.outcome(),
                    PlatformPulseLifecycleObservation::IntentRoutingStopped(
                        PlatformPulseIntentRoutingStoppedObservation::Unrouted {
                            graph_node,
                            interaction: PlatformPulseIntentInteractionFamily::Activate,
                        },
                    ) if *graph_node != 0
                ) =>
            {
                return Ok(())
            }
            Ok(envelope) => return Err(unexpected(envelope.outcome())),
            Err(PlatformPulseLifecycleStreamFailure::Deadline) => return Ok(()),
            Err(failure) => {
                return Err(PlatformPulsePortalJourneyFailure::Observation(
                    WatchedPulseObservationFailure::Lifecycle(failure),
                ))
            }
        }
    }
}

fn keyboard(
    world: &mut NativeBoundExecutableWorld,
    command: NativeKeyboardCommand,
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    let delivery = world
        .platform
        .deliver_keyboard_command(&world.native_client, command)
        .map_err(PlatformPulsePortalJourneyFailure::Native)?;
    let expected_event_count = match command {
        NativeKeyboardCommand::Escape | NativeKeyboardCommand::Tab => 2,
        NativeKeyboardCommand::PrimaryShiftP => 6,
    };
    require_delivery(
        delivery,
        NativeInputProbeKind::Keyboard,
        expected_event_count,
        world.process.id(),
    )
}

fn require_delivery(
    delivery: crate::external_observation::NativeInputDeliveryObservation,
    expected: NativeInputProbeKind,
    expected_event_count: u32,
    expected_process_id: u32,
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    if delivery.kind() == expected
        && delivery.delivered_event_count() == expected_event_count
        && delivery.process_id() == expected_process_id
    {
        Ok(())
    } else {
        Err(PlatformPulsePortalJourneyFailure::InputDelivery(
            "native command did not preserve its exact OS event sequence",
        ))
    }
}
