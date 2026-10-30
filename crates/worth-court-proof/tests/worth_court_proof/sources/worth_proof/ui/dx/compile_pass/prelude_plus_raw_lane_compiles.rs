use worth_proof::prelude::*;
use worth_proof::raw::{resolve_family_symbol, Recipe, Unresolved};

fn main() {
    let pleasant_recipe = recipe("payload");
    let raw_recipe = Recipe::<Unresolved, _>::new("payload");
    let pleasant_pair = pair("left", "right");
    let pleasant_non_empty = non_empty("head", vec!["tail"]);
    let resolved = resolve_family_symbol(sym(7_u8), member(11_u16));

    let _ = pleasant_recipe.payload();
    let _ = raw_recipe.payload();
    let _ = pleasant_pair.left();
    let _ = pleasant_non_empty.first();
    let _ = resolved.authoritative().value();
}
