pub use {crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonFailure, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath};

pub fn compare_entities(
    expected: &ExpectedSupplyChainObservation,
    observed: &ObservedSupplyChainState,
) -> Result<(), ComparisonFailure> {
    for (key, expected_value) in &expected.entities {
        let Some(observed_value) = observed.entities.get(key) else {
            return Err(ComparisonFailure {
                mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::MissingEntity(*key),
            });
        };
        if expected_value != observed_value {
            return Err(ComparisonFailure {
                mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::EntityValue(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::entity(*key)),
            });
        }
    }
    if let Some(key) = observed
        .entities
        .keys()
        .find(|key| !expected.entities.contains_key(key))
    {
        return Err(ComparisonFailure {
            mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::UnexpectedEntity(*key),
        });
    }
    if expected.absent_entities != observed.absent_entities {
        let key = expected
            .absent_entities
            .symmetric_difference(&observed.absent_entities)
            .next()
            .copied()
            .unwrap_or(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, u32::MAX));
        return Err(ComparisonFailure {
            mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::EntityAbsence(key),
        });
    }
    Ok(())
}
