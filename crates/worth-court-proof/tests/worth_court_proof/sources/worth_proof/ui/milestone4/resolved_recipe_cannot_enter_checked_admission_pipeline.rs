use worth_proof::{
    apply_contextual_transition, CheckedAdmitRecipeTransition, CurrentValidity,
    FreshnessScopedBasis, Recipe, Resolved, TransitionReadiness,
};

struct AdmissionAuthority;
impl worth_proof::AuthorityMarker for AdmissionAuthority {}

fn invalid_checked_admission_pipeline(
    resolved: Recipe<Resolved, &str, FreshnessScopedBasis<CurrentValidity, worth_proof::AssumptionBasis<u8>>>,
) {
    let readiness: worth_proof::RecipeAdmissionReadiness<
        &str,
        u8,
        AdmissionAuthority,
        &'static str,
        &'static str,
        &'static str,
    > = TransitionReadiness::failed("failed");
    let _ = apply_contextual_transition(
        &CheckedAdmitRecipeTransition::<AdmissionAuthority>::new(),
        resolved,
        readiness,
    );
}

fn main() {}
