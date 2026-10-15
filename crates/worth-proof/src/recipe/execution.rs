use crate::assumption::{AssumptionBasis, CurrentValidity, FreshnessScopedBasis};
use crate::recipe::ExecutionReadyRecipe;

#[derive(Debug, PartialEq, Eq)]
pub struct ExecutedRecipe<T, A> {
    pub(crate) ready: ExecutionReadyRecipe<T, A>,
}

impl<T, A> ExecutedRecipe<T, A> {
    pub(crate) fn new(ready: ExecutionReadyRecipe<T, A>) -> Self {
        Self { ready }
    }

    pub fn payload(&self) -> &T {
        self.ready.payload()
    }

    pub fn basis(&self) -> &A {
        self.ready.basis()
    }

    pub fn into_parts(self) -> (T, A) {
        self.ready.into_parts()
    }
}

impl<T, B> ExecutedRecipe<T, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<B>>> {
    pub fn strong_basis(&self) -> &AssumptionBasis<B> {
        self.basis().basis()
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::assumption::{AssumptionBasis, CurrentValidity, FreshnessScopedBasis};
    use crate::recipe::{ExecutedRecipe, ExecutionReadyRecipe};

    const _: () = {
        type ReadyRecipe =
            ExecutionReadyRecipe<u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>;

        assert!(
            size_of::<
                ExecutedRecipe<u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>,
            >() == size_of::<ReadyRecipe>()
        );
    };
}
