use worth_proof::{
    apply_contextual_transition, CheckedLowerRecipeTransition, CurrentValidity,
    FreshnessScopedBasis, Lowered, Recipe, TransitionReadiness,
};

struct LoweringCapability;
impl worth_proof::CapabilityMarker for LoweringCapability {}

fn invalid_checked_lowering_pipeline(
    lowered: Recipe<Lowered, &str, FreshnessScopedBasis<CurrentValidity, worth_proof::AssumptionBasis<u8>>>,
) {
    let readiness: worth_proof::RecipeLoweringReadiness<
        &str,
        u8,
        LoweringCapability,
        &'static str,
        &'static str,
        &'static str,
    > = TransitionReadiness::failed("failed");
    let _ = apply_contextual_transition(
        &CheckedLowerRecipeTransition::<LoweringCapability>::new(),
        lowered,
        readiness,
    );
}

fn main() {}
