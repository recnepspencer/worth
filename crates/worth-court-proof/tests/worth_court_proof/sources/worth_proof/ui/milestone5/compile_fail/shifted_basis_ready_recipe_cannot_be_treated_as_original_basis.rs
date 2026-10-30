use worth_proof::{
    AdmitExecutionReadyRecipeTransition, AssumptionBasis, AuthorityMarker, AuthorityWitness,
    CapabilityMarker, CapabilityWitness, ContextualTransition, CurrentValidity,
    ExecutionReadinessContext, ExecutionReadyRecipe, FreshnessScopedBasis,
    LowerRecipeTransition, Recipe, RecipeResolutionContext, ResolveRecipeTransition, Transition,
    Unresolved,
};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct ReadmissionAuthority;
impl AuthorityMarker for ReadmissionAuthority {}

struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}

fn shifted_basis_ready_recipe_cannot_be_treated_as_original_basis(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
    readmission_authority: AuthorityWitness<ReadmissionAuthority>,
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
    let readmitted = lowered
        .bridge_trust_boundary()
        .readmit_with_authority(13_u16, readmission_authority);
    let ready = AdmitExecutionReadyRecipeTransition
        .transition(
            readmitted,
            ExecutionReadinessContext::new("runtime admission", readiness_authority),
        )
        .into_value();

    let _ready: ExecutionReadyRecipe<
        _,
        FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
    > = ready;
}

fn main() {}
