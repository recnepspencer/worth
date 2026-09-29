use std::marker::PhantomData;

pub trait Currency: Copy + std::fmt::Debug + Eq + 'static {
    const CODE: &'static str;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct USD;

impl Currency for USD {
    const CODE: &'static str = "USD";
}

/// A strictly positive transaction amount.
///
/// Signed accounting values use [`SignedMoney`] and cannot inhabit operation
/// inputs that require `Money`.
///
/// ```compile_fail
/// use bank_domain::model::{Money, USD};
///
/// let _ = Money::<USD>::from_signed_minor(-1);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(bound = "")]
pub struct Money<C: Currency> {
    minor_units: i64,
    _currency: PhantomData<C>,
}

impl<C: Currency> Money<C> {
    pub const fn from_minor(minor_units: i64) -> Result<Self, MoneyError> {
        if minor_units <= 0 {
            return Err(MoneyError::NotPositive);
        }
        Ok(Self {
            minor_units,
            _currency: PhantomData,
        })
    }

    pub const fn minor_units(self) -> i64 {
        self.minor_units
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(bound = "")]
pub struct SignedMoney<C: Currency> {
    minor_units: i64,
    _currency: PhantomData<C>,
}

impl<C: Currency> SignedMoney<C> {
    pub const fn from_minor(minor_units: i64) -> Self {
        Self {
            minor_units,
            _currency: PhantomData,
        }
    }

    pub const fn minor_units(self) -> i64 {
        self.minor_units
    }
}

impl<C: Currency> From<Money<C>> for SignedMoney<C> {
    fn from(value: Money<C>) -> Self {
        Self::from_minor(value.minor_units())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MoneyError {
    NotPositive,
}

impl std::fmt::Display for MoneyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("money movement must be positive")
    }
}

impl std::error::Error for MoneyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_money_is_positive_while_accounting_money_is_signed() {
        assert_eq!(Money::<USD>::from_minor(0), Err(MoneyError::NotPositive));
        assert_eq!(Money::<USD>::from_minor(-1), Err(MoneyError::NotPositive));
        assert_eq!(Money::<USD>::from_minor(1).unwrap().minor_units(), 1);
        assert_eq!(SignedMoney::<USD>::from_minor(-1).minor_units(), -1);
        assert_eq!(SignedMoney::<USD>::from_minor(0).minor_units(), 0);
    }
}
