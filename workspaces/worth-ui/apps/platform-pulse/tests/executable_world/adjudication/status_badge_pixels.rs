use super::dashboard_visual_oracle as dashboard;
use crate::external_observation::NativeClientPixelCapture;

const STATUS_TEXT_RGB: [u8; 3] = [116, 103, 232];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DashboardStatusBadgeFailure {
    UnexpectedExtent([u32; 2]),
    InkMissing,
    InkClipped { edge_ink: usize },
}

/// The current dashboard projects Query status into the deployment badge,
/// not the removed two-line Query card. Require legible ink with room at the
/// badge edges rather than importing the product's text layout.
pub(crate) fn adjudicate_dashboard_status_badge(
    capture: &NativeClientPixelCapture,
) -> Result<(), DashboardStatusBadgeFailure> {
    let [width, height] = dashboard::LOGICAL_EXTENT;
    if capture.width() * height != capture.height() * width {
        return Err(DashboardStatusBadgeFailure::UnexpectedExtent([
            capture.width(),
            capture.height(),
        ]));
    }
    let [left, top, badge_width, badge_height] = dashboard::QUERY_POSTURE_REGION;
    let interior_x = left + 6..=left + badge_width - 7;
    let interior_y = top + 5..=top + badge_height - 6;
    let mut interior_ink = 0;
    let mut edge_ink = 0;
    for y in top..top + badge_height {
        for x in left..left + badge_width {
            let px = x * capture.width() / width;
            let py = y * capture.height() / height;
            let offset = ((py * capture.width() + px) * 4) as usize;
            let Some(pixel) = capture.rgba().get(offset..offset + 3) else {
                continue;
            };
            let ink = pixel
                .iter()
                .zip(STATUS_TEXT_RGB)
                .all(|(observed, expected)| {
                    observed.abs_diff(expected) <= dashboard::CHANNEL_TOLERANCE
                });
            if ink {
                if !interior_x.contains(&x) || !interior_y.contains(&y) {
                    edge_ink += 1;
                } else {
                    interior_ink += 1;
                }
            }
        }
    }
    if interior_ink < 8 {
        return Err(DashboardStatusBadgeFailure::InkMissing);
    }
    if edge_ink > 0 {
        return Err(DashboardStatusBadgeFailure::InkClipped { edge_ink });
    }
    Ok(())
}
