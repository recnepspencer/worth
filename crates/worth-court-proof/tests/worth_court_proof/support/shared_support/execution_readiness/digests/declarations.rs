pub(crate) use {std::any::type_name};

pub(crate) use {worth_proof::AdmitExecutionReadyRecipeTransition, worth_proof::AssumptionBasis, worth_proof::BoundaryBridgedStaleReadableBasis, worth_proof::CheckedAdmitExecutionReadyRecipeTransition, worth_proof::CheckedReadmitLoweredForExecutionReadyTransition, worth_proof::CurrentValidity, worth_proof::ExecutedRecipe, worth_proof::ExecutionReadinessContext, worth_proof::ExecutionReadyAdmissionReadiness, worth_proof::ExecutionReadyRecipe, worth_proof::FreshnessScopedBasis, worth_proof::Lowered, worth_proof::LoweredReadmissionContext, worth_proof::LoweredReadmissionReadiness, worth_proof::ReadmitLoweredForExecutionReadyTransition, worth_proof::RebindRequiredBasis, worth_proof::Recipe, worth_proof::Resolved, worth_proof::StaleReadableBasis};

pub(crate) use {crate::shared_support::proof_shapes::digest::BasisDigest, crate::shared_support::proof_shapes::digest::FailureDigest, crate::shared_support::proof_shapes::digest::TransitionDigest};
pub(crate) use {crate::shared_support::execution_readiness::representatives::RepresentativeReadinessAuthority};

pub(crate) fn basis_digest() -> BasisDigest {
    crate::shared_support::proof_shapes::digest::BasisDigest::new(
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

pub(crate) fn failure_digest() -> FailureDigest {
    crate::shared_support::proof_shapes::digest::FailureDigest::new(
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

pub(crate) fn transition_digest() -> TransitionDigest {
    crate::shared_support::proof_shapes::digest::TransitionDigest::new(
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