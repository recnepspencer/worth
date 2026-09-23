//! Two-axis native travel and return-to-origin against authored geometry.
use super::*;
use std::time::Instant;
pub(super) mod pixels;

const INLINE_MAX: f64 = 1560.0 - 768.0;
const INLINE_TRACK: f64 = 768.0 - 12.0;
const INLINE_THUMB: f64 = INLINE_TRACK * 768.0 / 1560.0;

pub(super) fn verify(
    world: &mut NativeBoundExecutableWorld,
    baseline: &NativeClientPixelCapture,
    dpi: u32,
    notch: f64,
) -> Result<(), PlatformPulseScrollJourneyFailure> {
    let geometry = RecentActivityScrollGeometry;
    let interior = client_point(baseline, dpi, geometry.content_interior_points())?;
    world
        .platform
        .deliver_shift_wheel_notch(&world.native_client, interior)
        .map_err(PlatformPulseScrollJourneyFailure::Native)?;
    let shifted = await_pose(
        world,
        dpi,
        notch,
        0.0,
        Some(ContentExpectation::HorizontalShift(baseline, notch)),
    )?;
    pixels::require_horizontal_shift(baseline, &shifted, dpi, notch)?;
    drag_inline(world, baseline, dpi, notch, INLINE_MAX)?;
    await_pose(world, dpi, INLINE_MAX, 0.0, None)?;
    drag_inline(world, baseline, dpi, INLINE_MAX, 0.0)?;
    let restored = await_pose(
        world,
        dpi,
        0.0,
        0.0,
        Some(ContentExpectation::Restored(baseline)),
    )?;
    pixels::require_restored(baseline, &restored, dpi)?;
    world
        .platform
        .drag_thumb_outside_below(
            &world.native_client,
            client_point(baseline, dpi, geometry.thumb_center_points(0.0))?,
        )
        .map_err(PlatformPulseScrollJourneyFailure::Native)?;
    await_pose(world, dpi, 0.0, geometry.max_offset_points(), None)?;
    // A later pressed drag could hide a release that never ended capture.
    // Move with no button pressed first; the clamped offset must stay put.
    world
        .platform
        .move_pointer_without_focus_recovery(&world.native_client, interior)
        .map_err(PlatformPulseScrollJourneyFailure::Native)?;
    std::thread::sleep(Duration::from_millis(150));
    await_pose(world, dpi, 0.0, geometry.max_offset_points(), None)?;
    world
        .platform
        .deliver_pointer_drag(
            &world.native_client,
            client_point(
                baseline,
                dpi,
                geometry.thumb_center_points(geometry.max_offset_points()),
            )?,
            client_point(baseline, dpi, geometry.thumb_center_points(0.0))?,
        )
        .map_err(PlatformPulseScrollJourneyFailure::Native)?;
    let restored = await_pose(
        world,
        dpi,
        0.0,
        0.0,
        Some(ContentExpectation::Restored(baseline)),
    )?;
    pixels::require_restored(baseline, &restored, dpi)?;
    println!("native Shift+wheel and horizontal thumb endpoints/return: passed");
    println!("native vertical endpoints, release outside window and restored text: passed");
    Ok(())
}

fn inline_center(offset: f64) -> [f64; 2] {
    [
        290.0 + (INLINE_TRACK - INLINE_THUMB) * offset / INLINE_MAX + INLINE_THUMB / 2.0,
        693.0 + 269.0 - 6.0,
    ]
}

fn drag_inline(
    world: &mut NativeBoundExecutableWorld,
    baseline: &NativeClientPixelCapture,
    dpi: u32,
    from: f64,
    to: f64,
) -> Result<(), PlatformPulseScrollJourneyFailure> {
    world
        .platform
        .deliver_pointer_drag(
            &world.native_client,
            client_point(baseline, dpi, inline_center(from))?,
            client_point(baseline, dpi, inline_center(to))?,
        )
        .map_err(PlatformPulseScrollJourneyFailure::Native)
}

#[derive(Clone, Copy)]
enum ContentExpectation<'a> {
    HorizontalShift(&'a NativeClientPixelCapture, f64),
    Restored(&'a NativeClientPixelCapture),
}

impl ContentExpectation<'_> {
    fn check(
        self,
        current: &NativeClientPixelCapture,
        dpi: u32,
    ) -> Result<(), PlatformPulseScrollJourneyFailure> {
        match self {
            Self::HorizontalShift(baseline, points) => {
                pixels::require_horizontal_shift(baseline, current, dpi, points)
            }
            Self::Restored(baseline) => pixels::require_restored(baseline, current, dpi),
        }
    }
}

fn await_pose(
    world: &mut NativeBoundExecutableWorld,
    dpi: u32,
    inline: f64,
    block: f64,
    content: Option<ContentExpectation<'_>>,
) -> Result<NativeClientPixelCapture, PlatformPulseScrollJourneyFailure> {
    let deadline = Instant::now() + TRANSITION_DEADLINE;
    loop {
        let current = capture(world)?;
        if pixels::thumb_at(&current, dpi, inline)
            && crate::adjudication::adjudicate_vertical_thumb(&current, dpi, block).is_ok()
            && content.is_none_or(|expected| expected.check(&current, dpi).is_ok())
        {
            return Ok(current);
        }
        if Instant::now() >= deadline {
            eprintln!("axis geometry deadline: inline={inline}, block={block}; inline_matches={}, block_result={:?}, content_result={:?}",
                pixels::thumb_at(&current, dpi, inline),
                crate::adjudication::adjudicate_vertical_thumb(&current, dpi, block),
                content.map(|expected| expected.check(&current, dpi)));
            export_capture("scroll-axis-mismatch.png", &current)?;
            return Err(PlatformPulseScrollJourneyFailure::InputDelivery(
                "two-axis thumb pixels did not reach independent expected geometry",
            ));
        }
        std::thread::sleep(PIXEL_POLL_SLICE);
    }
}
