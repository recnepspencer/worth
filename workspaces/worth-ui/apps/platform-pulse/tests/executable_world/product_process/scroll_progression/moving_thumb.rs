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
    // Qualify the independent sources while still at rest, before opening the
    // observation interval. No settled-source capture runs between wheel and press.
    let qualified = pixels(world)?;
    let exposure = world
        .platform
        .expose_client_area(&world.native_client)
        .map_err(Failure::Native)?;
    let region = geometry.viewport_points();
    let capture_region = region.map(|v| physical_px(v, dpi) as u32);
    let qualified = qualified
        .cropped(capture_region)
        .ok_or(Failure::InputDelivery(
            "qualified moving-thumb panel outside capture",
        ))?;
    // Only first-column content ink counts, excluding thumb hover and pointer.
    // GDI supplies a functional observation, never a display-cadence timestamp.
    let strip = [0, 0, physical_px(64.0, dpi) as u32, capture_region[3]];
    let initial = world
        .platform
        .observe_exposed_gdi_region(&exposure, capture_region)
        .map_err(Failure::Native)?;
    if changed_ink(&qualified, &initial, strip)? {
        return Err(Failure::InputDelivery(
            "GDI moving-thumb baseline disagrees with qualified content",
        ));
    }
    let rest_top = physical_px(geometry.thumb_top_points(0.0), dpi);
    let target_top = physical_px(geometry.thumb_top_points(notch), dpi);
    let wheel = input.deliver_notch(1).map_err(Failure::Native)?;
    let deadline = Instant::now() + TRANSITION_DEADLINE;
    let mut observed_range = (i64::MAX, i64::MIN, 0_u32);
    let (change_observed_by, pressed_observed_top) = loop {
        if Instant::now() >= deadline {
            eprintln!("moving-thumb observation missed intermediate pose: rest={rest_top}, target={target_top}, observed(min,max,count)={observed_range:?}");
            return Err(Failure::InputDelivery(
                "no visible moving content before thumb-press deadline",
            ));
        }
        let current = world
            .platform
            .observe_exposed_gdi_region(&exposure, capture_region)
            .map_err(Failure::Native)?;
        let completed = world
            .platform
            .observation_qpc_100ns()
            .map_err(Failure::Native)?;
        // A fast first sample can move content while the thumb is still within
        // the edge scanner's three-pixel tolerance. Choose an observable
        // intermediate pose, not a timer delay or a settled endpoint.
        let observed_top =
            i64::from(super::thumb_observation::observed(&current, dpi, capture_region)?.0);
        observed_range.0 = observed_range.0.min(observed_top);
        observed_range.1 = observed_range.1.max(observed_top);
        observed_range.2 += 1;
        if completed > wheel.after_qpc_100ns
            && observed_top > rest_top + 3
            && observed_top < target_top - 3
            && changed_ink(&initial, &current, strip)?
        {
            break (completed, observed_top);
        }
    };
    let (held, press) = input.press_primary().map_err(Failure::Native)?;
    println!("moving-thumb input brackets: wheel={wheel:?}, observed_by={change_observed_by}, observed_top={pressed_observed_top}, press={press:?}");
    if change_observed_by >= press.before_qpc_100ns {
        return Err(Failure::InputDelivery(
            "thumb press did not follow the visible intermediate pose",
        ));
    }

    // This wait is AFTER real button-down. It proves capture stopped the
    // unfinished animation, not that the harness waited for wheel settlement.
    // The stable intermediate pose below is the functional witness; no guessed
    // input-to-press delay substitutes for it. Cadence is qualified separately.
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
    eprintln!("moving-thumb held pose: observed_top={pressed_observed_top}, still_top={still_top}, held_offset={held_offset}pt");
    let (x, y) = press_point.coordinates();
    let dragged_y = y
        .checked_add(THUMB_DRAG_DELTA_PX as u32)
        .ok_or(Failure::PointOutsideCapture([x, y]))?;
    let destination =
        NativeClientPixelPoint::interior(baseline, x, dragged_y, CLICK_LANDING_TOLERANCE_PX)
            .ok_or(Failure::PointOutsideCapture([x, dragged_y]))?;
    held.move_to(destination).map_err(Failure::Native)?;
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
            eprintln!("moving-thumb drag mismatch: held_top={still_top}, observed_top={top}, expected_delta={THUMB_DRAG_DELTA_PX}px, expected_content_delta={delta_points}pt, content={:?}", adjudicate_content_shift(&still, &current, dpi, delta_points));
            return Err(Failure::InputDelivery(
                "moving-thumb drag did not preserve physical grab displacement",
            ));
        }
        std::thread::sleep(PIXEL_POLL_SLICE);
    };
    held.release().map_err(Failure::Native)?;
    let released = pixels(world)?;
    if (i64::from(thumb(&released, dpi)?.0) - i64::from(thumb(&dragged, dpi)?.0)).abs() > 2 {
        return Err(Failure::InputDelivery(
            "thumb release changed the held drag position",
        ));
    }
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
    println!("native moving-thumb capture: GDI change observed by QPC {change_observed_by} (completion upper bound, not frame time); wheel={wheel:?}, press={press:?}; held offset independently observed={held_offset}pt; 30px held drag and origin restore passed");
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
    super::thumb_observation::observed(capture, dpi, [0, 0, capture.width(), capture.height()])
}

fn changed_ink(
    before: &NativeClientPixelCapture,
    after: &NativeClientPixelCapture,
    [left, top, width, height]: [u32; 4],
) -> Result<bool, PlatformPulseScrollJourneyFailure> {
    if (before.process_id(), before.width(), before.height())
        != (after.process_id(), after.width(), after.height())
    {
        return Err(PlatformPulseScrollJourneyFailure::InputDelivery(
            "moving-thumb capture basis changed",
        ));
    }
    let right = left
        .checked_add(width)
        .filter(|right| *right <= before.width());
    let bottom = top
        .checked_add(height)
        .filter(|bottom| *bottom <= before.height());
    let (Some(right), Some(bottom)) = (right, bottom) else {
        return Err(PlatformPulseScrollJourneyFailure::InputDelivery(
            "moving-thumb ink strip outside capture",
        ));
    };
    Ok((top..bottom)
        .flat_map(|y| (left..right).map(move |x| (x, y)))
        .filter(|(x, y)| {
            let offset = (*y as usize * before.width() as usize + *x as usize) * 4;
            before.rgba()[offset..offset + 3]
                .iter()
                .zip(&after.rgba()[offset..offset + 3])
                .any(|(a, b)| a.abs_diff(*b) > 8)
        })
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

#[test]
fn moving_thumb_change_requires_ink_inside_the_qualified_content_strip() {
    let original = [20_u8, 20, 20, 255].repeat(64);
    let before = NativeClientPixelCapture::new(7, 8, 8, original.clone()).unwrap();
    let mut outside = original.clone();
    for x in 0..8 {
        outside[x * 4..x * 4 + 3].fill(200);
    }
    let outside = NativeClientPixelCapture::new(7, 8, 8, outside).unwrap();
    assert!(matches!(
        changed_ink(&before, &outside, [2, 2, 4, 4]),
        Ok(false)
    ));
    let mut inside = original;
    for y in 2..4 {
        for x in 2..6 {
            let offset = (y * 8 + x) * 4;
            inside[offset..offset + 3].fill(200);
        }
    }
    let inside = NativeClientPixelCapture::new(7, 8, 8, inside).unwrap();
    assert!(matches!(
        changed_ink(&before, &inside, [2, 2, 4, 4]),
        Ok(true)
    ));
    assert!(changed_ink(&before, &inside, [7, 7, 4, 4]).is_err());
}
