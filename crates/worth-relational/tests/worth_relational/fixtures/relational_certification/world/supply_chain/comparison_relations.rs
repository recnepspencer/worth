pub(crate) use std::collections::BTreeSet;

pub(crate) use super::comparison_state::{
    ComparisonFailure, ComparisonMismatch, ObservedSupplyChainState,
};
pub(crate) use super::expected_observation::ExpectedSupplyChainObservation;
pub(crate) use super::relation_comparison::{
    compare_relation_vector_sets, validate_relation_vector,
};
pub(crate) use super::schema::EntityRecord;
pub(crate) use super::semantic_key::{EntityKind, RelationKey, RelationKind};

pub(crate) fn compare_relations(
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

pub(crate) fn validate_observed_endpoints(
    expected: &ExpectedSupplyChainObservation,
    observed: &ObservedSupplyChainState,
) -> Result<(), ComparisonFailure> {
    let mut relation_keys = BTreeSet::new();
    for edge in &observed.relation_vector {
        if !relation_keys.insert(edge.key) {
            return Err(ComparisonFailure {
                mismatch: ComparisonMismatch::DuplicateRelation(edge.key),
            });
        }
        let Some(source) = observed.entities.get(&edge.source).map(EntityRecord::kind) else {
            return Err(ComparisonFailure {
                mismatch: ComparisonMismatch::IllegalEndpoint(
                    super::schema::SchemaError::InvalidEndpoint {
                        relation: edge.key.kind,
                        source: EntityKind::Port,
                        target: EntityKind::Port,
                    },
                ),
            });
        };
        let Some(target) = observed.entities.get(&edge.target).map(EntityRecord::kind) else {
            return Err(ComparisonFailure {
                mismatch: ComparisonMismatch::IllegalEndpoint(
                    super::schema::SchemaError::InvalidEndpoint {
                        relation: edge.key.kind,
                        source,
                        target: EntityKind::Port,
                    },
                ),
            });
        };
        expected
            .schema
            .validate_edge(*edge, source, target)
            .map_err(|error| ComparisonFailure {
                mismatch: ComparisonMismatch::IllegalEndpoint(error),
            })?;
    }
    Ok(())
}

pub(crate) fn compare_relation_maps(
    expected: &ExpectedSupplyChainObservation,
    observed: &ObservedSupplyChainState,
) -> Result<(), ComparisonFailure> {
    for (key, expected_edge) in &expected.relations {
        let Some(observed_edge) = observed.relations.get(key) else {
            return Err(ComparisonFailure {
                mismatch: ComparisonMismatch::MissingRelation(*key),
            });
        };
        if expected_edge.source != observed_edge.source {
            return Err(ComparisonFailure {
                mismatch: ComparisonMismatch::RelationSource(*key),
            });
        }
        if expected_edge.target != observed_edge.target {
            return Err(ComparisonFailure {
                mismatch: ComparisonMismatch::RelationTarget(*key),
            });
        }
    }
    if let Some(key) = observed
        .relations
        .keys()
        .find(|key| !expected.relations.contains_key(key))
    {
        return Err(ComparisonFailure {
            mismatch: ComparisonMismatch::UnexpectedRelation(*key),
        });
    }
    if expected.absent_relations != observed.absent_relations {
        let key = expected
            .absent_relations
            .symmetric_difference(&observed.absent_relations)
            .next()
            .copied()
            .unwrap_or(RelationKey::new(RelationKind::TerminalAtPort, u32::MAX));
        return Err(ComparisonFailure {
            mismatch: ComparisonMismatch::RelationAbsence(key),
        });
    }
    Ok(())
}
