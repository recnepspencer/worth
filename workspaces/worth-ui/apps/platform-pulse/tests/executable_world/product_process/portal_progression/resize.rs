use std::time::Instant;

use worth_ui_platform_pulse::observation_contract::PlatformPulseLifecycleObservation;

use crate::adjudication::{
    adjudicate_authored_portal_pixels, adjudicate_portal_control_point_for_extent,
    adjudicate_resized_wrapping_text, PlatformPulseAuthoredPortalPixelEvidence,
};
use crate::native_platform::{NativePlatformContract, NativePlatformFailure};

use super::{
    activate, await_completed_portal_intent, capture, export_capture, next, require_open_focus,
    NativeBoundExecutableWorld, PlatformPulsePortalJourneyFailure, WatchedPulseTransition,
    PIXEL_POLL_SLICE, TRANSITION_DEADLINE,
};

const RESIZED_EXTENT: [u32; 2] = [1_120, 700];

pub(super) struct ResizedPortalEvidence {
    closed_baseline: crate::external_observation::NativeClientPixelCapture,
    pixels: PlatformPulseAuthoredPortalPixelEvidence,
    root_focus: worth_ui_platform_pulse::observation_contract::PlatformPulseSemanticFocusPublished,
}

pub(super) fn exercise(
    world: &mut NativeBoundExecutableWorld,
) -> Result<ResizedPortalEvidence, PlatformPulsePortalJourneyFailure> {
    let observed = world
        .platform
        .observe_bound_client_area(&world.native_client)
        .map_err(PlatformPulsePortalJourneyFailure::Native)?;
    let resized_physical = project_extent(RESIZED_EXTENT, observed.dpi());
    let deadline = Instant::now() + TRANSITION_DEADLINE;
    world
        .platform
        .resize_bound_client_area(&mut world.native_client, resized_physical, deadline)
        .map_err(PlatformPulsePortalJourneyFailure::Native)?;
    await_presented_extent(world, resized_physical)?;
    let (closed_baseline, target) = await_closed_control(world, resized_physical)?;
    activate(world, target)?;
    let root_focus = await_completed_portal_intent(world)?;
    require_open_focus(root_focus)?;
    let resized = await_authored(
        world,
        RESIZED_EXTENT,
        resized_physical,
        Instant::now() + TRANSITION_DEADLINE,
    )?;
    super::modal_stack::exercise(world, RESIZED_EXTENT)?;
    Ok(ResizedPortalEvidence {
        closed_baseline,
        pixels: resized,
        root_focus,
    })
}

impl ResizedPortalEvidence {
    pub(super) const fn closed_baseline(
        &self,
    ) -> &crate::external_observation::NativeClientPixelCapture {
        &self.closed_baseline
    }

    pub(super) const fn pixels(&self) -> PlatformPulseAuthoredPortalPixelEvidence {
        self.pixels
    }

    pub(super) const fn root_focus(
        &self,
    ) -> worth_ui_platform_pulse::observation_contract::PlatformPulseSemanticFocusPublished {
        self.root_focus
    }
}

fn await_closed_control(
    world: &mut NativeBoundExecutableWorld,
    physical_extent: [u32; 2],
) -> Result<
    (
        crate::external_observation::NativeClientPixelCapture,
        crate::external_observation::NativeClientPixelPoint,
    ),
    PlatformPulsePortalJourneyFailure,
> {
    let deadline = Instant::now() + TRANSITION_DEADLINE;
    loop {
        let closed = match capture(world) {
            Ok(closed) => closed,
            Err(PlatformPulsePortalJourneyFailure::Native(
                NativePlatformFailure::ClientCapture(_),
            )) if Instant::now() < deadline => {
                std::thread::sleep(PIXEL_POLL_SLICE);
                continue;
            }
            Err(failure) => return Err(failure),
        };
        if [closed.width(), closed.height()] == physical_extent {
            if let Ok(target) = adjudicate_portal_control_point_for_extent(&closed, RESIZED_EXTENT)
            {
                return Ok((closed, target.point()));
            }
        }
        if Instant::now() >= deadline {
            let closed = capture(world)?;
            let target = adjudicate_portal_control_point_for_extent(&closed, RESIZED_EXTENT)
                .map_err(PlatformPulsePortalJourneyFailure::ControlPoint)?;
            return Ok((closed, target.point()));
        }
        std::thread::sleep(PIXEL_POLL_SLICE);
    }
}

fn await_presented_extent(
    world: &mut NativeBoundExecutableWorld,
    expected: [u32; 2],
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    loop {
        let envelope = next(world, WatchedPulseTransition::VisualSnapshot)?;
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::VisualSnapshotCaptured(snapshot)
                if snapshot.coordinates().client_physical_dimensions() == expected =>
            {
                return Ok(())
            }
            PlatformPulseLifecycleObservation::VisualSnapshotCaptured(_)
            | PlatformPulseLifecycleObservation::VisualPointTrace(_)
            | PlatformPulseLifecycleObservation::VisualOverlayPublished(_)
            | PlatformPulseLifecycleObservation::VisualOverlayCleared(_)
            | PlatformPulseLifecycleObservation::VisualSnapshotRetired(_)
            | PlatformPulseLifecycleObservation::VisualComparison(_) => {}
            outcome => return Err(super::unexpected(outcome)),
        }
    }
}

fn await_authored(
    world: &mut NativeBoundExecutableWorld,
    logical_extent: [u32; 2],
    physical_extent: [u32; 2],
    deadline: Instant,
) -> Result<PlatformPulseAuthoredPortalPixelEvidence, PlatformPulsePortalJourneyFailure> {
    loop {
        let current = capture(world)?;
        if [current.width(), current.height()] == physical_extent {
            if let Ok(evidence) = adjudicate_authored_portal_pixels(&current, logical_extent) {
                if adjudicate_resized_wrapping_text(&current).is_ok() {
                    export_resized_capture(&current, logical_extent)?;
                    return Ok(evidence);
                }
            }
        }
        if Instant::now() >= deadline {
            if [current.width(), current.height()] != physical_extent {
                return Err(PlatformPulsePortalJourneyFailure::Pixels(
                    crate::adjudication::PlatformPulsePortalPixelFailure::CaptureMismatch,
                ));
            }
            let evidence = adjudicate_authored_portal_pixels(&current, logical_extent)
                .map_err(PlatformPulsePortalJourneyFailure::Pixels)?;
            adjudicate_resized_wrapping_text(&current)
                .map_err(PlatformPulsePortalJourneyFailure::TextClipping)?;
            export_resized_capture(&current, logical_extent)?;
            return Ok(evidence);
        }
        std::thread::sleep(PIXEL_POLL_SLICE);
    }
}

fn export_resized_capture(
    capture: &crate::external_observation::NativeClientPixelCapture,
    logical_extent: [u32; 2],
) -> Result<(), PlatformPulsePortalJourneyFailure> {
    if logical_extent == RESIZED_EXTENT {
        export_capture("02-portal-resized-1120x700.png", capture)
            .map_err(PlatformPulsePortalJourneyFailure::CaptureExport)?;
    }
    Ok(())
}

fn project_extent(logical: [u32; 2], dpi: u32) -> [u32; 2] {
    logical.map(|value| ((u64::from(value) * u64::from(dpi) + 48) / 96) as u32)
}
