use worth_proof::{
    apply_contextual_transition, apply_transition, AdmitRecipeTransition, AuthorityMarker,
    AuthorityWitness, CapabilityMarker, CapabilityWitness, LowerRecipeTransition, Recipe,
    RecipeResolutionContext, ResolveRecipeTransition, Unresolved,
};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct AdmissionAuthority;
impl AuthorityMarker for AdmissionAuthority {}

fn explicit_transition_contract_progression_compiles(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
    admission_authority: AuthorityWitness<AdmissionAuthority>,
) {
    let unresolved = Recipe::<Unresolved, _>::new("payload");
    let resolved = apply_contextual_transition(
        &ResolveRecipeTransition,
        unresolved,
        RecipeResolutionContext::new(7_u8, resolution_authority),
    );
    let resolved = resolved.into_value();

    let lowered = apply_transition(
        &LowerRecipeTransition::new(lowering_capability),
        resolved,
    );
    let lowered = lowered.into_value();

    let admitted = apply_transition(
        &AdmitRecipeTransition::new(admission_authority),
        lowered,
    );
    let admitted = admitted.into_value();

    let _ = admitted.strong_basis();
}

fn main() {}
