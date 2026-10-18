use core::mem::{align_of, needs_drop};

use worth_proof::{
    AuthorityWitness, CapabilityWitness, ExecutedRecipe, ExecutionReadyRecipe, Lowered, ProofFlow,
    ProofOutcome, Recipe, TransitionOutcome,
};

use crate::shared_support::type_shapes::{CodegenHonestyReport, CodegenShapeCheck};

use super::representatives::{
    CurrentBasis, LoweringCapability, ReadinessAuthority, ResolutionAuthority,
};

type PleasantExecuted = ExecutedRecipe<&'static str, CurrentBasis<u8>>;
type RawExecuted = ExecutedRecipe<&'static str, CurrentBasis<u8>>;
type PleasantChecked = ProofOutcome<ExecutionReadyRecipe<&'static str, CurrentBasis<u8>>>;
type RawChecked = TransitionOutcome<ExecutionReadyRecipe<&'static str, CurrentBasis<u8>>>;
type PleasantFlow = ProofFlow<
    AuthorityWitness<ResolutionAuthority>,
    CapabilityWitness<LoweringCapability>,
    AuthorityWitness<ReadinessAuthority>,
>;
type RawFlowState = (
    AuthorityWitness<ResolutionAuthority>,
    CapabilityWitness<LoweringCapability>,
    AuthorityWitness<ReadinessAuthority>,
);
type PleasantLowered = Recipe<Lowered, &'static str, CurrentBasis<u8>>;
type RawLowered = Recipe<Lowered, &'static str, CurrentBasis<u8>>;

pub fn codegen_honesty_report() -> CodegenHonestyReport {
    CodegenHonestyReport::size_layout_and_drop_certified(
        "pleasant_lane_hot_path_honesty",
        vec![
            CodegenShapeCheck::new(
                "pleasant_executed_recipe_result",
                align_of::<PleasantExecuted>(),
                align_of::<RawExecuted>(),
                needs_drop::<PleasantExecuted>(),
                needs_drop::<RawExecuted>(),
            ),
            CodegenShapeCheck::new(
                "pleasant_checked_outcome_wrapper",
                align_of::<PleasantChecked>(),
                align_of::<RawChecked>(),
                needs_drop::<PleasantChecked>(),
                needs_drop::<RawChecked>(),
            ),
            CodegenShapeCheck::new(
                "pleasant_scoped_flow_carriage",
                align_of::<PleasantFlow>(),
                align_of::<RawFlowState>(),
                needs_drop::<PleasantFlow>(),
                needs_drop::<RawFlowState>(),
            ),
            CodegenShapeCheck::new(
                "pleasant_lowered_recipe_lane",
                align_of::<PleasantLowered>(),
                align_of::<RawLowered>(),
                needs_drop::<PleasantLowered>(),
                needs_drop::<RawLowered>(),
            ),
        ],
        "No MIR or ASM diff yet; representative DX certification remains limited to alignment and drop parity plus compile-fail and compile-pass proof surfaces.",
    )
}
