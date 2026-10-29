use worth_proof::{ExecutionReadyRecipe, Lowered, NoAssumptionBasis, Recipe};

fn lowered_plan() -> Recipe<Lowered, &'static str> {
    todo!("compile-fail fixture never executes")
}

fn requires_execution_ready_plan(_: ExecutionReadyRecipe<&'static str, NoAssumptionBasis>) {}

fn main() {
    requires_execution_ready_plan(lowered_plan());
}
