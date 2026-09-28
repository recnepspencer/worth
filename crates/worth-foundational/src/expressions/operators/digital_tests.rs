//! Digital semantics against an independent per-bit reference: exhaustive
//! four-state truth tables at small widths, and bit-vector arithmetic across
//! limb boundaries.

use super::{
    add, and, concat, extend, logic_and, logic_eq, logic_not, logic_or, logic_xor, mux, not, or,
    shift_left, shift_right, slice, to_bits, to_unsigned, xor, Plane,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Symbol {
    Zero,
    One,
    X,
    Z,
}

use Symbol::{One, Zero, X, Z};

const SYMBOLS: [Symbol; 4] = [Zero, One, X, Z];

fn known(symbol: Symbol) -> Option<bool> {
    match symbol {
        Zero => Some(false),
        One => Some(true),
        X | Z => None,
    }
}

fn symbol(value: Option<bool>) -> Symbol {
    match value {
        Some(false) => Zero,
        Some(true) => One,
        None => X,
    }
}

fn reference_and(a: Symbol, b: Symbol) -> Symbol {
    match (known(a), known(b)) {
        (Some(false), _) | (_, Some(false)) => Zero,
        (Some(true), Some(true)) => One,
        _ => X,
    }
}

fn reference_or(a: Symbol, b: Symbol) -> Symbol {
    match (known(a), known(b)) {
        (Some(true), _) | (_, Some(true)) => One,
        (Some(false), Some(false)) => Zero,
        _ => X,
    }
}

fn reference_xor(a: Symbol, b: Symbol) -> Symbol {
    symbol(known(a).zip(known(b)).map(|(a, b)| a ^ b))
}

fn reference_not(a: Symbol) -> Symbol {
    symbol(known(a).map(|a| !a))
}

fn reference_mux(select: Symbol, one: Symbol, zero: Symbol) -> Symbol {
    match known(select) {
        Some(true) => one,
        Some(false) => zero,
        None if one == zero => one,
        None => X,
    }
}

/// Every `width`-symbol bus, bit 0 first.
fn buses(width: u32) -> Vec<Vec<Symbol>> {
    (0..4_usize.pow(width))
        .map(|index| {
            (0..width)
                .map(|bit| SYMBOLS[index / 4_usize.pow(bit) % 4])
                .collect()
        })
        .collect()
}

fn encode(bus: &[Symbol]) -> (Plane, Plane) {
    let (mut value, mut unknown) = (0_u64, 0_u64);
    for (bit, symbol) in bus.iter().enumerate() {
        let (v, u) = match symbol {
            Zero => (0, 0),
            One => (1, 0),
            X => (0, 1),
            Z => (1, 1),
        };
        value |= v << bit;
        unknown |= u << bit;
    }
    (vec![value], vec![unknown])
}

fn decode(width: u32, (value, unknown): &(Plane, Plane)) -> Vec<Symbol> {
    let unused = !super::mask(width, 0);
    assert_eq!((value[0] | unknown[0]) & unused, 0, "no bit past the width");
    (0..width)
        .map(
            |bit| match ((value[0] >> bit) & 1, (unknown[0] >> bit) & 1) {
                (0, 0) => Zero,
                (1, 0) => One,
                (0, 1) => X,
                _ => Z,
            },
        )
        .collect()
}

fn elementwise(a: &[Symbol], b: &[Symbol], f: fn(Symbol, Symbol) -> Symbol) -> Vec<Symbol> {
    a.iter().zip(b).map(|(a, b)| f(*a, *b)).collect()
}

#[test]
fn four_state_bitwise_operations_match_the_truth_tables() {
    for width in 1..=3 {
        for a in buses(width) {
            let pa = encode(&a);
            let la = (&pa.0[..], &pa.1[..]);
            let not_a: Vec<Symbol> = a.iter().map(|a| reference_not(*a)).collect();
            assert_eq!(decode(width, &logic_not(width, la)), not_a);
            for b in buses(width) {
                let pb = encode(&b);
                let lb = (&pb.0[..], &pb.1[..]);
                let and = logic_and(width, la, lb);
                assert_eq!(decode(width, &and), elementwise(&a, &b, reference_and));
                let or = logic_or(width, la, lb);
                assert_eq!(decode(width, &or), elementwise(&a, &b, reference_or));
                let xor = logic_xor(width, la, lb);
                assert_eq!(decode(width, &xor), elementwise(&a, &b, reference_xor));
                let mismatch = a
                    .iter()
                    .zip(&b)
                    .any(|(a, b)| matches!((known(*a), known(*b)), (Some(a), Some(b)) if a != b));
                let unknown = a.iter().chain(&b).any(|s| known(*s).is_none());
                let expected = match (mismatch, unknown) {
                    (true, _) => Zero,
                    (false, true) => X,
                    (false, false) => One,
                };
                let (v, u) = logic_eq(la, lb);
                assert_eq!(decode(1, &(vec![v], vec![u])), vec![expected]);
            }
        }
    }
}

#[test]
fn unknown_select_preserves_identical_arm_symbols() {
    for width in 1..=2 {
        for select in SYMBOLS {
            let (sv, su) = encode(&[select]);
            for one in buses(width) {
                let po = encode(&one);
                for zero in buses(width) {
                    let pz = encode(&zero);
                    let got = mux(width, (sv[0], su[0]), (&po.0, &po.1), (&pz.0, &pz.1));
                    let expected: Vec<Symbol> = one
                        .iter()
                        .zip(&zero)
                        .map(|(one, zero)| reference_mux(select, *one, *zero))
                        .collect();
                    assert_eq!(decode(width, &got), expected, "{select:?} {one:?} {zero:?}");
                }
            }
        }
    }
}

/// A deterministic bit stream for wide-bus cases.
fn bits(seed: &mut u64, width: u32) -> Vec<bool> {
    (0..width)
        .map(|_| {
            *seed ^= *seed << 13;
            *seed ^= *seed >> 7;
            *seed ^= *seed << 17;
            *seed & 1 == 1
        })
        .collect()
}

fn pack(bits: &[bool]) -> Plane {
    let mut plane = vec![0; bits.len().div_ceil(64)];
    for (index, bit) in bits.iter().enumerate() {
        plane[index / 64] |= u64::from(*bit) << (index % 64);
    }
    plane
}

fn map2(a: &[bool], b: &[bool], f: fn(bool, bool) -> bool) -> Vec<bool> {
    a.iter().zip(b).map(|(a, b)| f(*a, *b)).collect()
}

#[test]
fn wide_buses_match_the_bit_vector_reference() {
    let mut seed = 0x9e37_79b9_7f4a_7c15;
    for width in [1_u32, 7, 63, 64, 65, 127, 128, 129, 200] {
        let (a, b) = (bits(&mut seed, width), bits(&mut seed, width));
        let (pa, pb) = (pack(&a), pack(&b));
        assert_eq!(and(&pa, &pb), pack(&map2(&a, &b, |a, b| a & b)));
        assert_eq!(or(&pa, &pb), pack(&map2(&a, &b, |a, b| a | b)));
        assert_eq!(xor(&pa, &pb), pack(&map2(&a, &b, |a, b| a ^ b)));
        let inverted: Vec<bool> = a.iter().map(|a| !a).collect();
        assert_eq!(not(width, &pa), pack(&inverted));
        for count in [0, 1, 5, 63, 64, 65, width - 1, width, width + 9] {
            let left: Vec<bool> = (0..width)
                .map(|i| i.checked_sub(count).is_some_and(|j| a[j as usize]))
                .collect();
            let right: Vec<bool> = (0..width)
                .map(|i| a.get((i + count) as usize).copied().unwrap_or(false))
                .collect();
            let count = i128::from(count);
            assert_eq!(shift_left(width, &pa, count).unwrap(), pack(&left));
            assert_eq!(shift_right(width, &pa, count).unwrap(), pack(&right));
        }
        assert!(shift_left(width, &pa, -1).is_err());
        let third = width / 3;
        for (low, high) in [
            (0, width),
            (0, 1),
            (width - 1, width),
            (third, 2 * third + 1),
        ] {
            assert_eq!(
                slice(&pa, low, high - low),
                pack(&a[low as usize..high as usize])
            );
        }
        let joined: Vec<bool> = b.iter().chain(&a).copied().collect();
        assert_eq!(concat(&pa, &pb, width, 2 * width), pack(&joined));
        let wide: Vec<bool> = a.iter().copied().chain([false; 70]).collect();
        assert_eq!(extend(&pa, width + 70), pack(&wide));
    }
}

#[test]
fn bit_addition_checks_or_wraps_the_carry() {
    let mut seed = 0x2545_f491_4f6c_dd1d;
    for width in [1, 8, 64, 65, 130] {
        for _ in 0..16 {
            let (a, b) = (bits(&mut seed, width), bits(&mut seed, width));
            let mut carry = false;
            let sum: Vec<bool> = a
                .iter()
                .zip(&b)
                .map(|(a, b)| {
                    let bit = a ^ b ^ carry;
                    carry = (a & b) | (carry & (a ^ b));
                    bit
                })
                .collect();
            let (pa, pb) = (pack(&a), pack(&b));
            assert_eq!(add(width, &pa, &pb, true).unwrap(), pack(&sum));
            let checked = add(width, &pa, &pb, false).ok();
            assert_eq!(checked, (!carry).then(|| pack(&sum)));
        }
    }
}

#[test]
fn integers_convert_to_and_from_buses() {
    assert_eq!(to_bits(5, 3).unwrap(), vec![5]);
    assert!(to_bits(8, 3).is_err());
    assert!(to_bits(-1, 8).is_err());
    let widest = vec![u64::MAX, u64::MAX >> 1, 0, 0];
    assert_eq!(to_bits(i128::MAX, 200).unwrap(), widest);
    assert!(to_bits(1 << 100, 100).is_err());
    assert_eq!(to_unsigned(&[7, 1]), Some(7 | (1 << 64)));
    assert_eq!(to_unsigned(&[0, 0, 1]), None);
    assert_eq!(to_unsigned(&[0, 0, 0]), Some(0));
}
