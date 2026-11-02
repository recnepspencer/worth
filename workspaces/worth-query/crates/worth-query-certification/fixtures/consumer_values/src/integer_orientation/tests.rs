//! Independent small determinants and literal full-width cancellation oracles.
use super::*;

#[test]
fn integer_turns_match_independent_bounded_determinants_in_both_classes() {
    let points: Vec<_> = (-2i64..=2)
        .flat_map(|x| (-2..=2).map(move |y| [x, y]))
        .collect();
    for &a in &points {
        for &b in &points {
            for &c in &points {
                let cross = (i128::from(b[0]) - i128::from(a[0]))
                    * (i128::from(c[1]) - i128::from(a[1]))
                    - (i128::from(b[1]) - i128::from(a[1])) * (i128::from(c[0]) - i128::from(a[0]));
                let expected = cross.cmp(&0);
                assert_eq!(integer_orientation(a, b, c), expected);
                let unsigned = |point: [i64; 2]| point.map(|v| u64::try_from(v + 2).unwrap());
                assert_eq!(
                    integer_orientation(unsigned(a), unsigned(b), unsigned(c)),
                    expected
                );
            }
        }
    }
}

#[test]
fn full_unsigned_turns_preserve_unit_determinants_and_exact_collinearity() {
    let m = u64::MAX;
    for (a, b, c, expected) in [
        ([1, 1], [m, 1], [1, m], Ordering::Greater),
        // (m-2)^2 - (m-3)(m-1) is exactly +1.
        ([1, 1], [m - 1, m - 2], [m, m - 1], Ordering::Greater),
        ([1, 1], [m, m - 1], [m - 1, m - 2], Ordering::Less),
        ([1, 1], [m - 1, m - 1], [m, m], Ordering::Equal),
        ([0, m], [m, 0], [0, 0], Ordering::Less),
    ] {
        assert_eq!(integer_orientation(a, b, c), expected, "{a:?},{b:?},{c:?}");
        assert_eq!(integer_orientation(b, c, a), expected);
        assert_eq!(integer_orientation(a, c, b), expected.reverse());
    }
}

#[test]
fn complete_signed_range_preserves_the_same_exact_sign_contract() {
    let lo = i64::MIN;
    let hi = i64::MAX;
    for (a, b, c, expected) in [
        ([lo, lo], [hi, lo], [lo, hi], Ordering::Greater),
        ([lo, lo], [hi - 1, hi - 2], [hi, hi - 1], Ordering::Greater),
        ([lo, lo], [hi, hi - 1], [hi - 1, hi - 2], Ordering::Less),
        ([lo, lo], [hi - 1, hi - 1], [hi, hi], Ordering::Equal),
    ] {
        assert_eq!(integer_orientation(a, b, c), expected);
        assert_eq!(integer_orientation(b, c, a), expected);
        assert_eq!(integer_orientation(a, c, b), expected.reverse());
    }
}
