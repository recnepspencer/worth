use worth_proof::{
    apply_transition, AdmitRecipeTransition, AuthorityMarker, AuthorityWitness, AssumptionBasis,
    CurrentValidity, FreshnessScopedBasis, Recipe, Resolved,
};

struct AdmissionAuthority;
impl AuthorityMarker for AdmissionAuthority {}

fn resolved_recipe_cannot_admit_through_transition_contract(
    resolved: Recipe<
        Resolved,
        &'static str,
        FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
    >,
    admission_authority: AuthorityWitness<AdmissionAuthority>,
) {
    let _ = apply_transition(
        &AdmitRecipeTransition::new(admission_authority),
        resolved,
    );
}

fn main() {}
