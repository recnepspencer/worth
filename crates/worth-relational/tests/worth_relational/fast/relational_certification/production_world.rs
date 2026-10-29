pub(crate) use super::world::supply_chain::{
    audit_supply_chain_baseline, compile_supply_chain_baseline, CompiledSupplyChainProgram,
    SupplyChainProgramError, SupplyChainScale, SupplyChainWorldDefinition,
};
pub(crate) use worth_relational::facade::history::BranchId;

pub(crate) fn empty_scale() -> SupplyChainScale {
    let mut scale = SupplyChainScale::court();
    scale.ports = 0;
    scale.terminals = 0;
    scale.berths = 0;
    scale.vessels = 0;
    scale.voyages = 0;
    scale.port_calls = 0;
    scale.cargo_lots = 0;
    scale.regions = 1;
    scale
}

#[test]
fn production_compiler_rejects_a_scaled_empty_definition_before_runtime_admission() {
    let error = CompiledSupplyChainProgram::compile(SupplyChainWorldDefinition::empty(
        SupplyChainScale::court(),
    ))
    .expect_err("a Court-sized empty definition is not an operating baseline");
    assert!(matches!(error, SupplyChainProgramError::Definition(_)));
}
