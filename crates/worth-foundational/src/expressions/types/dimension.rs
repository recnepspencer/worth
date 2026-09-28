//! Type-level physical dimensions: integer exponents over the seven SI base
//! dimensions plus angle.

use std::fmt;

/// Exponents stay within `-16..=16`; products that leave the range deny.
const EXPONENT_BOUND: i8 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BaseDimension {
    Length,
    Mass,
    Time,
    Current,
    Temperature,
    Amount,
    LuminousIntensity,
    Angle,
}

impl BaseDimension {
    pub(crate) const ALL: [Self; 8] = [
        Self::Length,
        Self::Mass,
        Self::Time,
        Self::Current,
        Self::Temperature,
        Self::Amount,
        Self::LuminousIntensity,
        Self::Angle,
    ];

    /// The name authored in `Quantity<...>` type arguments.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Length => "length",
            Self::Mass => "mass",
            Self::Time => "time",
            Self::Current => "current",
            Self::Temperature => "temperature",
            Self::Amount => "amount",
            Self::LuminousIntensity => "luminous_intensity",
            Self::Angle => "angle",
        }
    }

    pub(crate) fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|base| base.name() == name)
    }
}

/// A dimension exponent vector, indexed in [`BaseDimension`] order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ExpressionDimension {
    exponents: [i8; 8],
}

impl ExpressionDimension {
    pub const DIMENSIONLESS: Self = Self { exponents: [0; 8] };

    /// Returns `None` when an exponent is outside `-16..=16`.
    pub fn new(exponents: [i8; 8]) -> Option<Self> {
        exponents
            .iter()
            .all(|exponent| exponent.abs() <= EXPONENT_BOUND)
            .then_some(Self { exponents })
    }

    /// Catalog rows pass small literal exponents, well inside the bound.
    pub(crate) const fn from_catalog(exponents: [i8; 8]) -> Self {
        Self { exponents }
    }

    pub const fn base(base: BaseDimension) -> Self {
        let mut exponents = [0; 8];
        exponents[base as usize] = 1;
        Self { exponents }
    }

    pub const fn exponents(self) -> [i8; 8] {
        self.exponents
    }

    pub fn exponent(self, base: BaseDimension) -> i8 {
        self.exponents[base as usize]
    }

    pub fn is_dimensionless(self) -> bool {
        self == Self::DIMENSIONLESS
    }

    /// The dimension of a product; `None` when an exponent leaves the range.
    pub fn multiply(self, other: Self) -> Option<Self> {
        self.combine(other, i8::checked_add)
    }

    /// The dimension of a quotient; `None` when an exponent leaves the range.
    pub fn divide(self, other: Self) -> Option<Self> {
        self.combine(other, i8::checked_sub)
    }

    /// The dimension of a square root; `None` when an exponent is odd.
    pub fn square_root(self) -> Option<Self> {
        let mut exponents = self.exponents;
        for exponent in &mut exponents {
            if *exponent % 2 != 0 {
                return None;
            }
            *exponent /= 2;
        }
        Some(Self { exponents })
    }

    fn combine(self, other: Self, op: fn(i8, i8) -> Option<i8>) -> Option<Self> {
        let mut exponents = [0; 8];
        for (index, exponent) in exponents.iter_mut().enumerate() {
            *exponent = op(self.exponents[index], other.exponents[index])?;
        }
        Self::new(exponents)
    }
}

impl fmt::Display for ExpressionDimension {
    /// Renders the authored type-argument form, such as `length*length/time`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_dimensionless() {
            return f.write_str("dimensionless");
        }
        let mut first = true;
        for (sign, separator) in [(1, "*"), (-1, "/")] {
            for base in BaseDimension::ALL {
                let exponent = self.exponent(base) * sign;
                for _ in 0..exponent.max(0) {
                    let prefix = match (first, sign) {
                        (true, 1) => "",
                        (true, _) => "1/",
                        _ => separator,
                    };
                    write!(f, "{prefix}{}", base.name())?;
                    first = false;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{BaseDimension, ExpressionDimension};

    #[test]
    fn renders_authored_products() {
        let length = ExpressionDimension::base(BaseDimension::Length);
        let time = ExpressionDimension::base(BaseDimension::Time);
        let area = length.multiply(length).unwrap();
        assert_eq!(area.divide(time).unwrap().to_string(), "length*length/time");
        let frequency = ExpressionDimension::DIMENSIONLESS.divide(time).unwrap();
        assert_eq!(frequency.to_string(), "1/time");
        assert_eq!(area.square_root(), Some(length));
        assert_eq!(length.square_root(), None);
    }

    #[test]
    fn exponents_are_bounded() {
        let mut exponents = [0; 8];
        exponents[0] = 16;
        let edge = ExpressionDimension::new(exponents).unwrap();
        let length = ExpressionDimension::base(BaseDimension::Length);
        assert_eq!(edge.multiply(length), None);
        exponents[0] = 17;
        assert_eq!(ExpressionDimension::new(exponents), None);
    }
}
