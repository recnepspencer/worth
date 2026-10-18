use crate::assumption::{AssumptionBasis, CurrentValidity, FreshnessScopedBasis};
use crate::recipe::{Lowered, Recipe};

#[derive(Debug, PartialEq, Eq)]
pub struct ExecutionReadyRecipe<T, A> {
    pub(crate) lowered: Recipe<Lowered, T, A>,
}

impl<T, A> ExecutionReadyRecipe<T, A> {
    pub(crate) fn new(lowered: Recipe<Lowered, T, A>) -> Self {
        Self { lowered }
    }

    pub fn payload(&self) -> &T {
        self.lowered.payload()
    }

    pub fn basis(&self) -> &A {
        self.lowered.basis()
    }

    pub fn into_parts(self) -> (T, A) {
        self.lowered.into_parts()
    }
}

impl<T, B> ExecutionReadyRecipe<T, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<B>>> {
    pub fn strong_basis(&self) -> &AssumptionBasis<B> {
        self.basis().basis()
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::assumption::{AssumptionBasis, CurrentValidity, FreshnessScopedBasis};
    use crate::recipe::{ExecutionReadyRecipe, Lowered, Recipe};

    const _: () = {
        type LoweredRecipe =
            Recipe<Lowered, u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>;

        assert!(
            size_of::<
                ExecutionReadyRecipe<
                    u64,
                    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
                >,
            >() == size_of::<LoweredRecipe>()
        );
    };
}
