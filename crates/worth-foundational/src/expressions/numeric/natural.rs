//! A minimal arbitrary-precision natural number for exact rounding.
//!
//! Only what exact rational rounding and checked decimals need: construction,
//! addition, multiplication, shifts, comparison, subtraction, and division.
//! Callers bound sizes.

use std::cmp::Ordering;

/// Little-endian `u32` limbs with no trailing zero limbs; zero is empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Natural {
    limbs: Vec<u32>,
}

impl Natural {
    pub(crate) fn from_u128(mut value: u128) -> Self {
        let mut limbs = Vec::new();
        while value != 0 {
            limbs.push(value as u32);
            value >>= 32;
        }
        Self { limbs }
    }

    /// Parses ASCII decimal digits; `None` for any other byte.
    pub(crate) fn from_decimal(digits: &str) -> Option<Self> {
        let mut value = Self::from_u128(0);
        for chunk in digits.as_bytes().chunks(9) {
            let mut part = 0_u32;
            for byte in chunk {
                if !byte.is_ascii_digit() {
                    return None;
                }
                part = part * 10 + u32::from(byte - b'0');
            }
            value.mul_add_small(10_u32.pow(chunk.len() as u32), part);
        }
        Some(value)
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }

    pub(crate) fn bit_len(&self) -> u64 {
        match self.limbs.last() {
            Some(top) => (self.limbs.len() as u64 - 1) * 32 + u64::from(32 - top.leading_zeros()),
            None => 0,
        }
    }

    fn mul_add_small(&mut self, factor: u32, addend: u32) {
        let mut carry = u64::from(addend);
        for limb in &mut self.limbs {
            let product = u64::from(*limb) * u64::from(factor) + carry;
            *limb = product as u32;
            carry = product >> 32;
        }
        if carry != 0 {
            self.limbs.push(carry as u32);
        }
        self.trim();
    }

    pub(crate) fn mul(&self, other: &Self) -> Self {
        if self.is_zero() || other.is_zero() {
            return Self::from_u128(0);
        }
        let mut limbs = vec![0_u32; self.limbs.len() + other.limbs.len()];
        for (i, left) in self.limbs.iter().enumerate() {
            let mut carry = 0_u64;
            for (j, right) in other.limbs.iter().enumerate() {
                let slot = u64::from(limbs[i + j]) + u64::from(*left) * u64::from(*right) + carry;
                limbs[i + j] = slot as u32;
                carry = slot >> 32;
            }
            limbs[i + other.limbs.len()] = carry as u32;
        }
        let mut product = Self { limbs };
        product.trim();
        product
    }

    /// `self^exponent` by repeated squaring.
    pub(crate) fn pow(&self, mut exponent: u32) -> Self {
        let mut result = Self::from_u128(1);
        let mut base = self.clone();
        while exponent != 0 {
            if exponent & 1 == 1 {
                result = result.mul(&base);
            }
            exponent >>= 1;
            if exponent != 0 {
                base = base.mul(&base);
            }
        }
        result
    }

    pub(crate) fn shl(&self, bits: u64) -> Self {
        if self.is_zero() {
            return self.clone();
        }
        let whole = (bits / 32) as usize;
        let part = (bits % 32) as u32;
        let mut limbs = vec![0_u32; whole];
        let mut carry = 0_u32;
        for limb in &self.limbs {
            limbs.push(if part == 0 {
                *limb
            } else {
                (limb << part) | carry
            });
            carry = if part == 0 { 0 } else { limb >> (32 - part) };
        }
        if carry != 0 {
            limbs.push(carry);
        }
        Self { limbs }
    }

    /// `self -= other`; callers guarantee `self >= other`.
    pub(crate) fn sub_assign(&mut self, other: &Self) {
        let mut borrow = 0_i64;
        for (index, limb) in self.limbs.iter_mut().enumerate() {
            let right = i64::from(other.limbs.get(index).copied().unwrap_or(0));
            let mut difference = i64::from(*limb) - right - borrow;
            borrow = i64::from(difference < 0);
            if difference < 0 {
                difference += 1 << 32;
            }
            *limb = difference as u32;
        }
        debug_assert_eq!(borrow, 0, "subtraction underflow");
        self.trim();
    }

    pub(crate) fn add(&self, other: &Self) -> Self {
        let (long, short) = if self.limbs.len() >= other.limbs.len() {
            (self, other)
        } else {
            (other, self)
        };
        let mut carry = 0_u64;
        let mut limbs: Vec<u32> = long
            .limbs
            .iter()
            .enumerate()
            .map(|(index, limb)| {
                let sum = u64::from(*limb)
                    + u64::from(short.limbs.get(index).copied().unwrap_or(0))
                    + carry;
                carry = sum >> 32;
                sum as u32
            })
            .collect();
        if carry != 0 {
            limbs.push(carry as u32);
        }
        Self { limbs }
    }

    /// `(self / divisor, self % divisor)` by binary long division; callers
    /// guarantee a nonzero divisor.
    pub(crate) fn divmod(&self, divisor: &Self) -> (Self, Self) {
        debug_assert!(!divisor.is_zero(), "division by zero");
        let mut remainder = self.clone();
        let mut quotient = vec![0_u32; self.limbs.len()];
        let shift = self.bit_len() as i64 - divisor.bit_len() as i64;
        for bit in (0..=shift.max(-1)).rev() {
            let shifted = divisor.shl(bit as u64);
            if remainder >= shifted {
                remainder.sub_assign(&shifted);
                quotient[(bit / 32) as usize] |= 1 << (bit % 32);
            }
        }
        let mut quotient = Self { limbs: quotient };
        quotient.trim();
        (quotient, remainder)
    }

    /// The value when it fits in 128 bits.
    pub(crate) fn to_u128(&self) -> Option<u128> {
        if self.limbs.len() > 4 {
            return None;
        }
        Some(
            self.limbs
                .iter()
                .rev()
                .fold(0_u128, |value, limb| (value << 32) | u128::from(*limb)),
        )
    }

    pub(crate) fn is_odd(&self) -> bool {
        self.limbs.first().is_some_and(|limb| limb & 1 == 1)
    }

    fn trim(&mut self) {
        while self.limbs.last() == Some(&0) {
            self.limbs.pop();
        }
    }
}

impl Ord for Natural {
    fn cmp(&self, other: &Self) -> Ordering {
        self.limbs
            .len()
            .cmp(&other.limbs.len())
            .then_with(|| self.limbs.iter().rev().cmp(other.limbs.iter().rev()))
    }
}

impl PartialOrd for Natural {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
