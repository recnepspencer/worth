use super::{whole_i64, whole_u16, whole_u32};

#[test]
fn a_whole_value_in_range_reads_as_its_integer() {
    assert_eq!(whole_u16(65_535.0), Some(u16::MAX));
    assert_eq!(whole_u32(4_294_967_295.0), Some(u32::MAX));
    assert_eq!(whole_u32(-0.0), Some(0));
    assert_eq!(whole_i64(-9_223_372_036_854_775_808.0), Some(i64::MIN));
    assert_eq!(whole_i64(-42.0), Some(-42));
}

#[test]
fn a_fraction_is_refused_rather_than_rounded_toward_zero() {
    assert_eq!(whole_u16(10.7), None);
    assert_eq!(whole_u32(0.5), None);
    assert_eq!(whole_i64(-1.5), None);
}

#[test]
fn a_value_past_the_range_is_refused_rather_than_saturated() {
    assert_eq!(whole_u16(65_536.0), None);
    assert_eq!(whole_u32(4_294_967_296.0), None);
    assert_eq!(whole_u32(-1.0), None);
    // `i64::MAX as f64` rounds up to 2^63, which no `i64` holds.
    assert_eq!(whole_i64(9_223_372_036_854_775_808.0), None);
}

#[test]
fn a_value_that_is_not_finite_is_refused_rather_than_read_as_zero() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(whole_u16(value), None);
        assert_eq!(whole_u32(value), None);
        assert_eq!(whole_i64(value), None);
    }
}
