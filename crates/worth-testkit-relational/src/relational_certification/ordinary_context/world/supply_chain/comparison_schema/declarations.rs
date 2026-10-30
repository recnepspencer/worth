pub use {crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonFailure, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation};

pub fn compare_schema(
    expected: &ExpectedSupplyChainObservation,
    observed: &ObservedSupplyChainState,
) -> Result<(), ComparisonFailure> {
    if expected.schema.version != observed.schema {
        return Err(ComparisonFailure {
            mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::SchemaMeaning {
                expected: expected.schema.version,
                observed: observed.schema,
            },
        });
    }
    Ok(())
}
