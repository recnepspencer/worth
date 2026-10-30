pub use std::collections::BTreeSet;

pub use {crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonFailure, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::relation_comparison::compare_relation_vector_sets, crate::relational_certification::ordinary_context::world::supply_chain::relation_comparison::validate_relation_vector};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind};

pub fn compare_relations(
    expected: &ExpectedSupplyChainObservation,
    observed: &ObservedSupplyChainState,
) -> Result<(), ComparisonFailure> {
    validate_observed_endpoints(expected, observed)?;
    compare_relation_maps(expected, observed)?;
    compare_relation_vector_sets(expected, observed)?;
    validate_relation_vector(
        &expected.schema,
        &observed.relation_vector,
        &observed.entities,
    )?;
    Ok(())
}

pub fn validate_observed_endpoints(
    expected: &ExpectedSupplyChainObservation,
    observed: &ObservedSupplyChainState,
) -> Result<(), ComparisonFailure> {
    let mut relation_keys = std::collections::BTreeSet::new();
    for edge in &observed.relation_vector {
        if !relation_keys.insert(edge.key) {
            return Err(ComparisonFailure {
                mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::DuplicateRelation(edge.key),
            });
        }
        let Some(source) = observed.entities.get(&edge.source).map(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::kind) else {
            return Err(ComparisonFailure {
                mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::IllegalEndpoint(
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::InvalidEndpoint {
                        relation: edge.key.kind,
                        source: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
                        target: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
                    },
                ),
            });
        };
        let Some(target) = observed.entities.get(&edge.target).map(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::kind) else {
            return Err(ComparisonFailure {
                mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::IllegalEndpoint(
                    crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::InvalidEndpoint {
                        relation: edge.key.kind,
                        source,
                        target: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
                    },
                ),
            });
        };
        expected
            .schema
            .validate_edge(*edge, source, target)
            .map_err(|error| ComparisonFailure {
                mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::IllegalEndpoint(error),
            })?;
    }
    Ok(())
}

pub fn compare_relation_maps(
    expected: &ExpectedSupplyChainObservation,
    observed: &ObservedSupplyChainState,
) -> Result<(), ComparisonFailure> {
    for (key, expected_edge) in &expected.relations {
        let Some(observed_edge) = observed.relations.get(key) else {
            return Err(ComparisonFailure {
                mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::MissingRelation(*key),
            });
        };
        if expected_edge.source != observed_edge.source {
            return Err(ComparisonFailure {
                mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::RelationSource(*key),
            });
        }
        if expected_edge.target != observed_edge.target {
            return Err(ComparisonFailure {
                mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::RelationTarget(*key),
            });
        }
    }
    if let Some(key) = observed
        .relations
        .keys()
        .find(|key| !expected.relations.contains_key(key))
    {
        return Err(ComparisonFailure {
            mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::UnexpectedRelation(*key),
        });
    }
    if expected.absent_relations != observed.absent_relations {
        let key = expected
            .absent_relations
            .symmetric_difference(&observed.absent_relations)
            .next()
            .copied()
            .unwrap_or(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::TerminalAtPort, u32::MAX));
        return Err(ComparisonFailure {
            mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::RelationAbsence(key),
        });
    }
    Ok(())
}
