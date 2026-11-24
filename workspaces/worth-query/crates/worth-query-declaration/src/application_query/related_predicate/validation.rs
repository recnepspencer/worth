//! Definition-level validation for related-record absence/existence predicates,
//! shared by declaration validation, package-archive encoding, and
//! package-archive decoding.
use super::{
    ApplicationQueryRelatedPolarity, ApplicationQueryRelatedPredicateMeaning,
    ApplicationQueryRelatedTargetPredicate,
};
use crate::application_query::{
    predicate_literal_is_valid, where_filter, ApplicationQueryDefinitionDenial as Denial,
    ApplicationQueryParameterDefinition, ApplicationQueryPredicateExpected,
    ApplicationQueryWhereFilterProgram, ApplicationQueryWhereFilterSource as Source,
    WorthQueryPortableApplicationQueryParts,
};
use std::cmp::Ordering;
use worth_foundational::facade::AspectValue;

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
/// endpoints, direction, target predicates, and target filter) in O(N) time
/// using the already-verified canonical sort order (`Absent` precedes `Exists`
/// within each contiguous `(relation, from, to, direction)` group).
fn reject_contradictory_related_clauses(
    predicates: &[ApplicationQueryRelatedPredicateMeaning],
) -> Result<(), Denial> {
    let same_edge = |a: &ApplicationQueryRelatedPredicateMeaning,
                     b: &ApplicationQueryRelatedPredicateMeaning| {
        a.relation() == b.relation()
            && a.from() == b.from()
            && a.to() == b.to()
            && a.direction() == b.direction()
    };
    fn matcher(
        p: &ApplicationQueryRelatedPredicateMeaning,
    ) -> (
        &[ApplicationQueryRelatedTargetPredicate],
        Option<&ApplicationQueryWhereFilterProgram>,
    ) {
        (p.target_predicates(), p.target_where_filter())
    }
    let mut start = 0;
    while start < predicates.len() {
        let mut end = start + 1;
        while end < predicates.len() && same_edge(&predicates[start], &predicates[end]) {
            end += 1;
        }
        let group = &predicates[start..end];
        let split =
            group.partition_point(|p| p.polarity() == ApplicationQueryRelatedPolarity::Absent);
        let (absent, exists) = group.split_at(split);
        let (mut a, mut e) = (0, 0);
        while a < absent.len() && e < exists.len() {
            match matcher(&absent[a]).cmp(&matcher(&exists[e])) {
                Ordering::Equal => return Err(Denial::RelatedPredicateContradictory),
                Ordering::Less => a += 1,
                Ordering::Greater => e += 1,
            }
        }
        start = end;
    }
    Ok(())
}

/// Rejects contradictory literal equalities (two distinct values on the same
/// target field) in O(M) time using the already-verified canonical sort order
/// where predicates on the same `(entity, aspect, field)` are contiguous.
fn reject_contradictory_target_literals(
    targets: &[ApplicationQueryRelatedTargetPredicate],
) -> Result<(), Denial> {
    let mut last: Option<((&str, &str, &str), &AspectValue)> = None;
    for target in targets {
        let ApplicationQueryPredicateExpected::Value(val) = target.expected() else {
            continue;
        };
        let field = target.field();
        if let Some((prev_field, prev_val)) = last
            && prev_field == field
            && prev_val != val
        {
            return Err(Denial::RelatedPredicateContradictory);
        }
        last = Some((field, val));
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
