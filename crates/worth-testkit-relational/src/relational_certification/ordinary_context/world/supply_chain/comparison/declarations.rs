pub use {crate::relational_certification::ordinary_context::world::supply_chain::comparison_ancestry::compare_ancestry};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::comparison_entities::compare_entities};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::comparison_relations::compare_relations};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::comparison_schema::compare_schema};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation};

pub use {crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonFailure, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState};

/// Compare a complete observation by semantic axis.  Each axis owns its
/// mismatch construction; this function is only the ordered table of contents.
pub fn compare(
    expected: &ExpectedSupplyChainObservation,
    observed: &ObservedSupplyChainState,
) -> Result<(), ComparisonFailure> {
    compare_schema(expected, observed)?;
    compare_ancestry(expected, observed)?;
    compare_entities(expected, observed)?;
    compare_relations(expected, observed)?;
    Ok(())
}