use worth_proof::{Lowered, Recipe, Resolved};

fn resolved_declaration() -> Recipe<Resolved, &'static str> {
    todo!("compile-fail fixture never executes")
}

fn requires_lowered_plan(_: Recipe<Lowered, &'static str>) {}

fn main() {
    requires_lowered_plan(resolved_declaration());
}
