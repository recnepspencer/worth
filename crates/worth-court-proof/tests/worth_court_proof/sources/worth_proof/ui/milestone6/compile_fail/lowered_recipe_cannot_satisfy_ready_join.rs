use worth_proof::{
    join_ready_recipe_pair, AssumptionBasis, CurrentValidity, FreshnessScopedBasis, JoinInputs2,
    Lowered, Recipe,
};

fn lowered_recipe_cannot_satisfy_ready_join(
    left: Recipe<Lowered, &'static str, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>,
    right: Recipe<Lowered, &'static str, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>,
) {
    let _joined = join_ready_recipe_pair(JoinInputs2::new(left, right));
}

fn main() {}
