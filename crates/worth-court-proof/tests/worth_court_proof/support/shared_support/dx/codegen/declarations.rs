pub(crate) use {core::mem::align_of, core::mem::needs_drop};

pub(crate) use {
    worth_proof::AuthorityWitness, worth_proof::CapabilityWitness, worth_proof::ExecutedRecipe,
    worth_proof::ExecutionReadyRecipe, worth_proof::Lowered, worth_proof::ProofFlow,
    worth_proof::ProofOutcome, worth_proof::Recipe, worth_proof::TransitionOutcome,
};

pub(crate) use {
    crate::shared_support::type_shapes::codegen::CodegenHonestyReport,
    crate::shared_support::type_shapes::codegen::CodegenShapeCheck,
};

pub(crate) use {
    crate::shared_support::dx::representatives::CurrentBasis,
    crate::shared_support::dx::representatives::LoweringCapability,
    crate::shared_support::dx::representatives::ReadinessAuthority,
    crate::shared_support::dx::representatives::ResolutionAuthority,
};

pub(crate) type PleasantExecuted = ExecutedRecipe<&'static str, CurrentBasis<u8>>;
pub(crate) type RawExecuted = ExecutedRecipe<&'static str, CurrentBasis<u8>>;
pub(crate) type PleasantChecked =
    ProofOutcome<ExecutionReadyRecipe<&'static str, CurrentBasis<u8>>>;
pub(crate) type RawChecked =
    TransitionOutcome<ExecutionReadyRecipe<&'static str, CurrentBasis<u8>>>;
pub(crate) type PleasantFlow = ProofFlow<
    AuthorityWitness<ResolutionAuthority>,
    CapabilityWitness<LoweringCapability>,
    AuthorityWitness<ReadinessAuthority>,
>;
pub(crate) type RawFlowState = (
    AuthorityWitness<ResolutionAuthority>,
    CapabilityWitness<LoweringCapability>,
    AuthorityWitness<ReadinessAuthority>,
);
pub(crate) type PleasantLowered = Recipe<Lowered, &'static str, CurrentBasis<u8>>;
pub(crate) type RawLowered = Recipe<Lowered, &'static str, CurrentBasis<u8>>;

pub(crate) fn codegen_honesty_report() -> CodegenHonestyReport {
    crate::shared_support::type_shapes::codegen::CodegenHonestyReport::size_layout_and_drop_certified(
        "pleasant_lane_hot_path_honesty",
        vec![
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "pleasant_executed_recipe_result",
                align_of::<PleasantExecuted>(),
                align_of::<RawExecuted>(),
                needs_drop::<PleasantExecuted>(),
                needs_drop::<RawExecuted>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "pleasant_checked_outcome_wrapper",
                align_of::<PleasantChecked>(),
                align_of::<RawChecked>(),
                needs_drop::<PleasantChecked>(),
                needs_drop::<RawChecked>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
                "pleasant_scoped_flow_carriage",
                align_of::<PleasantFlow>(),
                align_of::<RawFlowState>(),
                needs_drop::<PleasantFlow>(),
                needs_drop::<RawFlowState>(),
            ),
            crate::shared_support::type_shapes::codegen::CodegenShapeCheck::new(
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
