// Exhaustive small-width truth tables for `Bits<N>` and `Logic4<N>`,
// checked against an independent per-symbol oracle written from the
// language rules: bus literals are most significant bit first, Z is
// unknown in bitwise operations, and a mux keeps the symbols its arms share.
#[allow(unused_imports)]
use crate::expressions::reference_context::digital::*;

pub(crate) use crate::expressions::ordinary_context::evaluate;

pub(crate) use {
    crate::expressions::reference_context::digital::Symbol::One,
    crate::expressions::reference_context::digital::Symbol::Zero,
    crate::expressions::reference_context::digital::Symbol::X,
    crate::expressions::reference_context::digital::Symbol::Z,
};

#[test]
fn logic4_bitwise_tables_are_exhaustive_to_width_two() {
    for width in 1..=2 {
        for a in buses(width) {
            let not: Vec<_> = a.iter().map(|symbol| symbol.not()).collect();
            assert_eq!(outcome(&format!("bit_not({})", literal(&a))), logic(&not));
            for b in buses(width) {
                let (x, y) = (literal(&a), literal(&b));
                let cases: [(&str, fn(Symbol, Symbol) -> Symbol); 3] = [
                    (
                        "bit_and",
                        crate::expressions::reference_context::digital::Symbol::and,
                    ),
                    (
                        "bit_or",
                        crate::expressions::reference_context::digital::Symbol::or,
                    ),
                    (
                        "bit_xor",
                        crate::expressions::reference_context::digital::Symbol::xor,
                    ),
                ];
                for (name, op) in cases {
                    let source = format!("{name}({x}, {y})");
                    assert_eq!(outcome(&source), logic(&zip(&a, &b, op)), "{source}");
                }
            }
        }
    }
}

#[test]
fn logic4_equality_tables_are_exhaustive_to_width_two() {
    for width in 1..=2 {
        for a in buses(width) {
            for b in buses(width) {
                let (x, y) = (literal(&a), literal(&b));
                let mismatch = a
                    .iter()
                    .zip(&b)
                    .any(|(a, b)| matches!((a.known(), b.known()), (Some(p), Some(q)) if p != q));
                let unknown = a.iter().chain(&b).any(|symbol| symbol.known().is_none());
                let expected = match (mismatch, unknown) {
                    (true, _) => Zero,
                    (false, true) => X,
                    (false, false) => One,
                };
                assert_eq!(outcome(&format!("logic_eq({x}, {y})")), logic(&[expected]));
                assert_eq!(
                    outcome(&format!("case_equal({x}, {y})")),
                    worth_foundational::expression_api::ExpressionValue::bool(a == b)
                );
            }
        }
    }
}

#[test]
fn logic4_mux_tables_are_exhaustive_to_width_two() {
    for width in 1..=2 {
        for select in SYMBOLS {
            for one in buses(width) {
                for zero in buses(width) {
                    let expected: Vec<_> = match select {
                        One => one.clone(),
                        Zero => zero.clone(),
                        X | Z => zip(&one, &zero, |a, b| if a == b { a } else { X }),
                    };
                    let source = format!(
                        "mux({}, {}, {})",
                        literal(&[select]),
                        literal(&one),
                        literal(&zero)
                    );
                    assert_eq!(outcome(&source), logic(&expected), "{source}");
                }
            }
        }
    }
}

#[test]
fn bits_tables_are_exhaustive_to_width_three() {
    const WIDTH: u32 = 3;
    const MASK: u64 = (1 << WIDTH) - 1;
    for a in 0..=MASK {
        let x = bits_literal(WIDTH, a);
        assert_eq!(outcome(&format!("bit_not({x})")), bits(WIDTH, !a & MASK));
        assert_eq!(
            outcome(&format!("exact_cast<UInt8>({x})")),
            worth_foundational::expression_api::ExpressionValue::integer(a.into())
        );
        assert_eq!(outcome(&format!("to_bits<3>({a})")), bits(WIDTH, a));
        assert_eq!(outcome(&format!("extend<5>({x})")), bits(5, a));
        assert_eq!(outcome(&format!("truncate<2>({x})")), bits(2, a & 0b11));
        assert_eq!(outcome(&format!("slice<1, 3>({x})")), bits(2, a >> 1));
        for count in 0..=4_u64 {
            let left = if count >= 3 { 0 } else { (a << count) & MASK };
            let right = if count >= 3 { 0 } else { a >> count };
            assert_eq!(
                outcome(&format!("shift_left({x}, {count})")),
                bits(WIDTH, left)
            );
            assert_eq!(
                outcome(&format!("shift_right({x}, {count})")),
                bits(WIDTH, right)
            );
        }
        for b in 0..=MASK {
            let y = bits_literal(WIDTH, b);
            assert_eq!(outcome(&format!("bit_and({x}, {y})")), bits(WIDTH, a & b));
            assert_eq!(outcome(&format!("bit_or({x}, {y})")), bits(WIDTH, a | b));
            assert_eq!(outcome(&format!("bit_xor({x}, {y})")), bits(WIDTH, a ^ b));
            assert_eq!(outcome(&format!("concat({x}, {y})")), bits(6, a << 3 | b));
            assert_eq!(
                outcome(&format!("{x} == {y}")),
                worth_foundational::expression_api::ExpressionValue::bool(a == b)
            );
            let sum = a + b;
            assert_eq!(
                outcome(&format!("wrapping_add({x}, {y})")),
                bits(WIDTH, sum & MASK)
            );
            let checked = evaluate(&format!("bits_add({x}, {y})")).into_result();
            match checked {
                Ok(value) => assert_eq!((value, sum <= MASK), (bits(WIDTH, sum), true)),
                Err(denial) => {
                    assert!(sum > MASK);
                    assert_eq!(denial.family(), worth_foundational::expression_api::ExpressionDenialFamily::ArithmeticOverflow);
                }
            }
        }
    }
    let denial = evaluate("to_bits<3>(8)")
        .into_result()
        .expect_err("needs four bits");
    assert_eq!(
        denial.family(),
        worth_foundational::expression_api::ExpressionDenialFamily::ArithmeticOverflow
    );
}
