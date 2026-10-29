use worth_proof::{
    DeferredTransitionOutcome, DenialTransitionOutcome, FreshnessTransitionOutcome, Lowered,
    RebindRequiredBasis, Recipe, Resolved, StaleReadableBasis, TransitionOutcome,
};

fn typed_transition_outcomes_preserve_non_success_categories(
    stale_recipe: Recipe<Lowered, &'static str, StaleReadableBasis<u8>>,
    rebind_recipe: Recipe<Resolved, &'static str, RebindRequiredBasis<u8>>,
) {
    let denied: DenialTransitionOutcome<u64, &'static str> = TransitionOutcome::denied("denied");
    let deferred: DeferredTransitionOutcome<u64, &'static str, &'static str> =
        TransitionOutcome::deferred("deferred");
    let stale: FreshnessTransitionOutcome<
        u64,
        Recipe<Lowered, &'static str, StaleReadableBasis<u8>>,
        Recipe<Resolved, &'static str, RebindRequiredBasis<u8>>,
        &'static str,
    > = TransitionOutcome::stale(stale_recipe);
    let rebind: FreshnessTransitionOutcome<
        u64,
        Recipe<Lowered, &'static str, StaleReadableBasis<u8>>,
        Recipe<Resolved, &'static str, RebindRequiredBasis<u8>>,
        &'static str,
    > = TransitionOutcome::rebind_required(rebind_recipe);

    let _ = (
        matches!(denied, TransitionOutcome::Denied("denied")),
        matches!(deferred, TransitionOutcome::Deferred("deferred")),
        matches!(stale, TransitionOutcome::Stale(_)),
        matches!(rebind, TransitionOutcome::RebindRequired(_)),
    );
}

fn main() {}
