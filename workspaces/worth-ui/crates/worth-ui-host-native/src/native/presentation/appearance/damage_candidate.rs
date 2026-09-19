use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
};

use super::damage::UiNativeAppearanceDamageRect;
use super::geometry::UiNativeGeometryDenial;

/// The existing native spatial index is a bounded candidate accelerator with
/// f32 AABBs. Its bounds are deliberately wider than the exact i64 damage
/// boxes so float conversion can never suppress an exact replay candidate.
pub(super) fn candidate_bounds(
    rect: UiNativeAppearanceDamageRect,
) -> Result<UiMountedCanonicalBox, UiNativeGeometryDenial> {
    let margin = candidate_margin(rect);
    let left = rect
        .left
        .checked_sub(margin)
        .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?;
    let top = rect
        .top
        .checked_sub(margin)
        .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?;
    let right = rect
        .right
        .checked_add(margin)
        .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?;
    let bottom = rect
        .bottom
        .checked_add(margin)
        .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?;
    let width = right
        .checked_sub(left)
        .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?;
    let height = bottom
        .checked_sub(top)
        .ok_or(UiNativeGeometryDenial::CoordinateOverflow)?;
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: left as f32,
        y: top as f32,
        width: width as f32,
        height: height as f32,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .map_err(|_| UiNativeGeometryDenial::CoordinateOverflow)
}

fn candidate_margin(rect: UiNativeAppearanceDamageRect) -> i64 {
    let magnitude = [rect.left, rect.top, rect.right, rect.bottom]
        .into_iter()
        .map(i64::unsigned_abs)
        .max()
        .unwrap_or(0) as f64;
    let margin = (magnitude * f64::from(f32::EPSILON) * 8.0).ceil();
    margin.max(1.0) as i64
}
