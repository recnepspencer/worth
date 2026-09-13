use worth_ui_platform_pulse::observation_contract::PlatformPulseLifecycleObservation;

use crate::adjudication::{
    adjudicate_confirmation_control_point, adjudicate_visible_control_change,
    require_distinct_control_points, NativeControlPixelRegion, PlatformPulseActionControlPoint,
    PlatformPulseConfirmationControlPoint, VisibleControlPixelChange,
};
use crate::external_observation::NativeClientPixelCapture;
use crate::native_platform::NativePlatformContract;
use crate::product_process::{NativeBoundExecutableWorld, WatchedPulseTransition};

use super::{next, IntentObservationFailure};

pub(in crate::product_process::intent_progression) fn await_visual_refresh(
    world: &mut NativeBoundExecutableWorld,
) -> Result<u64, IntentObservationFailure> {
    let retired = next(world, WatchedPulseTransition::IntentVisualRefreshRetired)?;
    if !matches!(
        retired.outcome(),
        PlatformPulseLifecycleObservation::VisualSnapshotRetired(_)
    ) {
        return Err(super::unexpected(
            "retired predecessor visual snapshot",
            retired.outcome(),
        ));
    }
    let captured = next(world, WatchedPulseTransition::IntentVisualRefreshCaptured)?;
    if !matches!(
        captured.outcome(),
        PlatformPulseLifecycleObservation::VisualSnapshotCaptured(_)
    ) {
        return Err(super::unexpected(
            "captured successor visual snapshot",
            captured.outcome(),
        ));
    }
    Ok(captured.sequence().value())
}

pub(in crate::product_process::intent_progression) fn capture_visible_change(
    world: &mut NativeBoundExecutableWorld,
    baseline: &NativeClientPixelCapture,
    region: NativeControlPixelRegion,
) -> Result<VisibleControlPixelChange, IntentObservationFailure> {
    let current = world
        .platform
        .capture_client_area(&world.native_client)
        .map_err(IntentObservationFailure::Native)?;
    adjudicate_visible_control_change(baseline, &current, region)
        .map_err(IntentObservationFailure::Visible)
}

pub(in crate::product_process::intent_progression) fn capture_visible_confirmation(
    world: &mut NativeBoundExecutableWorld,
    baseline: &NativeClientPixelCapture,
    action: PlatformPulseActionControlPoint,
) -> Result<
    (
        PlatformPulseConfirmationControlPoint,
        VisibleControlPixelChange,
    ),
    IntentObservationFailure,
> {
    let current = world
        .platform
        .capture_client_area(&world.native_client)
        .map_err(IntentObservationFailure::Native)?;
    let confirmation = adjudicate_confirmation_control_point(&current)
        .map_err(IntentObservationFailure::Visible)?;
    require_distinct_control_points(action, confirmation)
        .map_err(IntentObservationFailure::Visible)?;
    let change = adjudicate_visible_control_change(baseline, &current, confirmation.region())
        .map_err(IntentObservationFailure::Visible)?;
    Ok((confirmation, change))
}
