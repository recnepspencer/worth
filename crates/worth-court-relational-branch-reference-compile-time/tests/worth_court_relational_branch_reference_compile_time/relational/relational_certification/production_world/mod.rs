#[allow(unused_imports)]
use crate::relational_certification::ordinary_context::production_world::*;

#[test]
fn production_compiler_rejects_a_scaled_empty_definition_before_runtime_admission() {
    let error = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram::compile(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::empty(
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court(),
    ))
    .expect_err("a Court-sized empty definition is not an operating baseline");
    assert!(matches!(error, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::SupplyChainProgramError::Definition(_)));
}
