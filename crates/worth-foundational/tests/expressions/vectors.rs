//! Fixed numeric byte vectors. Each expected bit pattern follows from
//! IEEE 754 binary64 with one nearest-even rounding per declared operation,
//! so the same vectors pass unchanged on every supported target; a target
//! that fused, reassociated, flushed, or widened would fail here.

use worth_foundational::expression_api::ExpressionDenialFamily as Family;

use super::{evaluation_denial, value};

fn bits(source: &str) -> u64 {
    match value(source).as_f64() {
        Some(float) => float.to_bits(),
        None => panic!("{source} is not a Float64"),
    }
}

#[test]
fn float_results_have_fixed_bytes() {
    let vectors: [(&str, u64); 12] = [
        ("0.1 + 0.2", 0x3FD3_3333_3333_3334),
        ("0.1 * 3.0", 0x3FD3_3333_3333_3334),
        ("1.0 / 3.0", 0x3FD5_5555_5555_5555),
        ("2.0 / 3.0", 0x3FE5_5555_5555_5555),
        ("sqrt(2.0)", 0x3FF6_A09E_667F_3BCD),
        ("sqrt(3.0)", 0x3FFB_B67A_E858_4CAA),
        // One rounding per operation: an implicit fused multiply-add would
        // leave the 2^-54-scale product error behind.
        ("0.1 * 10.0 - 1.0", 0),
        // Left to right, no reassociation: 1e16 + 1 ties back to 1e16.
        ("(1e16 + 1.0) - 1e16", 0),
        // Gradual underflow keeps subnormals; nearest-even may reach zero.
        ("2.2250738585072014e-308 / 2.0", 0x0008_0000_0000_0000),
        ("4.9e-324 / 2.0", 0),
        // Zero results are positive zero.
        ("0.0 * -1.0", 0),
        ("-0.0", 0),
    ];
    for (source, expected) in vectors {
        assert_eq!(bits(source), expected, "{source}: {:#018x}", bits(source));
    }
}

#[test]
fn quantities_round_once_into_canonical_units() {
    let magnitude = |source: &str| value(source).as_quantity().expect("a quantity").to_bits();
    // 1 mm is 1/1000 m, rounded once to the nearest binary64.
    assert_eq!(magnitude("quantity(1.0, mm)"), 0x3F50_624D_D2F1_A9FC);
    assert_eq!(magnitude("quantity(0.5, m)"), 0x3FE0_0000_0000_0000);
}

#[test]
fn nonfinite_results_deny_identically() {
    let cases = [
        ("1e308 * 10.0", Family::ArithmeticOverflow),
        ("-1e308 - 1e308", Family::ArithmeticOverflow),
        ("0.0 / 0.0", Family::DivisionByZero),
        ("sqrt(-4.9e-324)", Family::ArithmeticDomain),
    ];
    for (source, family) in cases {
        assert_eq!(evaluation_denial(source).family(), family, "{source}");
    }
}

#[test]
fn decimals_have_fixed_parts() {
    let parts = |source: &str| value(source).as_decimal().expect("a decimal");
    let cases = [
        (
            r#"decimal_div(decimal("2"), decimal("3"), 18, Rounding::NearestEven)"#,
            (666_666_666_666_666_667, 18),
        ),
        (
            r#"decimal_div(decimal("-2"), decimal("3"), 18, Rounding::TowardZero)"#,
            (-666_666_666_666_666_666, 18),
        ),
        (r#"decimal("1.2500")"#, (125, 2)),
        (r#"decimal("0.000")"#, (0, 0)),
        (
            r#"decimal("99999999999999999999.999999999999999999") - decimal("0.000000000000000001")"#,
            (99_999_999_999_999_999_999_999_999_999_999_999_998, 18),
        ),
    ];
    for (source, expected) in cases {
        assert_eq!(parts(source), expected, "{source}");
    }
    assert_eq!(parts(r#"decimal("0.1") + decimal("0.2")"#), (3, 1));
}
