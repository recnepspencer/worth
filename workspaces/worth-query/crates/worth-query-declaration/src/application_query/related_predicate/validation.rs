//! Definition-level validation for related-record absence/existence predicates.
use super::ApplicationQueryRelatedPredicateMeaning;
use crate::application_query::{
    predicate_literal_is_valid, where_filter, ApplicationQueryDefinitionDenial as Denial,
    ApplicationQueryParameterDefinition, ApplicationQueryPredicateExpected,
    ApplicationQueryWhereFilterSource as Source, WorthQueryPortableApplicationQueryParts,
};

pub(in crate::application_query) fn validate(
    definition: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), Denial> {
    if definition.related_predicates().is_empty() {
        return Ok(());
    }
    if definition.orbit().is_some()
        || definition.component_selection().is_some()
        || definition.managed_index().is_some()
        || definition.continuation().is_some()
    {
        return Err(Denial::RelatedPredicateUnsupportedShape);
    }
    for predicate in definition.related_predicates() {
        validate_one(predicate, definition.root_entity(), definition.parameters())?;
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
    let target_entity = predicate.target_entity();
    for target in predicate.target_predicates() {
        let (entity, aspect, field) = target.field();
        if !identifier_is_clean(entity)
            || !identifier_is_clean(aspect)
            || !identifier_is_clean(field)
        {
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

fn identifier_is_clean(value: &str) -> bool {
    !value.is_empty() && value.trim() == value
}
