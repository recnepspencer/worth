use worth_proof::prelude::*;
use worth_proof::{
    resolve_family_symbol, AuthoritativeFamilyMember, CompositionFamilySymbol, NonEmpty, Pair,
    Recipe, Unresolved,
};

#[test]
fn pleasant_entrypoint_helpers_construct_the_same_semantic_shapes_as_raw_constructors() {
    let pleasant_recipe = recipe("payload");
    let raw_recipe = Recipe::<Unresolved, _>::new("payload");
    let pleasant_pair = pair("left", "right");
    let raw_pair = Pair::new("left", "right");
    let pleasant_non_empty = non_empty("head", vec!["tail"]);
    let raw_non_empty = NonEmpty::new("head", vec!["tail"]);
    let pleasant_symbol = sym(7_u8);
    let raw_symbol = CompositionFamilySymbol::new(7_u8);
    let pleasant_member = member(11_u16);
    let raw_member = AuthoritativeFamilyMember::new(11_u16);

    assert_eq!(pleasant_recipe, raw_recipe);
    assert_eq!(pleasant_pair, raw_pair);
    assert_eq!(pleasant_non_empty, raw_non_empty);
    assert_eq!(pleasant_symbol, raw_symbol);
    assert_eq!(pleasant_member, raw_member);
}

#[test]
fn prelude_can_be_used_alongside_raw_lane_without_competing_type_story() {
    let pleasant_recipe = recipe("payload");
    let raw_recipe = Recipe::<Unresolved, _>::new("payload");
    let resolved = resolve_family_symbol(sym(2_u8), member(5_u16));

    assert_eq!(pleasant_recipe.payload(), raw_recipe.payload());
    assert_eq!(resolved.symbol().value(), &2_u8);
    assert_eq!(resolved.authoritative().value(), &5_u16);
}
