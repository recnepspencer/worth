use worth_proof::{
    AdmitExecutionReadyRecipeTransition, AuthorityMarker, AuthorityWitness, ContextualTransition,
    ExecutionReadinessContext, Recipe, RecipeResolutionContext, ResolveRecipeTransition, Unresolved,
};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}

fn resolved_recipe_cannot_enter_execution_readiness(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    readiness_authority: AuthorityWitness<ReadinessAuthority>,
) {
    let unresolved = Recipe::<Unresolved, _>::new("payload");
    let resolved = ResolveRecipeTransition.transition(
        unresolved,
        RecipeResolutionContext::new(8_u8, resolution_authority),
    );

    let _ = AdmitExecutionReadyRecipeTransition.transition(
        resolved.into_value(),
        ExecutionReadinessContext::new("runtime admission", readiness_authority),
    );
}

fn main() {}
