use worth_proof::{
    compose_join_ready_recipe_pair, ExecutionReadyRecipe, JoinInputs2, SuccessfulTransitionOutcome,
    TransitionOutcome,
};

fn checked_multi_input_ordering_and_ready_join_compiles<LA, LB, A, B>(
    left: ExecutionReadyRecipe<LA, A>,
    right: ExecutionReadyRecipe<LB, B>,
) {
    let joined: TransitionOutcome<
        ExecutionReadyRecipe<JoinInputs2<LA, LB>, JoinInputs2<A, B>>,
    > = compose_join_ready_recipe_pair(
        SuccessfulTransitionOutcome::new(left).into(),
        || SuccessfulTransitionOutcome::new(right).into(),
    );

    let _ = joined;
}

fn main() {}
