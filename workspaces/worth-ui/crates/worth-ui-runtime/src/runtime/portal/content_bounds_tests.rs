use super::whole_points;

#[test]
fn a_fractional_length_takes_the_whole_point_that_holds_its_last_fraction() {
    assert_eq!(whole_points(100.0), 100);
    assert_eq!(whole_points(100.25), 101);
    assert_eq!(whole_points(0.5), 1);
}

#[test]
fn a_length_past_u16_holds_at_its_largest_count() {
    assert_eq!(whole_points(65_535.5), u16::MAX);
    assert_eq!(whole_points(1.0e9), u16::MAX);
    assert_eq!(whole_points(f32::INFINITY), u16::MAX);
}

#[test]
fn a_length_that_names_no_extent_lays_out_over_none() {
    assert_eq!(whole_points(-3.0), 0);
    assert_eq!(whole_points(f32::NAN), 0);
}
