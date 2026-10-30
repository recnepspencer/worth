use worth_proof::{Recipe, Resolved, Unresolved};

fn requires_resolved_recipe(_: Recipe<Resolved, &'static str>) {}

fn main() {
    let raw_declaration = Recipe::<Unresolved, _>::new("raw placement declaration");

    requires_resolved_recipe(raw_declaration);
}
