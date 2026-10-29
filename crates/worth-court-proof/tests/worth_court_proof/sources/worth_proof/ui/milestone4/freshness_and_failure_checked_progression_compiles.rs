use worth_proof::{
    resolve_checked_lower_and_admit_recipe, AuthorityMarker, AuthorityWitness, CapabilityMarker,
    PreConstructionGate, Recipe, RecipeAdmissionReadiness, RecipeLoweringReadiness,
    RecipeResolutionContext, TransitionOutcome, TransitionReadiness, Unresolved,
};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct AdmissionAuthority;
impl AuthorityMarker for AdmissionAuthority {}

fn freshness_and_failure_checked_progression_compiles(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
) {
    let failed_lowering: RecipeLoweringReadiness<
        &str,
        u8,
        LoweringCapability,
        &'static str,
        &'static str,
        &'static str,
    > = TransitionReadiness::failed("failed");
    let failed = resolve_checked_lower_and_admit_recipe(
        Recipe::<Unresolved, _>::new("payload"),
        PreConstructionGate::ready(RecipeResolutionContext::new(
            7_u8,
            resolution_authority,
        )),
        failed_lowering,
        RecipeAdmissionReadiness::<
            &str,
            u8,
            AdmissionAuthority,
            &'static str,
            &'static str,
            &'static str,
        >::failed("unreached"),
    );

    let _ = matches!(failed, TransitionOutcome::Failed("failed"));
}

fn main() {}
