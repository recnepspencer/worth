use worth_proof::{
    AdmitExecutionReadyRecipeTransition, AuthorityMarker, AuthorityWitness, CapabilityMarker,
    CapabilityWitness, ContextualTransition, ExecuteReadyRecipeTransition,
    ExecutionReadinessContext, LowerRecipeTransition, Recipe, RecipeResolutionContext,
    ResolveRecipeTransition, Transition, Unresolved,
};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}

fn explicit_lowered_ready_executed_progression_compiles(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
    readiness_authority: AuthorityWitness<ReadinessAuthority>,
) {
    let unresolved = Recipe::<Unresolved, _>::new("payload");
    let resolved = ResolveRecipeTransition.transition(
        unresolved,
        RecipeResolutionContext::new(8_u8, resolution_authority),
    );
    let lowered = LowerRecipeTransition::new(lowering_capability)
    .transition(resolved.into_value())
    .into_value();
    let ready = AdmitExecutionReadyRecipeTransition.transition(
        lowered,
        ExecutionReadinessContext::new("runtime admission", readiness_authority),
    );
    let executed = ExecuteReadyRecipeTransition.transition(ready.into_value()).into_value();

    let _ = executed.payload();
    let _ = executed.strong_basis().value();
}

fn main() {}
