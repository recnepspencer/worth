use worth_proof::{
    AdmitExecutionReadyRecipeTransition, AuthorityMarker, AuthorityWitness, CapabilityMarker,
    CapabilityWitness, ContextualTransition, ExecutionReadinessContext, LowerRecipeTransition,
    Recipe, RecipeResolutionContext, ResolveRecipeTransition, Transition, Unresolved,
};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}

fn boundary_bridged_lowered_cannot_enter_execution_readiness(
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
    let bridged = lowered.bridge_trust_boundary();

    let _ = AdmitExecutionReadyRecipeTransition.transition(
        bridged,
        ExecutionReadinessContext::new("runtime admission", readiness_authority),
    );
}

fn main() {}
