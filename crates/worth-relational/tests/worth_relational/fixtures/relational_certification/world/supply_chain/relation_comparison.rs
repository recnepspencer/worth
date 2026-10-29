pub(crate) use std::collections::{BTreeMap, BTreeSet};

pub(crate) use super::comparison_state::{
    ComparisonFailure, ComparisonMismatch, ObservedSupplyChainState,
};
pub(crate) use super::expected_observation::ExpectedSupplyChainObservation;
pub(crate) use super::schema::{EntityRecord, RelationEdge, SchemaError, SupplyChainSchema};
pub(crate) use super::semantic_key::{EntityKey, EntityKind};

pub(crate) fn compare_relation_vector_sets(
    expected: &ExpectedSupplyChainObservation,
    observed: &ObservedSupplyChainState,
) -> Result<(), ComparisonFailure> {
    let mut observed_by_key = BTreeMap::new();
    for edge in &observed.relation_vector {
        if observed_by_key.insert(edge.key, *edge).is_some() {
            return Err(ComparisonFailure {
                mismatch: ComparisonMismatch::DuplicateRelation(edge.key),
            });
        }
    }
    for (key, expected_edge) in &expected.relations {
        let Some(observed_edge) = observed_by_key.get(key) else {
            return Err(ComparisonFailure {
                mismatch: ComparisonMismatch::MissingRelationVector(*key),
            });
        };
        if observed_edge != expected_edge {
            return Err(ComparisonFailure {
                mismatch: ComparisonMismatch::RelationVectorValue(*key),
            });
        }
    }
    if let Some(key) = observed_by_key
        .keys()
        .find(|key| !expected.relations.contains_key(key))
    {
        return Err(ComparisonFailure {
            mismatch: ComparisonMismatch::UnexpectedRelationVector(*key),
        });
    }
    Ok(())
}

pub(crate) fn validate_relation_vector(
    schema: &SupplyChainSchema,
    relations: &[RelationEdge],
    entities: &BTreeMap<EntityKey, EntityRecord>,
) -> Result<(), ComparisonFailure> {
    let mut seen = BTreeSet::new();
    for edge in relations {
        if !seen.insert(edge.key) {
            return Err(ComparisonFailure {
                mismatch: ComparisonMismatch::DuplicateRelation(edge.key),
            });
        }
        let Some(source) = entities.get(&edge.source).map(EntityRecord::kind) else {
            return Err(ComparisonFailure {
                mismatch: ComparisonMismatch::IllegalEndpoint(SchemaError::InvalidEndpoint {
                    relation: edge.key.kind,
                    source: EntityKind::Port,
                    target: EntityKind::Port,
                }),
            });
        };
        let Some(target) = entities.get(&edge.target).map(EntityRecord::kind) else {
            return Err(ComparisonFailure {
                mismatch: ComparisonMismatch::IllegalEndpoint(SchemaError::InvalidEndpoint {
                    relation: edge.key.kind,
                    source,
                    target: EntityKind::Port,
                }),
            });
        };
        schema
            .validate_edge(*edge, source, target)
            .map_err(|error| ComparisonFailure {
                mismatch: ComparisonMismatch::IllegalEndpoint(error),
            })?;
    }
    schema
        .validate_relation_sequence(relations, entities)
        .map_err(|error| ComparisonFailure {
            mismatch: ComparisonMismatch::IllegalEndpoint(error),
        })?;
    Ok(())
}
