//! One rounding, one refusal, and a round trip that lands where it started.
use super::*;

#[test]
fn points_round_to_the_nearest_subpixel_with_halves_away_from_zero() {
    assert_eq!(
        UiSubpixels::nearest(1.25_f32),
        Some(UiSubpixels::new(1_250))
    );
    assert_eq!(UiSubpixels::nearest(0.0005_f64), Some(UiSubpixels::new(1)));
    assert_eq!(
        UiSubpixels::nearest(-0.0005_f64),
        Some(UiSubpixels::new(-1))
    );
    assert_eq!(
        UiSubpixels::nearest(-2.0_f32),
        Some(UiSubpixels::new(-2_000))
    );
}

/// Layout measures in `f32`; the crossing widens before it scales, so a large
/// coordinate keeps the subpixel an `f32` product would round away.
#[test]
fn a_large_coordinate_keeps_its_subpixel() {
    let points = 20_000.123_f32;
    assert_eq!(
        UiSubpixels::nearest(points),
        Some(UiSubpixels::new(
            (f64::from(points) * 1_000.0).round() as i64
        ))
    );
    assert_ne!(
        UiSubpixels::nearest(points).map(UiSubpixels::count),
        Some((points * 1_000.0).round() as i64),
        "the f32 product this replaces lands elsewhere"
    );
}

#[test]
fn values_that_name_no_subpixel_are_refused() {
    assert_eq!(UiSubpixels::nearest(f32::NAN), None);
    assert_eq!(UiSubpixels::nearest(f32::INFINITY), None);
    assert_eq!(UiSubpixels::nearest(f32::MAX), None);
    assert_eq!(UiSubpixels::nearest(-f64::MAX), None);
    assert_eq!(UiSubpixels::whole_points(i64::MAX), None);
}

#[test]
fn a_distance_refuses_a_negative_rather_than_clamping_it() {
    assert_eq!(UiSubpixels::nearest_distance(-0.25_f32), None);
    assert_eq!(
        UiSubpixels::nearest_distance(0.0_f32),
        Some(UiSubpixels::new(0))
    );
    assert_eq!(
        UiSubpixels::nearest_distance(3.0_f32),
        Some(UiSubpixels::new(3_000))
    );
}

/// A count read back as points converts to the same count: always in `f64`,
/// and in the `f32` layout reads it at up to 16_384 points.
#[test]
fn subpixels_read_as_points_convert_back_to_themselves() {
    for count in [
        -16_384_000,
        -1_234_567,
        -1,
        0,
        1,
        999,
        1_001,
        8_388_607,
        16_383_999,
        16_384_000,
    ] {
        let subpixels = UiSubpixels::new(count);
        assert_eq!(UiSubpixels::nearest(subpixels.to_points()), Some(subpixels));
        assert_eq!(
            UiSubpixels::nearest(subpixels.to_points_f32()),
            Some(subpixels),
            "{count} survives the f32 layout reads it at"
        );
    }
}

/// Past 16_384 points `f32` steps coarser than a subpixel. The `f64` reading
/// still holds; the `f32` one lands on a neighbor. This pins where the layout
/// precision ends so a claim of an exact round trip cannot outgrow it.
#[test]
fn past_sixteen_thousand_points_f32_cannot_hold_every_subpixel() {
    let subpixels = UiSubpixels::new(16_384_001);
    assert_eq!(UiSubpixels::nearest(subpixels.to_points()), Some(subpixels));
    assert_ne!(
        UiSubpixels::nearest(subpixels.to_points_f32()),
        Some(subpixels)
    );
}

#[test]
fn thousandths_of_a_distance_round_like_points_do() {
    let line = UiSubpixels::whole_points(20).unwrap();
    assert_eq!(line.thousandths(3_000), Some(UiSubpixels::new(60_000)));
    assert_eq!(line.thousandths(1), Some(UiSubpixels::new(20)));
    assert_eq!(
        UiSubpixels::new(3).thousandths(500),
        Some(UiSubpixels::new(2))
    );
    assert_eq!(
        UiSubpixels::new(-3).thousandths(500),
        Some(UiSubpixels::new(-2))
    );
    assert_eq!(UiSubpixels::new(i64::MAX).thousandths(2_000), None);
}

#[test]
fn a_host_line_count_reads_as_thousandths() {
    assert_eq!(host_count_thousandths(host_count_of(3)), Some(3_000));
    assert_eq!(host_count_thousandths(-host_count_of(1)), Some(-1_000));
}
