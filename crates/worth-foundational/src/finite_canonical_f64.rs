use std::cmp::Ordering;

use serde::{Deserialize, Deserializer, Serialize};

use crate::CanonicalF64;

/// Finite IEEE-754 binary64 in numeric order with one representation of zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct FiniteCanonicalF64(u64);

impl FiniteCanonicalF64 {
    /// Admits a finite value and normalizes signed zero.
    pub const fn from_f64(value: f64) -> Option<Self> {
        if value.is_finite() {
            Some(Self(Self::normalize_zero_bits(value.to_bits())))
        } else {
            None
        }
    }

    /// Admits canonical wire bits. Negative zero and non-finite values refuse.
    pub const fn from_wire_bits(bits: u64) -> Option<Self> {
        if bits == (-0.0_f64).to_bits() {
            None
        } else {
            Self::from_f64(f64::from_bits(bits))
        }
    }

    pub const fn bits(self) -> u64 {
        self.0
    }

    pub const fn as_f64(self) -> f64 {
        f64::from_bits(self.0)
    }

    pub const fn normalize_zero_bits(bits: u64) -> u64 {
        if bits == (-0.0_f64).to_bits() {
            0
        } else {
            bits
        }
    }
}

impl Ord for FiniteCanonicalF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_f64().total_cmp(&other.as_f64())
    }
}

impl PartialOrd for FiniteCanonicalF64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<FiniteCanonicalF64> for CanonicalF64 {
    fn from(value: FiniteCanonicalF64) -> Self {
        Self(value.bits())
    }
}

impl<'de> Deserialize<'de> for FiniteCanonicalF64 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_wire_bits(u64::deserialize(deserializer)?)
            .ok_or_else(|| serde::de::Error::custom("expected canonical finite f64 bits"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_values_order_numerically_and_signed_zero_has_one_encoding() {
        let negative = FiniteCanonicalF64::from_f64(-2.0).unwrap();
        let zero = FiniteCanonicalF64::from_f64(-0.0).unwrap();
        let positive = FiniteCanonicalF64::from_f64(2.0).unwrap();
        assert!(negative < zero && zero < positive);
        assert_eq!(zero.bits(), 0);
        assert_eq!(
            rmp_serde::from_slice::<FiniteCanonicalF64>(&rmp_serde::to_vec(&negative).unwrap())
                .unwrap(),
            negative
        );
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(FiniteCanonicalF64::from_f64(invalid).is_none());
        }
        for invalid in [-0.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let bits = invalid.to_bits();
            assert!(FiniteCanonicalF64::from_wire_bits(bits).is_none());
            assert!(rmp_serde::from_slice::<FiniteCanonicalF64>(
                &rmp_serde::to_vec(&bits).unwrap()
            )
            .is_err());
        }
    }
}
