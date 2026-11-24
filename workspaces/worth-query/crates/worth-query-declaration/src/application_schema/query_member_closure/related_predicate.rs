//! Schema-closure validation for related-record absence/existence predicates:
//! every traversed relation and inspected target field must exist on the
//! declared schema with exact endpoints, scalar family, presence, and unitless
//! discipline.
use crate::application_query::{
    predicate_literal_is_valid, ApplicationQueryPredicateExpected,
    ApplicationQueryWhereFilterSource as Source, ErasedApplicationQueryDefinition,
};
use crate::application_schema::{
    ApplicationFieldPresence, ApplicationSchemaDeclarationDenial, ApplicationSchemaMember,
};

use super::super::schema_read_closure::{entity_exists, field_is_unitless, relation_exists};

pub(super) fn validate(
    definition: &ErasedApplicationQueryDefinition,
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    for predicate in definition.related_predicates() {
        if predicate.root_entity() != definition.root_entity()
            || !relation_exists(
                members,
                predicate.relation(),
                predicate.from(),
                predicate.to(),
            )
            || !entity_exists(members, predicate.target_entity())
        {
            return Err(ApplicationSchemaDeclarationDenial::InvalidApplicationQuery);
        }
        let target_entity = predicate.target_entity();
        for target in predicate.target_predicates() {
            let (entity, aspect, field) = target.field();
            if entity != target_entity
                || !target_field_matches(
                    members,
                    entity,
                    aspect,
                    field,
                    target.scalar_family(),
                    target.presence(),
                )
            {
                return Err(ApplicationSchemaDeclarationDenial::InvalidApplicationQuery);
            }
            let expected_ok = match target.expected() {
                ApplicationQueryPredicateExpected::Parameter(name) => {
                    definition.parameters().iter().any(|parameter| {
                        parameter.name() == name
                            && parameter.scalar_family() == target.scalar_family()
                            && !parameter.is_optional()
                            && parameter.unit().is_none()
                    })
                }
                ApplicationQueryPredicateExpected::Value(value) => {
                    field_is_unitless(members, entity, aspect, field)
                        && predicate_literal_is_valid(value, target.scalar_family())
                }
            };
            if !expected_ok {
                return Err(ApplicationSchemaDeclarationDenial::InvalidApplicationQuery);
            }
        }
        if let Some(filter) = predicate.target_where_filter() {
            for operand in filter.operands() {
                let Source::Field {
                    entity,
                    aspect,
                    field,
                    scalar_family,
                    presence,
                } = operand.source()
                else {
                    continue;
                };
                if entity != target_entity
                    || !target_field_matches(
                        members,
                        entity,
                        aspect,
                        field,
                        *scalar_family,
                        *presence,
                    )
                {
                    return Err(ApplicationSchemaDeclarationDenial::InvalidApplicationQuery);
                }
            }
        }
    }
    Ok(())
}

fn target_field_matches(
    members: &[ApplicationSchemaMember],
    entity: &str,
    aspect: &str,
    field: &str,
    scalar_family: worth_foundational::facade::ScalarAspectType,
    presence: ApplicationFieldPresence,
) -> bool {
    members.iter().any(|member| {
        matches!(
            member,
            ApplicationSchemaMember::Field {
                entity: declared_entity,
                aspect: declared_aspect,
                field: declared_field,
                scalar_family: declared_scalar,
                presence: declared_presence,
                unit: None,
                ..
            } if declared_entity == entity
                && declared_aspect == aspect
                && declared_field == field
                && *declared_scalar == scalar_family
                && *declared_presence == presence
        )
    })
}
