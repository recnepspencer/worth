#[allow(unused_imports)] use crate::relational_certification::ordinary_context::production_world::*;
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::baseline_audit::audit as audit_supply_chain_baseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::SupplyChainProgramError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition};
pub(crate) use {worth_relational::facade::history::BranchId};

#[test]
fn production_compiler_rejects_a_scaled_empty_definition_before_runtime_admission() {
    let error = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram::compile(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::empty(
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court(),
    ))
    .expect_err("a Court-sized empty definition is not an operating baseline");
    assert!(matches!(error, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::SupplyChainProgramError::Definition(_)));
}
