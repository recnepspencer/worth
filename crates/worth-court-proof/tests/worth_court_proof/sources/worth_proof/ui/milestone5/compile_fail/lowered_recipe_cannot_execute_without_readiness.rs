use worth_proof::{
    AuthorityMarker, AuthorityWitness, CapabilityMarker, CapabilityWitness,
    ContextualTransition,
    ExecuteReadyRecipeTransition, LowerRecipeTransition, Recipe, RecipeResolutionContext,
    ResolveRecipeTransition, Transition, Unresolved,
};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

fn lowered_recipe_cannot_execute_without_readiness(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
) {
    let unresolved = Recipe::<Unresolved, _>::new("payload");
    let resolved = ResolveRecipeTransition.transition(
        unresolved,
        RecipeResolutionContext::new(8_u8, resolution_authority),
    );
    let lowered = LowerRecipeTransition::new(lowering_capability)
    .transition(resolved.into_value())
    .into_value();

    let _ = ExecuteReadyRecipeTransition.transition(lowered);
}

fn main() {}
