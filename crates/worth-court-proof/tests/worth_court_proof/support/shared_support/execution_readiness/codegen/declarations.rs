pub(crate) use {std::mem::align_of, std::mem::needs_drop};

pub(crate) use {
    worth_proof::AssumptionBasis, worth_proof::BoundaryBridgedStaleReadableBasis,
    worth_proof::CurrentValidity, worth_proof::ExecutedRecipe,
    worth_proof::ExecutionReadinessContext, worth_proof::ExecutionReadyAdmissionReadiness,
    worth_proof::ExecutionReadyRecipe, worth_proof::FreshnessScopedBasis, worth_proof::Lowered,
    worth_proof::LoweredReadmissionContext, worth_proof::LoweredReadmissionReadiness,
    worth_proof::RebindRequiredBasis, worth_proof::Recipe, worth_proof::StaleReadableBasis,
};

pub(crate) use crate::shared_support::execution_readiness::representatives::RepresentativeReadinessAuthority;
pub(crate) use {
    crate::shared_support::type_shapes::codegen::CodegenHonestyReport,
    crate::shared_support::type_shapes::codegen::CodegenShapeCheck,
};

pub(crate) fn codegen_honesty_report() -> CodegenHonestyReport {
    crate::shared_support::type_shapes::codegen::CodegenHonestyReport::size_layout_and_drop_certified(
        "lowering_and_execution_readiness_boundary",
        vec![
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "admit_execution_ready_transition",
                align_of::<worth_proof::AdmitExecutionReadyRecipeTransition>(),
                align_of::<()>(),
                needs_drop::<worth_proof::AdmitExecutionReadyRecipeTransition>(),
                needs_drop::<()>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "checked_admit_execution_ready_transition",
                align_of::<worth_proof::CheckedAdmitExecutionReadyRecipeTransition>(),
                align_of::<()>(),
                needs_drop::<worth_proof::CheckedAdmitExecutionReadyRecipeTransition>(),
                needs_drop::<()>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "execute_ready_transition",
                align_of::<worth_proof::ExecuteReadyRecipeTransition>(),
                align_of::<()>(),
                needs_drop::<worth_proof::ExecuteReadyRecipeTransition>(),
                needs_drop::<()>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "readmit_lowered_transition",
                align_of::<worth_proof::ReadmitLoweredForExecutionReadyTransition>(),
                align_of::<()>(),
                needs_drop::<worth_proof::ReadmitLoweredForExecutionReadyTransition>(),
                needs_drop::<()>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "checked_readmit_lowered_transition",
                align_of::<worth_proof::CheckedReadmitLoweredForExecutionReadyTransition>(),
                align_of::<()>(),
                needs_drop::<worth_proof::CheckedReadmitLoweredForExecutionReadyTransition>(),
                needs_drop::<()>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "execution_readiness_context",
                align_of::<ExecutionReadinessContext<&'static str, RepresentativeReadinessAuthority>>(),
                align_of::<&'static str>(),
                needs_drop::<ExecutionReadinessContext<
                    &'static str,
                    RepresentativeReadinessAuthority,
                >>(),
                needs_drop::<&'static str>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "lowered_readmission_context",
                align_of::<LoweredReadmissionContext<
                    u16,
                    RepresentativeReadinessAuthority,
                    &'static str,
                    RepresentativeReadinessAuthority,
                >>(),
                align_of::<ExecutionReadinessContext<
                    &'static str,
                    RepresentativeReadinessAuthority,
                >>(),
                needs_drop::<LoweredReadmissionContext<
                    u16,
                    RepresentativeReadinessAuthority,
                    &'static str,
                    RepresentativeReadinessAuthority,
                >>(),
                needs_drop::<ExecutionReadinessContext<
                    &'static str,
                    RepresentativeReadinessAuthority,
                >>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "execution_ready_wrapper",
                align_of::<
                    ExecutionReadyRecipe<
                        u64,
                        FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
                    >,
                >(),
                align_of::<
                    Recipe<Lowered, u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>,
                >(),
                needs_drop::<
                    ExecutionReadyRecipe<
                        u64,
                        FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
                    >,
                >(),
                needs_drop::<
                    Recipe<Lowered, u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>,
                >(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "executed_wrapper",
                align_of::<
                    ExecutedRecipe<u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>,
                >(),
                align_of::<
                    ExecutionReadyRecipe<
                        u64,
                        FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
                    >,
                >(),
                needs_drop::<
                    ExecutedRecipe<u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>,
                >(),
                needs_drop::<
                    ExecutionReadyRecipe<
                        u64,
                        FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
                    >,
                >(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "checked_execution_readiness",
                align_of::<
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
                align_of::<Recipe<Lowered, u64, StaleReadableBasis<u8>>>(),
                needs_drop::<
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
                needs_drop::<Recipe<Lowered, u64, StaleReadableBasis<u8>>>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "checked_lowered_readmission",
                align_of::<
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
                align_of::<Recipe<Lowered, u64, BoundaryBridgedStaleReadableBasis<u8>>>(),
                needs_drop::<
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
                needs_drop::<Recipe<Lowered, u64, BoundaryBridgedStaleReadableBasis<u8>>>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "rebind_sensitive_readiness_payload",
                align_of::<Recipe<worth_proof::Resolved, u64, RebindRequiredBasis<u8>>>(),
                align_of::<u64>(),
                needs_drop::<Recipe<worth_proof::Resolved, u64, RebindRequiredBasis<u8>>>(),
                needs_drop::<u64>(),
            ),
        ],
        "Milestone 5 certifies representative size/layout/drop honesty for lowered, ready, executed, readiness-context, and lowered-readmission carriers; it does not yet ship cross-crate executor baseline comparisons or exhaustive executed-state families.",
    )
}
