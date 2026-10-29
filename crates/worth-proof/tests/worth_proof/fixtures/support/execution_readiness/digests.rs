use std::any::type_name;

use worth_proof::{
    AdmitExecutionReadyRecipeTransition, AssumptionBasis, BoundaryBridgedStaleReadableBasis,
    CheckedAdmitExecutionReadyRecipeTransition, CheckedReadmitLoweredForExecutionReadyTransition,
    CurrentValidity, ExecutedRecipe, ExecutionReadinessContext, ExecutionReadyAdmissionReadiness,
    ExecutionReadyRecipe, FreshnessScopedBasis, Lowered, LoweredReadmissionContext,
    LoweredReadmissionReadiness, ReadmitLoweredForExecutionReadyTransition, RebindRequiredBasis,
    Recipe, Resolved, StaleReadableBasis,
};

use super::super::proof_shapes::{BasisDigest, FailureDigest, TransitionDigest};
use super::representatives::RepresentativeReadinessAuthority;

pub fn basis_digest() -> BasisDigest {
    BasisDigest::new(
        "lowering_and_execution_readiness_boundary",
        vec![
            type_name::<FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>(),
            type_name::<FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u16>>>(),
            type_name::<
                ExecutionReadyRecipe<
                    u64,
                    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
                >,
            >(),
            type_name::<
                ExecutionReadyRecipe<
                    u64,
                    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u16>>,
                >,
            >(),
            type_name::<
                ExecutedRecipe<u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>,
            >(),
            type_name::<
                ExecutedRecipe<u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u16>>>,
            >(),
        ],
    )
}

pub fn failure_digest() -> FailureDigest {
    FailureDigest::new(
        "lowering_and_execution_readiness_boundary",
        vec![
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
        ],
    )
}

pub fn transition_digest() -> TransitionDigest {
    TransitionDigest::new(
        "lowering_and_execution_readiness_boundary",
        vec![
            type_name::<
                Recipe<Lowered, u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>,
            >(),
            type_name::<Recipe<Lowered, u64, BoundaryBridgedStaleReadableBasis<u8>>>(),
            type_name::<
                Recipe<Lowered, u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u16>>>,
            >(),
            type_name::<ExecutionReadinessContext<&'static str, RepresentativeReadinessAuthority>>(
            ),
            type_name::<
                LoweredReadmissionContext<
                    u16,
                    RepresentativeReadinessAuthority,
                    &'static str,
                    RepresentativeReadinessAuthority,
                >,
            >(),
            type_name::<AdmitExecutionReadyRecipeTransition>(),
            type_name::<CheckedAdmitExecutionReadyRecipeTransition>(),
            type_name::<ReadmitLoweredForExecutionReadyTransition>(),
            type_name::<CheckedReadmitLoweredForExecutionReadyTransition>(),
            type_name::<
                ExecutionReadyRecipe<
                    u64,
                    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
                >,
            >(),
            type_name::<
                ExecutedRecipe<u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>,
            >(),
            type_name::<worth_proof::ExecuteReadyRecipeTransition>(),
            "checked_admit_ready_and_execute_recipe",
            "checked_readmit_ready_and_execute_recipe",
            "readmit_ready_and_execute_recipe",
            type_name::<
                ExecutionReadyAdmissionReadiness<
                    u64,
                    u8,
                    &'static str,
                    RepresentativeReadinessAuthority,
                    &'static str,
                    &'static str,
                    &'static str,
                >,
            >(),
            type_name::<
                LoweredReadmissionReadiness<
                    u64,
                    u8,
                    u16,
                    RepresentativeReadinessAuthority,
                    &'static str,
                    RepresentativeReadinessAuthority,
                    &'static str,
                    &'static str,
                    &'static str,
                >,
            >(),
            type_name::<Recipe<Lowered, u64, StaleReadableBasis<u8>>>(),
            type_name::<Recipe<Resolved, u64, RebindRequiredBasis<u8>>>(),
        ],
    )
}
