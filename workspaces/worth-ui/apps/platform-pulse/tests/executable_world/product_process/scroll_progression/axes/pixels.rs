//! Independent row/rectangle pixels; no diagnostic Scroll offsets are read.
use super::*;

fn rgb(capture: &NativeClientPixelCapture, x: u32, y: u32) -> Option<[u8; 3]> {
    if x >= capture.width() || y >= capture.height() {
        return None;
    }
    let index = (y as usize * capture.width() as usize + x as usize) * 4;
    capture.rgba().get(index..index + 3)?.try_into().ok()
}

fn same(a: [u8; 3], b: [u8; 3]) -> bool {
    a.into_iter().zip(b).all(|(a, b)| a.abs_diff(b) <= 8)
}

pub(super) fn thumb_at(capture: &NativeClientPixelCapture, dpi: u32, offset: f64) -> bool {
    let y = physical_px(inline_center(offset)[1], dpi) as u32;
    let left = physical_px(290.0, dpi) as u32;
    let right = physical_px(290.0 + INLINE_TRACK, dpi) as u32;
    let mut longest = (0, 0);
    let mut start = None;
    for x in left..=right {
        let thumb = x < right
            && rgb(capture, x, y).is_some_and(|pixel| {
                [[176, 181, 196], [146, 152, 172], [112, 119, 142]]
                    .into_iter()
                    .any(|tone| same(pixel, tone))
            });
        match (thumb, start) {
            (true, None) => start = Some(x),
            (false, Some(first)) => {
                if x - first > longest.1 {
                    longest = (first, x - first);
                }
                start = None;
            }
            _ => {}
        }
    }
    let expected = physical_px(inline_center(offset)[0] - INLINE_THUMB / 2.0, dpi);
    (i64::from(longest.0) - expected).abs() <= 3
        && (i64::from(longest.1) - physical_px(INLINE_THUMB, dpi)).abs() <= 3
}

pub(super) fn require_horizontal_shift(
    before: &NativeClientPixelCapture,
    after: &NativeClientPixelCapture,
    dpi: u32,
    points: f64,
) -> Result<(), PlatformPulseScrollJourneyFailure> {
    let shift = physical_px(points, dpi) as u32;
    let left = physical_px(290.0 + 2.0, dpi) as u32;
    let right = physical_px(290.0 + INLINE_TRACK - 2.0, dpi) as u32 - shift;
    let top = physical_px(693.0 + 5.0, dpi) as u32;
    let bottom = physical_px(693.0 + 240.0, dpi) as u32;
    let mut matches = 0;
    let mut changed = 0;
    let mut ink = 0;
    for y in top..bottom {
        for x in left..right {
            let a = rgb(before, x + shift, y).unwrap();
            // Reject blank-overflow proof: only compare actual authored ink.
            if a.iter().all(|v| *v > 220) {
                continue;
            }
            ink += 1;
            matches += usize::from(same(a, rgb(after, x, y).unwrap()));
            changed += usize::from(!same(rgb(before, x, y).unwrap(), rgb(after, x, y).unwrap()));
        }
    }
    if ink < 100 || matches * 100 < ink * 85 || changed < 100 {
        return Err(PlatformPulseScrollJourneyFailure::InputDelivery(
            "Shift+wheel content ink did not move by the independent notch distance",
        ));
    }
    Ok(())
}

pub(in crate::product_process::scroll_progression) fn require_restored(
    before: &NativeClientPixelCapture,
    after: &NativeClientPixelCapture,
    dpi: u32,
) -> Result<(), PlatformPulseScrollJourneyFailure> {
    // First row text plus a neighboring fixed panel: restored content and no
    // viewport-wide transform. Avoid chrome whose hover legitimately differs.
    for rect in [[310.0, 699.0, 700.0, 40.0], [1090.0, 695.0, 150.0, 180.0]] {
        let [x, y, w, h] = rect.map(|v| physical_px(v, dpi) as u32);
        let mut mismatch = 0;
        for row in y..y + h {
            for col in x..x + w {
                mismatch += usize::from(!same(
                    rgb(before, col, row).unwrap(),
                    rgb(after, col, row).unwrap(),
                ));
            }
        }
        if mismatch * 100 > w as usize * h as usize * 2 {
            return Err(PlatformPulseScrollJourneyFailure::InputDelivery(
                "return to origin did not restore text or preserve the neighboring panel",
            ));
        }
    }
    Ok(())
}
