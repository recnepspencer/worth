use crate::recipe::{ExecutionReadyRecipe, Lowered, Recipe};
use crate::transition::{
    compose_join_success_transition, SuccessfulTransitionOutcome, TransitionOutcome,
};

use super::JoinInputs2;

pub type JoinedExecutionReadyRecipe<L, R, LA, RA> =
    ExecutionReadyRecipe<JoinInputs2<L, R>, JoinInputs2<LA, RA>>;

pub type JoinedExecutionReadyOutcome<L, R, LA, RA, D, De, St, Rb, F> =
    TransitionOutcome<JoinedExecutionReadyRecipe<L, R, LA, RA>, D, De, St, Rb, F>;

pub fn join_ready_recipe_pair<L, R, LA, RA>(
    inputs: JoinInputs2<ExecutionReadyRecipe<L, LA>, ExecutionReadyRecipe<R, RA>>,
) -> JoinedExecutionReadyRecipe<L, R, LA, RA> {
    let (left, right) = inputs.into_parts();
    let (left_payload, left_basis) = left.into_parts();
    let (right_payload, right_basis) = right.into_parts();

    ExecutionReadyRecipe::new(Recipe::<Lowered, _, _>::with_stage(
        JoinInputs2::new(left_payload, right_payload),
        JoinInputs2::new(left_basis, right_basis),
    ))
}

pub fn compose_join_ready_recipe_pair<L, R, LA, RA, D, De, St, Rb, F>(
    left: TransitionOutcome<ExecutionReadyRecipe<L, LA>, D, De, St, Rb, F>,
    right: impl FnOnce() -> TransitionOutcome<ExecutionReadyRecipe<R, RA>, D, De, St, Rb, F>,
) -> JoinedExecutionReadyOutcome<L, R, LA, RA, D, De, St, Rb, F> {
    compose_join_success_transition(left, right, |inputs| {
        SuccessfulTransitionOutcome::new(join_ready_recipe_pair(inputs))
    })
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::assumption::{AssumptionBasis, CurrentValidity, FreshnessScopedBasis};
    use crate::recipe::{ExecutionReadyRecipe, Lowered, Recipe};

    use super::{compose_join_ready_recipe_pair, join_ready_recipe_pair};
    use crate::composition::JoinInputs2;
    use crate::transition::TransitionOutcome;

    #[test]
    fn ready_recipe_join_preserves_explicit_payload_and_basis_positions() {
        let left = ExecutionReadyRecipe::new(Recipe::<Lowered, _, _>::with_stage(
            "left",
            FreshnessScopedBasis::<CurrentValidity, _>::new(AssumptionBasis::new(3_u8)),
        ));
        let right = ExecutionReadyRecipe::new(Recipe::<Lowered, _, _>::with_stage(
            "right",
            FreshnessScopedBasis::<CurrentValidity, _>::new(AssumptionBasis::new(5_u16)),
        ));

        let joined = join_ready_recipe_pair(JoinInputs2::new(left, right));

        assert_eq!(joined.payload().left(), &"left");
        assert_eq!(joined.payload().right(), &"right");
        assert_eq!(joined.basis().left().basis().value(), &3_u8);
        assert_eq!(joined.basis().right().basis().value(), &5_u16);
    }

    const _: () = {
        type LeftBasis = FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>;
        type RightBasis = FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u16>>;

        assert!(
            size_of::<
                ExecutionReadyRecipe<JoinInputs2<u64, u16>, JoinInputs2<LeftBasis, RightBasis>>,
            >() == size_of::<
                Recipe<Lowered, JoinInputs2<u64, u16>, JoinInputs2<LeftBasis, RightBasis>>,
            >()
        );
    };

    #[test]
    fn ready_recipe_join_composition_short_circuits_and_preserves_success_shape() {
        type LeftBasis = FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>;

        let left = ExecutionReadyRecipe::new(Recipe::<Lowered, _, _>::with_stage(
            "left",
            FreshnessScopedBasis::<CurrentValidity, _>::new(AssumptionBasis::new(3_u8)),
        ));
        let right = ExecutionReadyRecipe::new(Recipe::<Lowered, _, _>::with_stage(
            "right",
            FreshnessScopedBasis::<CurrentValidity, _>::new(AssumptionBasis::new(5_u16)),
        ));

        let joined = compose_join_ready_recipe_pair(
            TransitionOutcome::<_, &'static str>::success(left),
            || TransitionOutcome::success(right),
        );

        let joined = match joined {
            TransitionOutcome::Success(joined) => joined,
            _ => panic!("expected success"),
        };

        assert_eq!(joined.payload().left(), &"left");
        assert_eq!(joined.payload().right(), &"right");

        let denied = compose_join_ready_recipe_pair(
            TransitionOutcome::<ExecutionReadyRecipe<&'static str, LeftBasis>, &'static str>::denied(
                "denied",
            ),
            || -> TransitionOutcome<ExecutionReadyRecipe<&'static str, LeftBasis>, &'static str> {
                panic!("right lane must not run after left denial")
            },
        );

        assert!(matches!(denied, TransitionOutcome::Denied("denied")));
    }
}
