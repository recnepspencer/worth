//! Points and appearance subpixels cross in one place, rounding one way.
use super::*;

#[test]
fn points_quantize_to_the_nearest_subpixel_with_halves_to_even() {
    assert_eq!(appearance_coordinate_nearest(-10.25), Some(-10_250));
    assert_eq!(appearance_coordinate_nearest(-0.0625), Some(-62));
    assert_eq!(appearance_coordinate_nearest(0.1875), Some(188));
    assert_eq!(appearance_extent_nearest(0.0625), Some(62));
    assert_eq!(appearance_extent_nearest(100.125), Some(100_125));
}

#[test]
fn points_no_count_holds_are_refused_rather_than_saturated() {
    assert_eq!(appearance_coordinate_nearest(3_000_000.0), None);
    assert_eq!(appearance_coordinate_nearest(-3_000_000.0), None);
    assert_eq!(appearance_extent_nearest(5_000_000.0), None);
    assert_eq!(appearance_extent_nearest(-0.001), None);
    for points in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(appearance_coordinate_nearest(points), None);
        assert_eq!(appearance_extent_nearest(points), None);
    }
}

#[test]
fn every_record_count_reads_back_exactly_before_narrowing() {
    assert_eq!(appearance_points(i64::from(i32::MAX)), 2_147_483.647);
    assert_eq!(appearance_points(i64::from(i32::MIN)), -2_147_483.648);
    assert_eq!(appearance_points(i64::from(u32::MAX)), 4_294_967.295);
}

#[test]
fn a_quantized_point_reads_back_as_itself() {
    assert_eq!(appearance_points_f32(1_500), 1.5);
    for points in [-10.25_f32, 0.0, 0.5, 20.5, 100.125, 16_384.0] {
        let coordinate = appearance_coordinate_nearest(points).unwrap();
        assert_eq!(appearance_points_f32(i64::from(coordinate)), points);
    }
}
