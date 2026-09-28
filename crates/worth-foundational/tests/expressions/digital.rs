//! Exhaustive small-width truth tables for `Bits<N>` and `Logic4<N>`,
//! checked against an independent per-symbol oracle written from the
//! language rules: bus literals are most significant bit first, Z is
//! unknown in bitwise operations, and a mux keeps the symbols its arms share.

use worth_foundational::expression_api::{ExpressionDenialFamily as Family, ExpressionValue};

use super::evaluate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Symbol {
    Zero,
    One,
    X,
    Z,
}

use Symbol::{One, Zero, X, Z};

const SYMBOLS: [Symbol; 4] = [Zero, One, X, Z];

impl Symbol {
    fn char(self) -> char {
        match self {
            Zero => '0',
            One => '1',
            X => 'X',
            Z => 'Z',
        }
    }

    fn known(self) -> Option<bool> {
        match self {
            Zero => Some(false),
            One => Some(true),
            X | Z => None,
        }
    }

    fn from_known(bit: bool) -> Self {
        if bit {
            One
        } else {
            Zero
        }
    }

    fn and(self, other: Self) -> Self {
        match (self.known(), other.known()) {
            (Some(false), _) | (_, Some(false)) => Zero,
            (Some(true), Some(true)) => One,
            _ => X,
        }
    }

    fn or(self, other: Self) -> Self {
        match (self.known(), other.known()) {
            (Some(true), _) | (_, Some(true)) => One,
            (Some(false), Some(false)) => Zero,
            _ => X,
        }
    }

    fn xor(self, other: Self) -> Self {
        match (self.known(), other.known()) {
            (Some(a), Some(b)) => Self::from_known(a != b),
            _ => X,
        }
    }

    fn not(self) -> Self {
        self.known().map_or(X, |bit| Self::from_known(!bit))
    }
}

/// Every bus of `width` symbols, most significant first.
fn buses(width: usize) -> Vec<Vec<Symbol>> {
    (0..4_usize.pow(width as u32))
        .map(|mut index| {
            let mut bus = vec![Zero; width];
            for symbol in bus.iter_mut().rev() {
                *symbol = SYMBOLS[index % 4];
                index /= 4;
            }
            bus
        })
        .collect()
}

fn literal(bus: &[Symbol]) -> String {
    format!(
        "logic4(\"{}\")",
        bus.iter().map(|symbol| symbol.char()).collect::<String>()
    )
}

/// The expected value, built from planes rather than parsed.
fn logic(bus: &[Symbol]) -> ExpressionValue {
    let (mut value, mut unknown) = (0_u64, 0_u64);
    for (position, symbol) in bus.iter().rev().enumerate() {
        value |= u64::from(matches!(symbol, One | Z)) << position;
        unknown |= u64::from(matches!(symbol, X | Z)) << position;
    }
    ExpressionValue::logic4(bus.len() as u32, vec![value], vec![unknown]).expect("oracle planes")
}

fn outcome(source: &str) -> ExpressionValue {
    match evaluate(source).into_result() {
        Ok(value) => value,
        Err(denial) => panic!("{source} denied: {denial:?}"),
    }
}

fn zip(a: &[Symbol], b: &[Symbol], op: fn(Symbol, Symbol) -> Symbol) -> Vec<Symbol> {
    a.iter().zip(b).map(|(a, b)| op(*a, *b)).collect()
}

#[test]
fn logic4_bitwise_tables_are_exhaustive_to_width_two() {
    for width in 1..=2 {
        for a in buses(width) {
            let not: Vec<_> = a.iter().map(|symbol| symbol.not()).collect();
            assert_eq!(outcome(&format!("bit_not({})", literal(&a))), logic(&not));
            for b in buses(width) {
                let (x, y) = (literal(&a), literal(&b));
                let cases: [(&str, fn(Symbol, Symbol) -> Symbol); 3] = [
                    ("bit_and", Symbol::and),
                    ("bit_or", Symbol::or),
                    ("bit_xor", Symbol::xor),
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
                    ExpressionValue::bool(a == b)
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

fn bits_literal(width: u32, value: u64) -> String {
    format!(
        "bits<{width}>(\"{value:0width$b}\")",
        width = width as usize
    )
}

fn bits(width: u32, value: u64) -> ExpressionValue {
    ExpressionValue::bits(width, vec![value]).expect("oracle bits")
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
            ExpressionValue::integer(a.into())
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
                ExpressionValue::bool(a == b)
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
                    assert_eq!(denial.family(), Family::ArithmeticOverflow);
                }
            }
        }
    }
    let denial = evaluate("to_bits<3>(8)")
        .into_result()
        .expect_err("needs four bits");
    assert_eq!(denial.family(), Family::ArithmeticOverflow);
}
