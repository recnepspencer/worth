use worth_ui_host_contract::{
    UiAppearanceAllocationBounds, UiMountedCanonicalBox, UiMountedGeometryPosture,
    UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedAppearanceGeometryDenial {
    AllocationHasNoArea,
    CoordinateOverflow,
    ExtentOverflow,
    EmptyAtCanonicalPrecision,
}

/// Allocation geometry is expressed in logical points. Appearance records use
/// millipoints; quantize once here, before integer radius and border arithmetic.
pub(super) fn allocation(
    bounds: UiMountedCanonicalBox,
) -> Result<UiAppearanceAllocationBounds, UiMountedAppearanceGeometryDenial> {
    if bounds.posture() == UiMountedGeometryPosture::Empty {
        return Err(UiMountedAppearanceGeometryDenial::AllocationHasNoArea);
    }
    UiAppearanceAllocationBounds::new(
        coordinate(bounds.x())?,
        coordinate(bounds.y())?,
        extent(bounds.width())?,
        extent(bounds.height())?,
    )
    .map_err(|_| UiMountedAppearanceGeometryDenial::EmptyAtCanonicalPrecision)
}

pub(super) fn coordinate(points: f32) -> Result<i32, UiMountedAppearanceGeometryDenial> {
    let subpixels = quantized_subpixels(points);
    if !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&subpixels) {
        return Err(UiMountedAppearanceGeometryDenial::CoordinateOverflow);
    }
    Ok(subpixels as i32)
}

pub(super) fn extent(points: f32) -> Result<u32, UiMountedAppearanceGeometryDenial> {
    let subpixels = quantized_subpixels(points);
    if !(0.0..=f64::from(u32::MAX)).contains(&subpixels) {
        return Err(UiMountedAppearanceGeometryDenial::ExtentOverflow);
    }
    Ok(subpixels as u32)
}

fn quantized_subpixels(points: f32) -> f64 {
    // Widen before multiplication so source f32 precision is not reduced again.
    (f64::from(points) * f64::from(UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT)).round_ties_even()
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_ui_host_contract::{UiMountedCanonicalBoxInput, UiMountedCoordinateSpace};

    fn bounds(x: f32, y: f32, width: f32, height: f32) -> UiMountedCanonicalBox {
        UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x,
            y,
            width,
            height,
            coordinate_space: UiMountedCoordinateSpace::Viewport,
        })
        .unwrap()
    }

    #[test]
    fn allocation_preserves_point_units_and_fractional_geometry() {
        let actual = allocation(bounds(-10.25, 20.5, 100.125, 32.0)).unwrap();
        assert_eq!(
            actual,
            UiAppearanceAllocationBounds::new(-10_250, 20_500, 100_125, 32_000).unwrap()
        );
        let radii = worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii::normalize(
            actual,
            [worth_ui_host_contract::UiAppearanceLogicalLength::new(8_000).unwrap(); 4],
        );
        assert_eq!(radii.corners(), [8_000; 4]);
    }

    #[test]
    fn canonical_precision_rounds_ties_even_for_signed_coordinates_and_extents() {
        assert_eq!(
            allocation(bounds(-0.0625, 0.1875, 0.0625, 0.1875)).unwrap(),
            UiAppearanceAllocationBounds::new(-62, 188, 62, 188).unwrap(),
        );
    }

    #[test]
    fn overflow_and_subpixel_empty_extent_deny_without_saturating_geometry() {
        assert_eq!(
            allocation(bounds(3_000_000.0, 0.0, 1.0, 1.0)),
            Err(UiMountedAppearanceGeometryDenial::CoordinateOverflow)
        );
        assert_eq!(
            allocation(bounds(0.0, 0.0, 5_000_000.0, 1.0)),
            Err(UiMountedAppearanceGeometryDenial::ExtentOverflow)
        );
        assert_eq!(
            allocation(bounds(0.0, 0.0, 0.0001, 1.0)),
            Err(UiMountedAppearanceGeometryDenial::EmptyAtCanonicalPrecision)
        );
        assert_eq!(
            allocation(bounds(0.0, 0.0, 0.0, 1.0)),
            Err(UiMountedAppearanceGeometryDenial::AllocationHasNoArea)
        );
    }
}
