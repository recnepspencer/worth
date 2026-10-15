#[allow(unused_imports)]
use crate::shared_support as support;

use support::compile_fail::run_compile_fail_bundle;
use support::compile_pass::run_compile_pass_bundle;
use support::transition_algebra;
use worth_proof::{
    DeferredTransitionOutcome, DenialTransitionOutcome, FreshnessTransitionOutcome,
    PreConstructionGate, TransitionOutcome, TransitionReadiness,
};

#[test]
fn transition_outcome_algebra_certification() {
    let compile_fail_bundle = transition_algebra::compile_fail_bundle();
    let compile_pass_bundle = transition_algebra::compile_pass_bundle();
    let transition_digest = transition_algebra::transition_digest();
    let failure_digest = transition_algebra::failure_digest();
    let codegen_honesty_report = transition_algebra::codegen_honesty_report();
    let residual_debt_report = transition_algebra::residual_debt_report();

    run_compile_fail_bundle(&compile_fail_bundle);
    run_compile_pass_bundle(&compile_pass_bundle);

    assert_eq!(compile_fail_bundle.suite(), "transition_outcome_algebra");
    assert_eq!(compile_fail_bundle.families(), vec!["ordering_misuse"]);
    assert_eq!(compile_pass_bundle.suite(), "transition_outcome_algebra");
    assert_eq!(
        compile_pass_bundle.families(),
        vec![
            "control_progression",
            "typed_outcome_progression",
            "checked_composition_progression",
            "freshness_failure_progression",
            "equivalent_admitted_progression",
        ]
    );
    assert_eq!(transition_digest.suite(), "transition_outcome_algebra");
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("CheckedLowerRecipeTransition")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("CheckedAdmitRecipeTransition")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("TransitionReadiness")));
    assert_eq!(failure_digest.suite(), "transition_outcome_algebra");
    assert_eq!(
        failure_digest.entries(),
        [
            "ordering_misuse::tests/ui/milestone4/unresolved_recipe_cannot_lower_through_transition_contract.rs",
            "ordering_misuse::tests/ui/milestone4/resolved_recipe_cannot_admit_through_transition_contract.rs",
            "ordering_misuse::tests/ui/milestone4/resolved_recipe_cannot_enter_checked_resolution_pipeline.rs",
            "ordering_misuse::tests/ui/milestone4/lowered_recipe_cannot_enter_checked_lowering_pipeline.rs",
            "ordering_misuse::tests/ui/milestone4/resolved_recipe_cannot_enter_checked_admission_pipeline.rs",
            "category_divergence::denied",
            "category_divergence::deferred",
            "category_divergence::stale",
            "category_divergence::rebind_required",
            "category_divergence::failed",
            "equivalence_lane::direct_checked_all_ready",
        ]
    );
    assert_eq!(codegen_honesty_report.suite(), "transition_outcome_algebra");
    assert_eq!(residual_debt_report.suite(), "transition_outcome_algebra");
    assert_eq!(
        codegen_honesty_report.verified_scope(),
        "size_layout_and_drop_only"
    );
    assert!(codegen_honesty_report
        .checks()
        .iter()
        .all(|check| check.matches()));
    assert!(!codegen_honesty_report.hidden_dynamic_lookup());
    assert!(!codegen_honesty_report.hidden_virtual_dispatch());
    assert!(!codegen_honesty_report.mandatory_allocation_introduced());
    assert_eq!(residual_debt_report.items().len(), 1);
    assert_eq!(
        residual_debt_report.items()[0].category(),
        "representative_scope"
    );

    let denied: DenialTransitionOutcome<u64, &'static str> = TransitionOutcome::denied("denied");
    let deferred: DeferredTransitionOutcome<u64, &'static str, &'static str> =
        TransitionOutcome::deferred("deferred");
    let stale: FreshnessTransitionOutcome<u64, &'static str, &'static str, &'static str> =
        TransitionOutcome::stale("stale");
    let rebind: FreshnessTransitionOutcome<u64, &'static str, &'static str, &'static str> =
        TransitionOutcome::rebind_required("rebind");
    let failed: TransitionOutcome<
        u64,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    > = TransitionOutcome::failed("failed");
    let denied_gate = PreConstructionGate::<u64, _, &'static str>::denied("denied");
    let deferred_gate = PreConstructionGate::<u64, &'static str, _>::deferred("deferred");
    let readiness_stale = TransitionReadiness::<
        u64,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    >::stale("stale");
    let readiness_rebind = TransitionReadiness::<
        u64,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    >::rebind_required("rebind");
    let readiness_failed = TransitionReadiness::<
        u64,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    >::failed("failed");

    assert!(matches!(denied, TransitionOutcome::Denied("denied")));
    assert!(matches!(deferred, TransitionOutcome::Deferred("deferred")));
    assert!(matches!(stale, TransitionOutcome::Stale(_)));
    assert!(matches!(rebind, TransitionOutcome::RebindRequired(_)));
    assert!(matches!(failed, TransitionOutcome::Failed("failed")));
    assert!(matches!(denied_gate, PreConstructionGate::Denied("denied")));
    assert!(matches!(
        deferred_gate,
        PreConstructionGate::Deferred("deferred")
    ));
    assert!(matches!(
        readiness_stale,
        TransitionReadiness::Stale("stale")
    ));
    assert!(matches!(
        readiness_rebind,
        TransitionReadiness::RebindRequired("rebind")
    ));
    assert!(matches!(
        readiness_failed,
        TransitionReadiness::Failed("failed")
    ));
}
