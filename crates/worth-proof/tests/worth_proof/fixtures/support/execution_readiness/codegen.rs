use std::mem::{align_of, needs_drop};

use worth_proof::{
    AssumptionBasis, BoundaryBridgedStaleReadableBasis, CurrentValidity, ExecutedRecipe,
    ExecutionReadinessContext, ExecutionReadyAdmissionReadiness, ExecutionReadyRecipe,
    FreshnessScopedBasis, Lowered, LoweredReadmissionContext, LoweredReadmissionReadiness,
    RebindRequiredBasis, Recipe, StaleReadableBasis,
};

use super::super::type_shapes::{CodegenHonestyReport, CodegenShapeCheck};
use super::representatives::RepresentativeReadinessAuthority;

pub fn codegen_honesty_report() -> CodegenHonestyReport {
    CodegenHonestyReport::size_layout_and_drop_certified(
        "lowering_and_execution_readiness_boundary",
        vec![
            CodegenShapeCheck::new(
                "admit_execution_ready_transition",
                align_of::<worth_proof::AdmitExecutionReadyRecipeTransition>(),
                align_of::<()>(),
                needs_drop::<worth_proof::AdmitExecutionReadyRecipeTransition>(),
                needs_drop::<()>(),
            ),
            CodegenShapeCheck::new(
                "checked_admit_execution_ready_transition",
                align_of::<worth_proof::CheckedAdmitExecutionReadyRecipeTransition>(),
                align_of::<()>(),
                needs_drop::<worth_proof::CheckedAdmitExecutionReadyRecipeTransition>(),
                needs_drop::<()>(),
            ),
            CodegenShapeCheck::new(
                "execute_ready_transition",
                align_of::<worth_proof::ExecuteReadyRecipeTransition>(),
                align_of::<()>(),
                needs_drop::<worth_proof::ExecuteReadyRecipeTransition>(),
                needs_drop::<()>(),
            ),
            CodegenShapeCheck::new(
                "readmit_lowered_transition",
                align_of::<worth_proof::ReadmitLoweredForExecutionReadyTransition>(),
                align_of::<()>(),
                needs_drop::<worth_proof::ReadmitLoweredForExecutionReadyTransition>(),
                needs_drop::<()>(),
            ),
            CodegenShapeCheck::new(
                "checked_readmit_lowered_transition",
                align_of::<worth_proof::CheckedReadmitLoweredForExecutionReadyTransition>(),
                align_of::<()>(),
                needs_drop::<worth_proof::CheckedReadmitLoweredForExecutionReadyTransition>(),
                needs_drop::<()>(),
            ),
            CodegenShapeCheck::new(
                "execution_readiness_context",
                align_of::<ExecutionReadinessContext<&'static str, RepresentativeReadinessAuthority>>(),
                align_of::<&'static str>(),
                needs_drop::<ExecutionReadinessContext<
                    &'static str,
                    RepresentativeReadinessAuthority,
                >>(),
                needs_drop::<&'static str>(),
            ),
            CodegenShapeCheck::new(
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
            CodegenShapeCheck::new(
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
            CodegenShapeCheck::new(
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
            CodegenShapeCheck::new(
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
            CodegenShapeCheck::new(
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
            CodegenShapeCheck::new(
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
