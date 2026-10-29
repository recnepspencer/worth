use worth_proof::{
    apply_transition, CapabilityMarker, CapabilityWitness, LowerRecipeTransition, Recipe,
    Unresolved,
};

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

fn unresolved_recipe_cannot_lower_through_transition_contract(
    lowering_capability: CapabilityWitness<LoweringCapability>,
) {
    let unresolved = Recipe::<Unresolved, _>::new("payload");
    let _ = apply_transition(
        &LowerRecipeTransition::new(lowering_capability),
        unresolved,
    );
}

fn main() {}
