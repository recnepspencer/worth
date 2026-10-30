pub use {std::collections::BTreeMap, std::collections::BTreeSet};

pub use {crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonFailure, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::RelationEdge, crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError, crate::relational_certification::ordinary_context::world::supply_chain::schema::SupplyChainSchema};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind};

pub fn compare_relation_vector_sets(
    expected: &ExpectedSupplyChainObservation,
    observed: &ObservedSupplyChainState,
) -> Result<(), ComparisonFailure> {
    let mut observed_by_key = std::collections::BTreeMap::new();
    for edge in &observed.relation_vector {
        if observed_by_key.insert(edge.key, *edge).is_some() {
            return Err(ComparisonFailure {
                mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::DuplicateRelation(edge.key),
            });
        }
    }
    for (key, expected_edge) in &expected.relations {
        let Some(observed_edge) = observed_by_key.get(key) else {
            return Err(ComparisonFailure {
                mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::MissingRelationVector(*key),
            });
        };
        if observed_edge != expected_edge {
            return Err(ComparisonFailure {
                mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::RelationVectorValue(*key),
            });
        }
    }
    if let Some(key) = observed_by_key
        .keys()
        .find(|key| !expected.relations.contains_key(key))
    {
        return Err(ComparisonFailure {
            mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::UnexpectedRelationVector(*key),
        });
    }
    Ok(())
}

pub fn validate_relation_vector(
    schema: &SupplyChainSchema,
    relations: &[RelationEdge],
    entities: &BTreeMap<EntityKey, EntityRecord>,
) -> Result<(), ComparisonFailure> {
    let mut seen = std::collections::BTreeSet::new();
    for edge in relations {
        if !seen.insert(edge.key) {
            return Err(ComparisonFailure {
                mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::DuplicateRelation(edge.key),
            });
        }
        let Some(source) = entities.get(&edge.source).map(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::kind) else {
            return Err(ComparisonFailure {
                mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::IllegalEndpoint(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::InvalidEndpoint {
                    relation: edge.key.kind,
                    source: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
                    target: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
                }),
            });
        };
        let Some(target) = entities.get(&edge.target).map(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::kind) else {
            return Err(ComparisonFailure {
                mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::IllegalEndpoint(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::InvalidEndpoint {
                    relation: edge.key.kind,
                    source,
                    target: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
                }),
            });
        };
        schema
            .validate_edge(*edge, source, target)
            .map_err(|error| ComparisonFailure {
                mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::IllegalEndpoint(error),
            })?;
    }
    schema
        .validate_relation_sequence(relations, entities)
        .map_err(|error| ComparisonFailure {
            mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::IllegalEndpoint(error),
        })?;
    Ok(())
}
