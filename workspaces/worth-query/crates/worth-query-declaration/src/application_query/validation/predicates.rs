use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};

use crate::application_query::{
    ApplicationQueryRelationPredicate, WorthQueryPortableApplicationQueryParts,
};

use super::result_shape;
use super::ApplicationQueryDefinitionDenial;

pub(crate) const MAXIMUM_PORTABLE_PREDICATE_LITERAL_BYTES: usize = 64 * 1024;

pub(crate) fn predicate_literal_is_valid(
    value: &AspectValue,
    scalar_family: ScalarAspectType,
) -> bool {
    scalar_family != ScalarAspectType::Null
        && value.value_family() == scalar_family
        && value.has_canonical_representation()
        && !matches!(value, AspectValue::String(InternedString::Symbol(_)))
        && value.semantic_byte_width() <= MAXIMUM_PORTABLE_PREDICATE_LITERAL_BYTES
}

pub(super) fn validate(
    definition: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), ApplicationQueryDefinitionDenial> {
    let relation_predicates = result_shape::relation_predicates(definition.result_shape());
    for predicate in &relation_predicates {
        if matches!(
            predicate,
            ApplicationQueryRelationPredicate::EqualParameter(inner) if inner.parameter().is_none()
        ) {
            return Err(ApplicationQueryDefinitionDenial::PredicateParameterMismatch);
        }
    }
    for predicate in definition.predicates() {
        if let Some(value) = predicate.literal_value()
            && !predicate_literal_is_valid(value, predicate.scalar_family())
        {
            return Err(ApplicationQueryDefinitionDenial::InvalidPredicateValue);
        }
    }
    let predicates = definition
        .predicates()
        .iter()
        .map(|predicate| (predicate.parameter(), predicate.scalar_family()))
        .chain(
            relation_predicates
                .iter()
                .map(|predicate| (predicate.parameter(), predicate.scalar_family())),
        );
    for (parameter_name, scalar_family) in predicates {
        let Some(parameter_name) = parameter_name else {
            continue;
        };
        let Some(parameter) = definition
            .parameters()
            .iter()
            .find(|parameter| parameter.name() == parameter_name)
        else {
            return Err(ApplicationQueryDefinitionDenial::UnknownPredicateParameter);
        };
        if parameter.scalar_family() != scalar_family || parameter.is_optional() {
            return Err(ApplicationQueryDefinitionDenial::PredicateParameterMismatch);
        }
    }
    if definition.predicates().iter().any(|predicate| {
        !result_shape::contains_entity(definition.result_shape(), predicate.field().0)
    }) {
        return Err(ApplicationQueryDefinitionDenial::ResultRootMismatch);
    }
    if !result_shape::relation_predicate_targets_are_valid(definition.result_shape()) {
        return Err(ApplicationQueryDefinitionDenial::RelationPredicateTargetMismatch);
    }
    Ok(())
}
