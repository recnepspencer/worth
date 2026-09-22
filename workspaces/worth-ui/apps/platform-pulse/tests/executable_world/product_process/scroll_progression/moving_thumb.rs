//! Catch wheel easing with real button-down, then observe held and dragged pixels.
use super::*;
use crate::adjudication::{adjudicate_content_shift, adjudicate_vertical_thumb};
use std::time::Instant;

pub(super) fn verify(
    world: &mut NativeBoundExecutableWorld,
    baseline: &NativeClientPixelCapture,
    dpi: u32,
    notch: f64,
) -> Result<(), PlatformPulseScrollJourneyFailure> {
    type Failure = PlatformPulseScrollJourneyFailure;
    let geometry = RecentActivityScrollGeometry;
    let press_point = client_point(baseline, dpi, geometry.thumb_center_points(0.0))?;
    // The unchanged point must remain inside the moving thumb for this whole
    // notch. This is geometry qualification, not a guessed animation sample.
    if geometry.thumb_top_points(notch) >= geometry.thumb_center_points(0.0)[1] - 3.0 {
        return Err(Failure::InputDelivery(
            "wheel setting leaves no qualified moving-thumb overlap",
        ));
    }
    let input = world
        .platform
        .prepare_wheel_input(&world.native_client, press_point)
        .map_err(Failure::Native)?;
    let exposure = world
        .platform
        .expose_client_area(&world.native_client)
        .map_err(Failure::Native)?;
    let region = geometry.viewport_points();
    // First-column content ink moves with Scroll. Keep the pointer >64px from
    // this strip so the desktop capture's existing cursor exclusion is intact.
    let strip = [region[0], region[1], 64.0, region[3]].map(|v| physical_px(v, dpi) as u32);
    let mut stream = world
        .platform
        .start_capture_stream(&exposure, strip)
        .map_err(Failure::Native)?;
    let initial = stream
        .next(Duration::from_secs(3))
        .map_err(Failure::Native)?
        .ok_or(Failure::InputDelivery("moving-thumb capture did not start"))?;
    let wheel = input.deliver_notch(1).map_err(Failure::Native)?;
    let deadline = Instant::now() + Duration::from_millis(80);
    let changed_at = loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(Failure::InputDelivery(
                "no visible moving content before thumb-press deadline",
            ));
        }
        if let Some(frame) = stream.next(remaining).map_err(Failure::Native)? {
            if frame.captured_qpc_100ns > wheel.after_qpc_100ns
                && changed_ink(&initial.pixels, &frame.pixels)?
            {
                break frame.captured_qpc_100ns;
            }
        }
    };
    let (held, press) = input.press_primary().map_err(Failure::Native)?;
    if changed_at >= press.before_qpc_100ns
        || press.after_qpc_100ns - wheel.before_qpc_100ns >= 800_000
    {
        return Err(Failure::InputDelivery(
            "thumb press was not bracketed inside first80ms after visible motion",
        ));
    }
    stream.finish().map_err(Failure::Native)?;

    // This wait is AFTER real button-down. It proves capture stopped the
    // unfinished animation, not that the harness waited for wheel settlement.
    let mut frozen_top = thumb(&pixels(world)?, dpi)?.0;
    let held_deadline = Instant::now() + TRANSITION_DEADLINE;
    loop {
        std::thread::sleep(Duration::from_millis(20));
        let next_top = thumb(&pixels(world)?, dpi)?.0;
        if next_top == frozen_top {
            break;
        }
        frozen_top = next_top;
        if Instant::now() >= held_deadline {
            return Err(Failure::InputDelivery("pressed thumb never stabilized"));
        }
    }
    std::thread::sleep(Duration::from_millis(160));
    let still = pixels(world)?;
    let (still_top, length) = thumb(&still, dpi)?;
    let rest_top = physical_px(geometry.thumb_top_points(0.0), dpi);
    let target_top = physical_px(geometry.thumb_top_points(notch), dpi);
    if still_top != frozen_top
        || i64::from(still_top) <= rest_top + 3
        || i64::from(still_top) >= target_top - 3
        || (i64::from(length) - physical_px(geometry.thumb_length_points(), dpi)).abs() > 3
    {
        return Err(Failure::InputDelivery(
            "held moving thumb did not preserve a stable intermediate pose",
        ));
    }
    let held_offset = content_offset(baseline, &still, dpi, still_top)?;
    let (x, y) = press_point.coordinates();
    let dragged_y = y
        .checked_add(THUMB_DRAG_DELTA_PX as u32)
        .ok_or(Failure::PointOutsideCapture([x, y]))?;
    let destination =
        NativeClientPixelPoint::interior(baseline, x, dragged_y, CLICK_LANDING_TOLERANCE_PX)
            .ok_or(Failure::PointOutsideCapture([x, dragged_y]))?;
    held.move_to(destination).map_err(Failure::Native)?;
    held.release().map_err(Failure::Native)?;
    let delta_points =
        geometry.offset_after_thumb_drag(0.0, THUMB_DRAG_DELTA_PX as f64 * 96.0 / f64::from(dpi));
    let deadline = Instant::now() + TRANSITION_DEADLINE;
    let dragged = loop {
        let current = pixels(world)?;
        let (top, _) = thumb(&current, dpi)?;
        if (i64::from(top) - i64::from(still_top) - THUMB_DRAG_DELTA_PX).abs() <= 2
            && adjudicate_content_shift(&still, &current, dpi, delta_points).is_ok()
        {
            break current;
        }
        if Instant::now() >= deadline {
            return Err(Failure::InputDelivery(
                "moving-thumb drag did not preserve physical grab displacement",
            ));
        }
        std::thread::sleep(PIXEL_POLL_SLICE);
    };
    drop(exposure);
    let (top, length) = thumb(&dragged, dpi)?;
    let from =
        NativeClientPixelPoint::interior(&dragged, x, top + length / 2, CLICK_LANDING_TOLERANCE_PX)
            .ok_or(Failure::PointOutsideCapture([x, top + length / 2]))?;
    // The top of the track forces the exact zero clamp irrespective of the
    // independently observed fractional held offset or snapped grab position.
    let to = client_point(
        &dragged,
        dpi,
        [geometry.thumb_center_points(0.0)[0], region[1]],
    )?;
    world
        .platform
        .deliver_pointer_drag(&world.native_client, from, to)
        .map_err(Failure::Native)?;
    let deadline = Instant::now() + TRANSITION_DEADLINE;
    loop {
        let restored = pixels(world)?;
        if adjudicate_vertical_thumb(&restored, dpi, 0.0).is_ok()
            && super::axes::pixels::require_restored(baseline, &restored, dpi).is_ok()
        {
            break;
        }
        if Instant::now() >= deadline {
            return Err(Failure::InputDelivery(
                "moving-thumb probe did not restore exact origin pixels",
            ));
        }
        std::thread::sleep(PIXEL_POLL_SLICE);
    }
    println!("native moving-thumb capture: visible change at{}100ns; wheel={wheel:?}, press={press:?}; held offset independently observed={held_offset}pt;30px held drag and origin restore passed", changed_at);
    Ok(())
}

fn pixels(
    world: &NativeBoundExecutableWorld,
) -> Result<NativeClientPixelCapture, PlatformPulseScrollJourneyFailure> {
    world
        .platform
        .capture_client_area(&world.native_client)
        .map_err(PlatformPulseScrollJourneyFailure::Native)
}

fn thumb(
    capture: &NativeClientPixelCapture,
    dpi: u32,
) -> Result<(u32, u32), PlatformPulseScrollJourneyFailure> {
    super::latency::thumb::observed(capture, dpi, [0, 0, capture.width(), capture.height()])
}

fn changed_ink(
    before: &NativeClientPixelCapture,
    after: &NativeClientPixelCapture,
) -> Result<bool, PlatformPulseScrollJourneyFailure> {
    if (before.process_id(), before.width(), before.height())
        != (after.process_id(), after.width(), after.height())
    {
        return Err(PlatformPulseScrollJourneyFailure::InputDelivery(
            "moving-thumb capture basis changed",
        ));
    }
    Ok(before
        .rgba()
        .chunks_exact(4)
        .zip(after.rgba().chunks_exact(4))
        .filter(|(a, b)| a[..3].iter().zip(&b[..3]).any(|(a, b)| a.abs_diff(*b) > 8))
        .take(8)
        .count()
        == 8)
}

fn content_offset(
    baseline: &NativeClientPixelCapture,
    held: &NativeClientPixelCapture,
    dpi: u32,
    thumb_top: u32,
) -> Result<f64, PlatformPulseScrollJourneyFailure> {
    let geometry = RecentActivityScrollGeometry;
    let thumb_delta = f64::from(thumb_top) * 96.0 / f64::from(dpi) - geometry.thumb_top_points(0.0);
    let estimated = physical_px(geometry.offset_after_thumb_drag(0.0, thumb_delta), dpi);
    // Inverting one snapped thumb edge gives a bounded three-content-pixel
    // interval. Actual content pixels select the observed translation within it.
    for shift in estimated.saturating_sub(3)..=estimated.saturating_add(3) {
        let offset = shift as f64 * 96.0 / f64::from(dpi);
        if adjudicate_vertical_thumb(held, dpi, offset).is_ok() {
            if let Ok(observed) = adjudicate_content_shift(baseline, held, dpi, offset) {
                return Ok(observed.shift_px() as f64 * 96.0 / f64::from(dpi));
            }
        }
    }
    Err(PlatformPulseScrollJourneyFailure::InputDelivery(
        "held content disagrees with independently observed thumb",
    ))
}
