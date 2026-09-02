use super::geometry::{UiNativeAppearanceScale, UiNativePhysicalRect};
use super::mounted_mechanic_fixtures::{allocation, logical_length};

#[test]
fn qualified_scales_preserve_exact_half_open_floor_and_ceil_edges() {
    let scale = UiNativeAppearanceScale::qualified(1_250).unwrap();
    let rect =
        UiNativePhysicalRect::from_allocation(allocation(-1_000, 2_000, 3_000, 4_000), scale)
            .unwrap();
    assert_eq!(
        rect.pixel_bounds(),
        super::geometry::UiNativePhysicalPixelRect {
            left: -2,
            top: 2,
            right: 3,
            bottom: 8,
        }
    );
    assert!(UiNativeAppearanceScale::qualified(1_333).is_err());
}

#[test]
fn staged_surface_fixture_keeps_logical_bounds_in_the_contract_unit() {
    let allocation = allocation(0, 0, 2_000, 3_000);
    assert_eq!(allocation.width(), 2_000);
    assert_eq!(logical_length(1_000).subpixels(), 1_000);
}
