//! Independent colour/rectangle observation, never a runtime offset readback.
use super::*;
use crate::external_observation::NativeClientPixelCapture;

pub(super) fn at_offset(frame: &VisibleScrollFrame, offset: f64, dpi: u32) -> bool {
    let geometry = RecentActivityScrollGeometry;
    (i64::from(frame.thumb_top_px) - physical_px(geometry.thumb_top_points(offset), dpi)).abs() <= 3
        && (i64::from(frame.thumb_length_px) - physical_px(geometry.thumb_length_points(), dpi))
            .abs()
            <= 3
}

pub(in crate::product_process::scroll_progression) fn observed(
    pixels: &NativeClientPixelCapture,
    dpi: u32,
    strip: [u32; 4],
) -> Result<(u32, u32), Failure> {
    let region = RecentActivityScrollGeometry.viewport_points();
    let column = physical_px(region[0] + region[2] - 6.0, dpi) as u32 - strip[0];
    if column >= pixels.width() {
        return Err(Failure::InputDelivery("thumb column outside timing strip"));
    }
    let mut best = (0, 0);
    let mut start = None;
    let first = (physical_px(region[1], dpi) as u32).saturating_sub(strip[1]);
    let end = (physical_px(region[1] + region[3] - 12.0, dpi) as u32)
        .saturating_sub(strip[1])
        .min(pixels.height());
    for y in first..=end {
        let thumb = if y == end {
            false
        } else {
            let offset = (y as usize * pixels.width() as usize + column as usize) * 4;
            let pixel = &pixels.rgba()[offset..offset + 3];
            [[176_u8, 181, 196], [146, 152, 172], [112, 119, 142]]
                .iter()
                .any(|tone| pixel.iter().zip(tone).all(|(a, b)| a.abs_diff(*b) <= 8))
        };
        match (thumb, start) {
            (true, None) => start = Some(y),
            (false, Some(first)) => {
                if y - first > best.1 {
                    best = (first, y - first);
                }
                start = None;
            }
            _ => {}
        }
    }
    if best.1 < 24 {
        return Err(Failure::InputDelivery(
            "complete visible thumb absent from timing capture",
        ));
    }
    Ok((strip[1] + best.0, best.1))
}
