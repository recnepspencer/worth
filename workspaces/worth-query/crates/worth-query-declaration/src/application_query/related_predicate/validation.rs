//! Definition-level validation for related-record absence/existence predicates,
//! shared by declaration validation, package-archive encoding, and
//! package-archive decoding.
use super::{ApplicationQueryRelatedPredicateMeaning, ApplicationQueryRelatedTargetPredicate};
use crate::application_query::{
    predicate_literal_is_valid, where_filter, ApplicationQueryDefinitionDenial as Denial,
    ApplicationQueryParameterDefinition, ApplicationQueryPredicateExpected,
    ApplicationQueryWhereFilterSource as Source, WorthQueryPortableApplicationQueryParts,
};

pub fn validate(definition: &WorthQueryPortableApplicationQueryParts) -> Result<(), Denial> {
    let predicates = definition.related_predicates();
    if predicates.is_empty() {
        return Ok(());
    }
    if definition.orbit().is_some()
        || definition.component_selection().is_some()
        || definition.managed_index().is_some()
        || definition.continuation().is_some()
    {
        return Err(Denial::RelatedPredicateUnsupportedShape);
    }
    if !predicates.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(Denial::InvalidCanonicalOrdering);
    }
    reject_contradictory_related_clauses(predicates)?;
    for predicate in predicates {
        if !predicate
            .target_predicates()
            .windows(2)
            .all(|pair| pair[0] < pair[1])
        {
            return Err(Denial::InvalidCanonicalOrdering);
        }
        validate_one(predicate, definition.root_entity(), definition.parameters())?;
    }
    Ok(())
}

/// Rejects `Absent` + `Exists` over the identical matcher (same relation,
/// endpoints, direction, target predicates, and target filter) while allowing
/// distinct filters on the same relation.
fn reject_contradictory_related_clauses(
    predicates: &[ApplicationQueryRelatedPredicateMeaning],
) -> Result<(), Denial> {
    for (idx, left) in predicates.iter().enumerate() {
        for right in &predicates[idx + 1..] {
            if left.relation() == right.relation()
                && left.from() == right.from()
                && left.to() == right.to()
                && left.direction() == right.direction()
                && left.polarity() != right.polarity()
                && left.target_predicates() == right.target_predicates()
                && left.target_where_filter() == right.target_where_filter()
            {
                return Err(Denial::RelatedPredicateContradictory);
            }
        }
    }
    Ok(())
}

/// Rejects contradictory literal equalities (two distinct values on the same
/// target field) within a single related predicate.
fn reject_contradictory_target_literals(
    targets: &[ApplicationQueryRelatedTargetPredicate],
) -> Result<(), Denial> {
    for (idx, left) in targets.iter().enumerate() {
        let ApplicationQueryPredicateExpected::Value(left_val) = left.expected() else {
            continue;
        };
        for right in &targets[idx + 1..] {
            if left.field() == right.field()
                && let ApplicationQueryPredicateExpected::Value(right_val) = right.expected()
                && left_val != right_val
            {
                return Err(Denial::RelatedPredicateContradictory);
            }
        }
    }
    Ok(())
}

pub(super) fn validate_one(
    predicate: &ApplicationQueryRelatedPredicateMeaning,
    root_entity: &str,
    parameters: &[ApplicationQueryParameterDefinition],
) -> Result<(), Denial> {
    if !identifier_is_clean(predicate.relation())
        || !identifier_is_clean(predicate.from())
        || !identifier_is_clean(predicate.to())
    {
        return Err(Denial::EmptyIdentifier);
    }
    if predicate.root_entity() != root_entity {
        return Err(Denial::RelatedPredicateEndpointMismatch);
    }
    reject_contradictory_target_literals(predicate.target_predicates())?;
    let target_entity = predicate.target_entity();
    for target in predicate.target_predicates() {
        validate_target_predicate(target, target_entity, parameters)?;
    }
    if let Some(filter) = predicate.target_where_filter() {
        for operand in filter.operands() {
            if let Source::Field {
                entity,
                aspect,
                field,
                ..
            } = operand.source()
            {
                if !identifier_is_clean(entity)
                    || !identifier_is_clean(aspect)
                    || !identifier_is_clean(field)
                {
                    return Err(Denial::EmptyIdentifier);
                }
                if entity != target_entity {
                    return Err(Denial::RelatedPredicateTargetFieldMismatch);
                }
            }
        }
        where_filter::validation::validate_parameter_operands(parameters, filter)?;
    }
    Ok(())
}

fn validate_target_predicate(
    target: &ApplicationQueryRelatedTargetPredicate,
    target_entity: &str,
    parameters: &[ApplicationQueryParameterDefinition],
) -> Result<(), Denial> {
    let (entity, aspect, field) = target.field();
    if !identifier_is_clean(entity) || !identifier_is_clean(aspect) || !identifier_is_clean(field) {
        return Err(Denial::EmptyIdentifier);
    }
    if entity != target_entity {
        return Err(Denial::RelatedPredicateTargetFieldMismatch);
    }
    match target.expected() {
        ApplicationQueryPredicateExpected::Parameter(name) => {
            if !identifier_is_clean(name) {
                return Err(Denial::EmptyIdentifier);
            }
            let Some(declared) = parameters.iter().find(|p| p.name() == name) else {
                return Err(Denial::UnknownPredicateParameter);
            };
            if declared.scalar_family() != target.scalar_family()
                || declared.is_optional()
                || declared.unit().is_some()
            {
                return Err(Denial::PredicateParameterMismatch);
            }
        }
        ApplicationQueryPredicateExpected::Value(value) => {
            if !predicate_literal_is_valid(value, target.scalar_family()) {
                return Err(Denial::InvalidPredicateValue);
            }
        }
    }
    Ok(())
}

fn identifier_is_clean(value: &str) -> bool {
    !value.is_empty() && value.trim() == value
}
