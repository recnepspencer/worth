use std::marker::PhantomData;

pub trait FreshnessClass: 'static {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentValidity;
impl FreshnessClass for CurrentValidity {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaleReadable;
impl FreshnessClass for StaleReadable {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RebindRequired;
impl FreshnessClass for RebindRequired {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthorityRevalidationRequired;
impl FreshnessClass for AuthorityRevalidationRequired {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshnessScopedBasis<F, B> {
    basis: B,
    freshness: PhantomData<F>,
}

impl<F, B> FreshnessScopedBasis<F, B>
where
    F: FreshnessClass,
{
    pub(crate) fn new(basis: B) -> Self {
        Self {
            basis,
            freshness: PhantomData,
        }
    }

    pub fn basis(&self) -> &B {
        &self.basis
    }

    pub fn into_basis(self) -> B {
        self.basis
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::assumption::{
        AssumptionBasis, CurrentValidity, FreshnessScopedBasis, StaleReadable,
    };

    const _: () = assert!(
        size_of::<FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>()
            == size_of::<AssumptionBasis<u8>>()
    );
    const _: () = assert!(
        size_of::<FreshnessScopedBasis<StaleReadable, AssumptionBasis<u8>>>()
            == size_of::<AssumptionBasis<u8>>()
    );
}
