use worth_proof::{
    AdmitRecipeTransition, Admitted, AuthorityMarker, AuthorityWitness, CapabilityMarker,
    CapabilityWitness, CurrentValidity, FreshnessScopedBasis, LowerRecipeTransition,
    PreConstructionGate, Recipe, RecipeAdmissionReadiness, RecipeLoweringReadiness,
    RecipeResolutionContext, RecipeResolutionGate, TransitionOutcome, Unresolved,
    resolve_checked_lower_and_admit_recipe, resolve_lower_and_admit_recipe,
};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct AdmissionAuthority;
impl AuthorityMarker for AdmissionAuthority {}

fn equivalent_admitted_checked_progression_compiles(
    direct_resolution_authority: AuthorityWitness<ResolutionAuthority>,
    checked_resolution_authority: AuthorityWitness<ResolutionAuthority>,
    direct_lowering_capability: CapabilityWitness<LoweringCapability>,
    checked_lowering_capability: CapabilityWitness<LoweringCapability>,
    direct_admission_authority: AuthorityWitness<AdmissionAuthority>,
    checked_admission_authority: AuthorityWitness<AdmissionAuthority>,
) {
    let direct_gate: RecipeResolutionGate<u8, ResolutionAuthority, &'static str, &'static str> =
        PreConstructionGate::ready(RecipeResolutionContext::new(
            7_u8,
            direct_resolution_authority,
        ));
    let checked_gate: RecipeResolutionGate<u8, ResolutionAuthority, &'static str, &'static str> =
        PreConstructionGate::ready(RecipeResolutionContext::new(
            7_u8,
            checked_resolution_authority,
        ));
    let direct = resolve_lower_and_admit_recipe(
        Recipe::<Unresolved, _>::new("payload"),
        direct_gate,
        &LowerRecipeTransition::new(direct_lowering_capability),
        &AdmitRecipeTransition::new(direct_admission_authority),
    );
    let checked = resolve_checked_lower_and_admit_recipe(
        Recipe::<Unresolved, _>::new("payload"),
        checked_gate,
        RecipeLoweringReadiness::<
            &str,
            u8,
            LoweringCapability,
            &'static str,
            &'static str,
            &'static str,
        >::ready(checked_lowering_capability),
        RecipeAdmissionReadiness::<
            &str,
            u8,
            AdmissionAuthority,
            &'static str,
            &'static str,
            &'static str,
        >::ready(checked_admission_authority),
    );

    let _direct_admitted: Recipe<
        Admitted,
        &str,
        FreshnessScopedBasis<CurrentValidity, worth_proof::AssumptionBasis<u8>>,
    > = match direct {
        TransitionOutcome::Success(admitted) => admitted,
        _ => panic!("direct all-ready lane should succeed"),
    };
    let _checked_admitted: Recipe<
        Admitted,
        &str,
        FreshnessScopedBasis<CurrentValidity, worth_proof::AssumptionBasis<u8>>,
    > = match checked {
        TransitionOutcome::Success(admitted) => admitted,
        _ => panic!("checked all-ready lane should succeed"),
    };
}

fn main() {}
