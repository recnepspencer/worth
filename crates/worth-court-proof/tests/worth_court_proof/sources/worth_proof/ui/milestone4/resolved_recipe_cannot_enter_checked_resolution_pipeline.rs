use worth_proof::{
    resolve_lower_and_admit_recipe, AdmitRecipeTransition, AuthorityMarker, AuthorityWitness,
    CapabilityMarker, CapabilityWitness, LowerRecipeTransition, PreConstructionGate, Recipe,
    RecipeResolutionContext, Resolved,
};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct AdmissionAuthority;
impl AuthorityMarker for AdmissionAuthority {}

fn resolved_recipe_cannot_enter_checked_resolution_pipeline(
    resolved: Recipe<Resolved, &'static str>,
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
    admission_authority: AuthorityWitness<AdmissionAuthority>,
) {
    let gate = PreConstructionGate::ready(RecipeResolutionContext::new(
        7_u8,
        resolution_authority,
    ));
    let lower = LowerRecipeTransition::new(lowering_capability);
    let admit = AdmitRecipeTransition::new(admission_authority);

    let _ = resolve_lower_and_admit_recipe(resolved, gate, &lower, &admit);
}

fn main() {}
