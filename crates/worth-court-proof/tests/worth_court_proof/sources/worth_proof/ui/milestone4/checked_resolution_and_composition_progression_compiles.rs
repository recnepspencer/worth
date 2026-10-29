use worth_proof::{
    resolve_lower_and_admit_recipe, AdmitRecipeTransition, AuthorityMarker, AuthorityWitness,
    CapabilityMarker, CapabilityWitness, LowerRecipeTransition, PreConstructionGate, Recipe,
    RecipeResolutionContext, TransitionOutcome, Unresolved,
};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct AdmissionAuthority;
impl AuthorityMarker for AdmissionAuthority {}

fn checked_resolution_and_composition_progression_compiles(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
    admission_authority: AuthorityWitness<AdmissionAuthority>,
) {
    let unresolved = Recipe::<Unresolved, _>::new("payload");
    let denied =
        PreConstructionGate::<RecipeResolutionContext<u8, ResolutionAuthority>, _, &'static str>::denied(
            "denied",
        );
    let deferred =
        PreConstructionGate::<RecipeResolutionContext<u8, ResolutionAuthority>, &'static str, _>::deferred(
            "deferred",
        );
    let ready: PreConstructionGate<
        RecipeResolutionContext<u8, ResolutionAuthority>,
        &'static str,
        &'static str,
    > = PreConstructionGate::ready(RecipeResolutionContext::new(7_u8, resolution_authority));
    let lower = LowerRecipeTransition::new(lowering_capability);
    let admit = AdmitRecipeTransition::new(admission_authority);

    let denied_outcome = resolve_lower_and_admit_recipe(
        Recipe::<Unresolved, _>::new("payload"),
        denied,
        &lower,
        &admit,
    );
    let deferred_outcome = resolve_lower_and_admit_recipe(
        Recipe::<Unresolved, _>::new("payload"),
        deferred,
        &lower,
        &admit,
    );
    let ready_outcome = resolve_lower_and_admit_recipe(unresolved, ready, &lower, &admit);

    let _ = (
        matches!(denied_outcome, TransitionOutcome::Denied("denied")),
        matches!(deferred_outcome, TransitionOutcome::Deferred("deferred")),
        matches!(ready_outcome, TransitionOutcome::Success(_)),
    );
}

fn main() {}
