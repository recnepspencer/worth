//! `Bits<N>` and `Logic4<N>` semantics over little-endian 64-bit limbs.
//!
//! Bit 0 is the least significant bit of limb 0; no plane holds a bit past
//! its width. A `Logic4` bus is a value plane and an unknown plane: 0 is
//! (0, 0), 1 is (1, 0), X is (0, 1), and Z is (1, 1). Bitwise operations
//! treat Z as unknown and produce X for every unknown result.

use crate::expressions::denial::{ExpressionDenial, ExpressionDenialDetail, ExpressionResult};

/// A bus plane: `ceil(width / 64)` limbs.
pub(crate) type Plane = Vec<u64>;

fn limbs(width: u32) -> usize {
    (width as usize).div_ceil(64)
}

/// The used bits of limb `index` of a `width`-bit plane.
fn mask(width: u32, index: usize) -> u64 {
    let used = width as usize - index * 64;
    if used >= 64 {
        u64::MAX
    } else {
        (1 << used) - 1
    }
}

fn masked(width: u32, mut plane: Plane) -> Plane {
    for (index, limb) in plane.iter_mut().enumerate() {
        *limb &= mask(width, index);
    }
    plane
}

fn zip(a: &[u64], b: &[u64], f: impl Fn(u64, u64) -> u64) -> Plane {
    a.iter().zip(b).map(|(a, b)| f(*a, *b)).collect()
}

pub(crate) fn and(a: &[u64], b: &[u64]) -> Plane {
    zip(a, b, |a, b| a & b)
}

pub(crate) fn or(a: &[u64], b: &[u64]) -> Plane {
    zip(a, b, |a, b| a | b)
}

pub(crate) fn xor(a: &[u64], b: &[u64]) -> Plane {
    zip(a, b, |a, b| a ^ b)
}

pub(crate) fn not(width: u32, a: &[u64]) -> Plane {
    masked(width, a.iter().map(|limb| !limb).collect())
}

/// A `Logic4` bus as its value and unknown planes.
pub(crate) type Logic<'a> = (&'a [u64], &'a [u64]);

fn logic_planes(
    width: u32,
    (va, ua): Logic,
    (vb, ub): Logic,
    f: impl Fn([u64; 4]) -> (u64, u64),
) -> (Plane, Plane) {
    (0..limbs(width))
        .map(|index| {
            let (value, known) = f([va[index], ua[index], vb[index], ub[index]]);
            let unknown = !known & mask(width, index);
            (value & known, unknown)
        })
        .unzip()
}

/// 0 AND anything is 0; 1 AND 1 is 1; everything else is X.
pub(crate) fn logic_and(width: u32, a: Logic, b: Logic) -> (Plane, Plane) {
    logic_planes(width, a, b, |[va, ua, vb, ub]| {
        let zero = (!va & !ua) | (!vb & !ub);
        let one = va & !ua & vb & !ub;
        (one, zero | one)
    })
}

/// 1 OR anything is 1; 0 OR 0 is 0; everything else is X.
pub(crate) fn logic_or(width: u32, a: Logic, b: Logic) -> (Plane, Plane) {
    logic_planes(width, a, b, |[va, ua, vb, ub]| {
        let one = (va & !ua) | (vb & !ub);
        let zero = !va & !ua & !vb & !ub;
        (one, zero | one)
    })
}

/// Known operands use binary truth; any unknown operand gives X.
pub(crate) fn logic_xor(width: u32, a: Logic, b: Logic) -> (Plane, Plane) {
    logic_planes(width, a, b, |[va, ua, vb, ub]| (va ^ vb, !ua & !ub))
}

/// 0 and 1 flip; X and Z give X.
pub(crate) fn logic_not(width: u32, (value, unknown): Logic) -> (Plane, Plane) {
    let value = zip(value, unknown, |value, unknown| !value & !unknown);
    (masked(width, value), unknown.to_vec())
}

/// Logical equality as one `Logic4` bit: any known mismatch is 0, otherwise
/// any X or Z is X, otherwise 1.
pub(crate) fn logic_eq((va, ua): Logic, (vb, ub): Logic) -> (u64, u64) {
    let mismatch = (0..va.len()).any(|i| (va[i] ^ vb[i]) & !ua[i] & !ub[i] != 0);
    let unknown = ua.iter().chain(ub).any(|limb| *limb != 0);
    match (mismatch, unknown) {
        (true, _) => (0, 0),
        (false, true) => (0, 1),
        (false, false) => (1, 0),
    }
}

/// `mux(select, one, zero)`: a known select chooses its arm; an unknown
/// select keeps the symbols the arms share and gives X where they differ.
pub(crate) fn mux(width: u32, select: (u64, u64), one: Logic, zero: Logic) -> (Plane, Plane) {
    match select {
        (1, 0) => (one.0.to_vec(), one.1.to_vec()),
        (0, 0) => (zero.0.to_vec(), zero.1.to_vec()),
        _ => (0..limbs(width))
            .map(|i| {
                let same = !(one.0[i] ^ zero.0[i]) & !(one.1[i] ^ zero.1[i]);
                let unknown = (one.1[i] & same) | (!same & mask(width, i));
                (one.0[i] & same, unknown)
            })
            .unzip(),
    }
}

fn overflow(reason: &'static str) -> ExpressionDenial {
    ExpressionDenial::new(ExpressionDenialDetail::ArithmeticOverflow(reason))
}

/// Bits `[low, low + width)` of `plane`, as a `width`-bit plane.
pub(crate) fn slice(plane: &[u64], low: u32, width: u32) -> Plane {
    let (skip, shift) = ((low / 64) as usize, low % 64);
    let out = (0..limbs(width)).map(|i| {
        let lower = plane.get(skip + i).copied().unwrap_or(0) >> shift;
        let upper = match shift {
            0 => 0,
            _ => plane.get(skip + i + 1).copied().unwrap_or(0) << (64 - shift),
        };
        lower | upper
    });
    masked(width, out.collect())
}

/// `plane` moved `count` bits toward the high end, zero-filled, as a
/// `width`-bit plane; bits past `width` drop.
fn raised(plane: &[u64], count: u32, width: u32) -> Plane {
    let (skip, shift) = ((count / 64) as usize, count % 64);
    let out = (0..limbs(width)).map(|i| {
        let at = |j: usize| -> u64 {
            i.checked_sub(skip + j)
                .and_then(|index| plane.get(index))
                .copied()
                .unwrap_or(0)
        };
        match shift {
            0 => at(0),
            _ => (at(0) << shift) | (at(1) >> (64 - shift)),
        }
    });
    masked(width, out.collect())
}

fn shift_count(width: u32, count: i128) -> ExpressionResult<Option<u32>> {
    if count < 0 {
        return Err(ExpressionDenial::new(
            ExpressionDenialDetail::ArithmeticDomain("shift counts are nonnegative"),
        ));
    }
    Ok(u32::try_from(count).ok().filter(|count| *count < width))
}

/// Logical shift toward the high end; counts of at least `width` give zero.
pub(crate) fn shift_left(width: u32, plane: &[u64], count: i128) -> ExpressionResult<Plane> {
    Ok(match shift_count(width, count)? {
        Some(count) => raised(plane, count, width),
        None => vec![0; limbs(width)],
    })
}

/// Logical shift toward bit 0; counts of at least `width` give zero.
pub(crate) fn shift_right(width: u32, plane: &[u64], count: i128) -> ExpressionResult<Plane> {
    Ok(match shift_count(width, count)? {
        Some(count) => slice(plane, count, width - count),
        None => vec![0; limbs(width)],
    }
    .into_iter()
    .chain(std::iter::repeat(0))
    .take(limbs(width))
    .collect())
}

/// `concat(high, low)`: `low` supplies bits `[0, low_width)`.
pub(crate) fn concat(high: &[u64], low: &[u64], low_width: u32, width: u32) -> Plane {
    let mut out = raised(high, low_width, width);
    for (limb, low) in out.iter_mut().zip(low) {
        *limb |= low;
    }
    out
}

/// Zero-fills the high bits up to `width`.
pub(crate) fn extend(plane: &[u64], width: u32) -> Plane {
    let mut out = plane.to_vec();
    out.resize(limbs(width), 0);
    out
}

/// The low `width` bits of a nonnegative integer; a negative integer or one
/// that needs more bits denies.
pub(crate) fn to_bits(value: i128, width: u32) -> ExpressionResult<Plane> {
    let value =
        u128::try_from(value).map_err(|_| overflow("to_bits needs a nonnegative integer"))?;
    if width < 128 && value >> width != 0 {
        return Err(overflow("integer does not fit the bit width"));
    }
    let limbs = [value as u64, (value >> 64) as u64];
    Ok(extend(&limbs[..limbs.len().min(self::limbs(width))], width))
}

/// The unsigned value of a plane, or `None` above `u128`.
pub(crate) fn to_unsigned(plane: &[u64]) -> Option<u128> {
    if plane.iter().skip(2).any(|limb| *limb != 0) {
        return None;
    }
    let high = plane.get(1).copied().unwrap_or(0);
    Some(u128::from(plane[0]) | (u128::from(high) << 64))
}

/// Unsigned addition: `bits_add` denies a carry out of `width`;
/// `wrapping_add` drops it.
pub(crate) fn add(width: u32, a: &[u64], b: &[u64], wrapping: bool) -> ExpressionResult<Plane> {
    let mut carry = false;
    let mut out: Plane = a
        .iter()
        .zip(b)
        .map(|(a, b)| {
            let (sum, first) = a.overflowing_add(*b);
            let (sum, second) = sum.overflowing_add(u64::from(carry));
            carry = first || second;
            sum
        })
        .collect();
    let spilled = carry
        || out
            .last()
            .is_some_and(|top| top & !mask(width, out.len() - 1) != 0);
    if spilled && !wrapping {
        return Err(overflow("bits_add carries out of the bus width"));
    }
    out = masked(width, out);
    Ok(out)
}

#[cfg(test)]
#[path = "digital_tests.rs"]
mod tests;
