use std::marker::PhantomData;

use crate::assumption::NoAssumptionBasis;

pub trait RecipeStageMarker: 'static {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unresolved;
impl RecipeStageMarker for Unresolved {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resolved;
impl RecipeStageMarker for Resolved {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lowered;
impl RecipeStageMarker for Lowered {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Admitted;
impl RecipeStageMarker for Admitted {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recipe<S, T, A = NoAssumptionBasis>
where
    S: RecipeStageMarker,
{
    pub(crate) payload: T,
    pub(crate) basis: A,
    pub(crate) stage: PhantomData<S>,
}

impl<T> Recipe<Unresolved, T, NoAssumptionBasis> {
    pub fn new(payload: T) -> Self {
        Self {
            payload,
            basis: NoAssumptionBasis,
            stage: PhantomData,
        }
    }
}

impl<S, T, A> Recipe<S, T, A>
where
    S: RecipeStageMarker,
{
    pub(crate) fn with_stage(payload: T, basis: A) -> Self {
        Self {
            payload,
            basis,
            stage: PhantomData,
        }
    }

    pub fn payload(&self) -> &T {
        &self.payload
    }

    pub fn basis(&self) -> &A {
        &self.basis
    }

    pub fn into_parts(self) -> (T, A) {
        (self.payload, self.basis)
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::assumption::{
        AssumptionBasis, CurrentValidity, FreshnessScopedBasis, NoAssumptionBasis,
    };

    use super::{Admitted, Recipe, Resolved, Unresolved};

    #[test]
    fn unresolved_recipe_uses_empty_basis_default() {
        let recipe = Recipe::<Unresolved, _>::new("payload");

        assert_eq!(recipe.payload(), &"payload");
        assert_eq!(recipe.basis(), &NoAssumptionBasis);
    }

    const _: () = assert!(size_of::<Recipe<Unresolved, u64>>() == size_of::<u64>());
    const _: () = assert!(
        size_of::<Recipe<Resolved, u64, AssumptionBasis<u8>>>()
            == size_of::<(u64, AssumptionBasis<u8>)>()
    );

    #[test]
    fn recipe_clone_preserves_stage_payload_and_basis() {
        let admitted = Recipe::<Admitted, _, _>::with_stage(
            "payload",
            FreshnessScopedBasis::<CurrentValidity, _>::new(AssumptionBasis::new(11_u8)),
        );

        let cloned = admitted.clone();

        assert_eq!(cloned.payload(), admitted.payload());
        assert_eq!(
            cloned.basis().basis().value(),
            admitted.basis().basis().value()
        );
    }
}
