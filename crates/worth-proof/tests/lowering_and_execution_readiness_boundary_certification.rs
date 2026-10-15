#[allow(unused_imports)]
use crate::shared_support as support;

use support::compile_fail::run_compile_fail_bundle;
use support::compile_pass::run_compile_pass_bundle;
use support::execution_readiness;
use worth_proof::{
    AssumptionBasis, CurrentValidity, ExecutedRecipe, ExecutionReadyRecipe, FreshnessScopedBasis,
};

#[test]
fn lowering_and_execution_readiness_boundary_certification() {
    let compile_fail_bundle = execution_readiness::compile_fail_bundle();
    let compile_pass_bundle = execution_readiness::compile_pass_bundle();
    let transition_digest = execution_readiness::transition_digest();
    let basis_digest = execution_readiness::basis_digest();
    let failure_digest = execution_readiness::failure_digest();
    let codegen_honesty_report = execution_readiness::codegen_honesty_report();
    let residual_debt_report = execution_readiness::residual_debt_report();

    run_compile_fail_bundle(&compile_fail_bundle);
    run_compile_pass_bundle(&compile_pass_bundle);

    assert_eq!(
        compile_fail_bundle.suite(),
        "lowering_and_execution_readiness_boundary"
    );
    assert_eq!(
        compile_fail_bundle.families(),
        vec![
            "lowered_ready_boundary",
            "pre_lowered_readiness_boundary",
            "bridged_lowered_readiness_boundary",
            "shifted_basis_ready_boundary",
        ]
    );
    assert_eq!(
        compile_pass_bundle.suite(),
        "lowering_and_execution_readiness_boundary"
    );
    assert_eq!(
        compile_pass_bundle.families(),
        vec![
            "lowered_ready_executed_progression",
            "checked_readiness_progression",
            "same_basis_runtime_readmission_progression",
            "shifted_basis_runtime_readmission_progression",
        ]
    );
    assert_eq!(
        transition_digest.suite(),
        "lowering_and_execution_readiness_boundary"
    );
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry: &&str| entry.contains("AdmitExecutionReadyRecipeTransition")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry: &&str| entry.contains("ExecuteReadyRecipeTransition")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry: &&str| entry.contains("CheckedAdmitExecutionReadyRecipeTransition")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry: &&str| entry.contains("ReadmitLoweredForExecutionReadyTransition")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry: &&str| entry.contains("CheckedReadmitLoweredForExecutionReadyTransition")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry: &&str| entry.contains("checked_admit_ready_and_execute_recipe")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry: &&str| entry.contains("checked_readmit_ready_and_execute_recipe")));
    assert!(transition_digest.entries().iter().any(|entry: &&str| {
        entry.contains("TransitionReadiness")
            && entry.contains("ExecutionReadinessContext")
            && entry.contains("StaleReadable")
            && entry.contains("RebindRequired")
    }));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry: &&str| entry.contains("BoundaryBridged")));
    assert_eq!(
        basis_digest.suite(),
        "lowering_and_execution_readiness_boundary"
    );
    assert!(basis_digest
        .entries()
        .iter()
        .any(|entry: &&str| entry.contains("ExecutionReadyRecipe")));
    assert!(basis_digest
        .entries()
        .iter()
        .any(|entry: &&str| entry.contains("ExecutedRecipe")));
    assert!(basis_digest
        .entries()
        .iter()
        .any(|entry: &&str| entry.contains("AssumptionBasis<u16>")));
    assert_eq!(
        failure_digest.suite(),
        "lowering_and_execution_readiness_boundary"
    );
    assert_eq!(
        failure_digest.entries(),
        [
            "lowered_ready_boundary::tests/ui/milestone5/compile_fail/lowered_recipe_cannot_execute_without_readiness.rs",
            "pre_lowered_readiness_boundary::tests/ui/milestone5/compile_fail/resolved_recipe_cannot_enter_execution_readiness.rs",
            "bridged_lowered_readiness_boundary::tests/ui/milestone5/compile_fail/boundary_bridged_lowered_cannot_enter_execution_readiness.rs",
            "shifted_basis_ready_boundary::tests/ui/milestone5/compile_fail/shifted_basis_ready_recipe_cannot_be_treated_as_original_basis.rs",
            "category_divergence::denied",
            "category_divergence::deferred",
            "category_divergence::stale",
            "category_divergence::rebind_required",
            "category_divergence::failed",
            "ready_executed_divergence::ready_and_executed_are_distinct_wrappers",
            "shifted_basis_divergence::readmitted_lowered_basis_differs_from_original_basis",
            "equivalence_lane::direct_ready_and_same_basis_runtime_readmission_match",
        ]
    );
    assert_eq!(
        codegen_honesty_report.suite(),
        "lowering_and_execution_readiness_boundary"
    );
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
    assert_eq!(
        residual_debt_report.suite(),
        "lowering_and_execution_readiness_boundary"
    );
    assert_eq!(residual_debt_report.items().len(), 1);
    assert_eq!(
        residual_debt_report.items()[0].category(),
        "representative_scope"
    );

    assert_ne!(
        std::any::type_name::<
            ExecutionReadyRecipe<
                &'static str,
                FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
            >,
        >(),
        std::any::type_name::<
            ExecutedRecipe<
                &'static str,
                FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
            >,
        >()
    );
    assert_eq!(
        std::mem::size_of::<
            ExecutionReadyRecipe<u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>,
        >(),
        std::mem::size_of::<
            ExecutedRecipe<u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>,
        >()
    );
    assert_ne!(
        std::any::type_name::<
            ExecutionReadyRecipe<
                &'static str,
                FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
            >,
        >(),
        std::any::type_name::<
            ExecutionReadyRecipe<
                &'static str,
                FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u16>>,
            >,
        >()
    );
}
