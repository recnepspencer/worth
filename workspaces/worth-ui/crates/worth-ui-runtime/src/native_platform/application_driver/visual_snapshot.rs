use std::time::{Duration, Instant};

use crate::facade::WorthUiNativeApplicationShell;
use crate::inspection::mounted_frame::{UiMountedInspectionReceipt, UiMountedInspectionRequest};
use crate::inspection::visual_snapshot::{
    UiVisualCapturePoll, UiVisualSnapshotOutcome, UiVisualSnapshotReceipt,
};
use worth_ui_inspection::{UiPixelsRequired, UiVisualCaptureDeadline, UiVisualSnapshotRequest};

const POLL_TICK_CAPACITY: u64 = 1_024;
const WALL_DEADLINE: Duration = Duration::from_secs(5);

pub(super) fn capture_presented_source(
    shell: &mut WorthUiNativeApplicationShell,
    tick: &mut u64,
) -> Result<worth_ui_host_native::UiNativeClientVisualSnapshotObservation, ()> {
    let frame = match shell.inspect_mounted_frame(UiMountedInspectionRequest::current()) {
        UiMountedInspectionReceipt::Available(frame) => frame,
        UiMountedInspectionReceipt::Omitted(_) => return Err(()),
    };
    let target = frame.current_visual_target().map_err(|_| ())?;
    let grant = shell.visual_inspection_authority().issue_pixel_grant();
    let deadline_tick = tick.checked_add(POLL_TICK_CAPACITY).ok_or(())?;
    let request = UiVisualSnapshotRequest::for_local_development_unredacted_frame(target)
        .artifacts(UiPixelsRequired::policy())
        .deadline(UiVisualCaptureDeadline::at_tick(deadline_tick));
    let mut pending = shell
        .begin_visual_pixel_snapshot(&grant, request)
        .map_err(|_| ())?;
    let wall_deadline = Instant::now().checked_add(WALL_DEADLINE).ok_or(())?;
    loop {
        match shell.poll_visual_snapshot(pending, *tick) {
            UiVisualCapturePoll::Pending(next) => pending = next,
            UiVisualCapturePoll::Completed(UiVisualSnapshotOutcome::Captured(receipt)) => {
                let observation = observe(&receipt);
                shell.dispose_visual_snapshot(receipt);
                return observation;
            }
            UiVisualCapturePoll::Completed(UiVisualSnapshotOutcome::Superseded(_))
            | UiVisualCapturePoll::Completed(UiVisualSnapshotOutcome::Omitted(_))
            | UiVisualCapturePoll::Completed(UiVisualSnapshotOutcome::Denied(_))
            | UiVisualCapturePoll::Completed(UiVisualSnapshotOutcome::Indeterminate(_)) => {
                return Err(());
            }
        }
        if Instant::now() >= wall_deadline || *tick >= deadline_tick {
            shell.cancel_visual_snapshot(pending);
            return Err(());
        }
        *tick = tick.checked_add(1).ok_or(())?;
        std::thread::yield_now();
    }
}

fn observe(
    receipt: &UiVisualSnapshotReceipt<UiPixelsRequired>,
) -> Result<worth_ui_host_native::UiNativeClientVisualSnapshotObservation, ()> {
    let affinity = receipt.affinity();
    let coordinates = receipt.coordinates();
    let pixels = receipt.pixel_artifact();
    if pixels.capture_source()
        != worth_ui_inspection::UiVisualPixelCaptureSource::NativePresentation
        || pixels.retention() != worth_ui_inspection::UiVisualPixelRetentionDisposition::Retained
    {
        return Err(());
    }
    Ok(
        worth_ui_host_native::UiNativeClientVisualSnapshotObservation::reported(
            worth_ui_host_native::UiNativeClientVisualSnapshotInput {
                affinity: [
                    affinity.snapshot(),
                    affinity.presentation_attempt(),
                    affinity.frame(),
                    affinity.semantic_surface(),
                    affinity.host_surface(),
                    affinity.binding_generation(),
                    affinity.presentation_epoch(),
                ],
                relation: relation(affinity.relation()),
                native_client_origin: coordinates.native_client_origin(),
                client_physical_dimensions: coordinates.client_physical_dimensions(),
                viewport_logical_dimension_bits: bits(coordinates.viewport_logical_dimensions()),
                scale_bits: bits(coordinates.scale()),
                translation_bits: bits(coordinates.translation()),
                coordinate_orientation: orientation(coordinates.orientation()),
                coordinate_rounding: rounding(coordinates.rounding()),
                pixel_dimensions: pixels.dimensions(),
                pixel_stride: pixels.stride(),
                pixel_color_space: color_space(pixels.color_space()),
                pixel_bytes: pixels.bytes().to_vec().into_boxed_slice(),
                visible_region_count: u64::try_from(receipt.visible_region_count())
                    .map_err(|_| ())?,
                hit_test_region_count: u64::try_from(receipt.hit_test_region_count())
                    .map_err(|_| ())?,
                cost_counters: receipt.cost().counters(),
            },
        ),
    )
}

fn bits(values: [f32; 2]) -> [u32; 2] {
    [values[0].to_bits(), values[1].to_bits()]
}

fn relation(
    value: worth_ui_inspection::UiVisualSnapshotRelation,
) -> worth_ui_host_native::UiNativeClientVisualSnapshotRelation {
    match value {
        worth_ui_inspection::UiVisualSnapshotRelation::Current => {
            worth_ui_host_native::UiNativeClientVisualSnapshotRelation::Current
        }
        worth_ui_inspection::UiVisualSnapshotRelation::RetainedPredecessor => {
            worth_ui_host_native::UiNativeClientVisualSnapshotRelation::RetainedPredecessor
        }
        worth_ui_inspection::UiVisualSnapshotRelation::Historical => {
            worth_ui_host_native::UiNativeClientVisualSnapshotRelation::Historical
        }
    }
}

fn orientation(
    value: worth_ui_inspection::UiVisualCoordinateOrientation,
) -> worth_ui_host_native::UiNativeClientVisualCoordinateOrientation {
    match value {
        worth_ui_inspection::UiVisualCoordinateOrientation::TopLeftOrigin => {
            worth_ui_host_native::UiNativeClientVisualCoordinateOrientation::TopLeftOrigin
        }
        worth_ui_inspection::UiVisualCoordinateOrientation::BottomLeftOrigin => {
            worth_ui_host_native::UiNativeClientVisualCoordinateOrientation::BottomLeftOrigin
        }
    }
}

fn rounding(
    value: worth_ui_inspection::UiVisualCoordinateRounding,
) -> worth_ui_host_native::UiNativeClientVisualCoordinateRounding {
    match value {
        worth_ui_inspection::UiVisualCoordinateRounding::PixelCenterNearest => {
            worth_ui_host_native::UiNativeClientVisualCoordinateRounding::PixelCenterNearest
        }
        worth_ui_inspection::UiVisualCoordinateRounding::FloorEdges => {
            worth_ui_host_native::UiNativeClientVisualCoordinateRounding::FloorEdges
        }
    }
}

fn color_space(
    value: worth_ui_inspection::UiVisualPixelColorSpace,
) -> worth_ui_host_native::UiNativeClientVisualPixelColorSpace {
    match value {
        worth_ui_inspection::UiVisualPixelColorSpace::Srgb => {
            worth_ui_host_native::UiNativeClientVisualPixelColorSpace::Srgb
        }
        worth_ui_inspection::UiVisualPixelColorSpace::AdapterDeclared => {
            worth_ui_host_native::UiNativeClientVisualPixelColorSpace::AdapterDeclared
        }
    }
}
